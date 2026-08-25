extends RefCounted
class_name DccToolBar

## The unified Sculpt / Paint / Measure tool bar —
## `design/Cartalith Paint Toolbar.dc.html`, whose own caption is the whole
## specification: *"one bar. three mode buttons on the left; the active mode's
## tools appear beside them. Options bar below carries brush size."*
##
## ## Where the canvas's two bars land in this shell
##
## The canvas draws two: a tool bar (modes + the active mode's tools) and a
## contextual options bar under it. `DccShell` has exactly one bar in that
## position (`tool_options_row`, §4), and growing the shell a second one would
## mean editing `dcc_shell.gd`'s desktop *and* phone compositions for a
## presentation change. So this file puts a two-row `VBoxContainer` inside the
## one bar: row 1 is the canvas's tool bar, row 2 its options bar. The
## `PanelContainer` around it has a *minimum* height, so it simply grows, and
## `_phone_fit_tool_options()` already recurses through arbitrary containers —
## nothing in `dcc_shell.gd` changed.
##
## ## The three modes map onto three tool ids that already exist
##
## SCULPT arms `"sculpt"`, PAINT arms `"paint"`, MEASURE arms `"measure"`.
## All three were already registered — by `world_workspace.gd` for the first
## two and `global_tools.gd` for the third — and all three are registered on
## `DccApp` globally rather than per domain, so switching modes here works
## from any workspace. **No sculpt or paint engine logic is reimplemented in
## this file**: every control writes through the same `bridge.sculpt_*` /
## `bridge.paint_*` call the left-dock panels already use, so the two views
## of one engine state cannot disagree — `world_workspace.gd` re-reads the
## engine on every rebuild.
##
## ## Where the canvas asks for something that does not exist
##
## Read against the real registries rather than the canvas's labels:
##
## - **Sculpt tools.** The canvas draws Raise · Lower · Smooth · Flatten ·
##   Noise · Ridge · Carve. The engine's own list is
##   `FreehandMode` — raise, lower, smooth, cliff, ridge, canyon, mesa,
##   volcano. Five of the canvas's seven are that list by another name (Carve
##   is Canyon); **Flatten and Noise have no engine mode at all**. This row is
##   built from `get_sculpt_freehand_modes()` live, so it shows the eight that
##   exist rather than seven of which two would do nothing — the same
##   read-the-registry rule `world_workspace.gd`'s own header already states.
## - **Paint tools.** The canvas draws Terrain · Biome · Water · Lithology ·
##   Custom field. `PaintTarget` is Biome, Terrain and Splat: **Water and
##   Lithology are not paintable** (neither has an override array for a dab to
##   write into), and Splat is the closest thing to the canvas's "Custom
##   field". Built from `get_paint_layers()` for the same reason.
## - **Mask.** Both Sculpt and Paint rows end in a Mask button. No mask
##   channel exists in either editor; it is disclosed in the row's trailing
##   note rather than drawn as a button that would do nothing.

## The one instance, held by `DccApp` so `global_tools.gd` can ask it to
## redraw when the measure mode changes.
static var _instance: DccToolBar = null

var app: DccApp
var bridge: EngineBridge

## Which of the three the bar is drawing. Follows `app.armed_tool` when that
## is one of the three, and otherwise stays on whatever it last showed so the
## bar does not blank while Inspect is armed.
var mode := "measure"

static func install(a: DccApp, b: EngineBridge) -> DccToolBar:
	var bar := DccToolBar.new()
	bar.app = a
	bar.bridge = b
	_instance = bar
	a.tool_armed.connect(bar._on_tool_armed)
	b.generation_finished.connect(func(_ok: bool): bar.refresh())
	b.world_loaded.connect(func(): bar.refresh())
	return bar

static func instance() -> DccToolBar:
	return _instance

const MODES := ["sculpt", "paint", "measure"]

func _on_tool_armed(id: String) -> void:
	if not MODES.has(id):
		## Leaving the three for Inspect (or any domain tool) hands the bar
		## back to whoever owns it for that context -- `app.gd`'s own
		## `_on_workspace_changed` default, re-applied here rather than left
		## showing a mode nobody has armed.
		if id == "inspect":
			app._on_workspace_changed(app.active_domain())
		return
	mode = id
	rebuild()

## Re-draws only if the bar is currently ours -- called after a generate, a
## load, or a measure-mode change.
func refresh() -> void:
	if MODES.has(app.armed_tool):
		rebuild()

func rebuild() -> void:
	app.set_tool_options(_build)

# -- Composition ---------------------------------------------------------------

func _build(row: HBoxContainer) -> void:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(col)

	var tools := HBoxContainer.new()
	tools.add_theme_constant_override("separation", 6)
	col.add_child(tools)
	_build_mode_buttons(tools)
	tools.add_child(DccTheme.rule(true))
	match mode:
		"sculpt": _build_sculpt_tools(tools)
		"paint": _build_paint_tools(tools)
		_: _build_measure_tools(tools)

	var options := HBoxContainer.new()
	options.add_theme_constant_override("separation", 8)
	col.add_child(options)
	match mode:
		"sculpt": _build_sculpt_options(options)
		"paint": _build_paint_options(options)
		_: _build_measure_options(options)

## The canvas's three left-hand mode buttons. `Cartalith Paint Toolbar.dc.html`
## draws the armed one **filled** -- the single filled surface anywhere in the
## tool bar, and the one exception to §48 DS-02's "the canvas has no filled
## buttons" (see `DccWidgets.set_mode_segment_on()` for the full reading). It
## also gives them `padding:5px 12px`, a taller box than the feature segments
## beside them, which is how the row reads as mode-then-tool rather than as
## eleven equal chips.
func _build_mode_buttons(row: HBoxContainer) -> void:
	for m in MODES:
		var b := DccWidgets.segment(row, String(m).to_upper(), _select_mode.bind(m))
		DccWidgets.set_mode_segment_on(b, mode == m)
		b.custom_minimum_size.y = 22
		if m != "measure" and not bridge.has_world:
			b.disabled = true
			b.tooltip_text = "Sculpt and Paint edit a generated world; generate one first (World ▸ Generate)."

func _select_mode(m: String) -> void:
	mode = m
	app.arm_tool(m)
	## `arm_tool` is a no-op when the id is already armed, so the rebuild is
	## unconditional here rather than left to `_on_tool_armed`.
	rebuild()

## Every tool row lights exactly one of its own buttons. Shared rather than
## repeated three times -- the difference between the rows is which list they
## iterate, not how a lit segment is drawn.
func _tool_segment(row: HBoxContainer, text: String, on: bool, tip: String, press: Callable) -> Button:
	var b := DccWidgets.segment(row, text, press)
	b.custom_minimum_size.y = 22
	b.tooltip_text = tip
	DccWidgets.set_segment_on(b, on)
	return b

## The trailing note each tool row ends with. **`clip_text` is load-bearing,
## not cosmetic**: a `Label` reports its full text width as its minimum size,
## and this bar sits in the shell's own top-level `VBoxContainer`, so one long
## sentence here raised the whole window's minimum width and pushed the right
## dock off the screen entirely. Caught in the first live pass, in the Sculpt
## row, which has the longest note. The visible half is short; `detail` is the
## full disclosure, on hover.
## `DccWidgets`' rows are dock-width by construction (`ROW_LABEL_W` is 132,
## `ROW_VALUE_W` 56). A bar is not a dock: four of those side by side raised
## the shell's own minimum width past the window and clipped the right dock,
## which is how the Sculpt row was found in the first live pass. Same widget,
## same styling, same call -- only the two fixed columns are pulled in
## afterwards, so nothing about `dcc_widgets.gd` had to change for a caller
## it was not written for.
## 56 fitted the prose face; Plex Mono is wider per character at the same size,
## and the first capture after the face changed had `Brush size` clipped to
## `Brush si` and `Intensity` losing its `y`. The engine's own control labels
## are longer than the canvas's `size` / `strength` / `falloff`, so this is the
## width those labels need rather than the width the canvas happens to use.
const BAR_LABEL_W := 74
const BAR_VALUE_W := 38
## `width:96px` / `78px` / `70px` on the three tracks in `Cartalith Paint
## Toolbar.dc.html`'s `Sculpt raise 1920`, and `width:70px` in the DCC shell's
## own tool options bar. 56 was a bar-specific narrowing of the dock's 78 that
## no artboard asks for; 70 is what the shell canvas draws, and the widest the
## bar can afford with four parameters and a commit pair on a 1600 px window.
const BAR_CONTROL_W := 70

func _fit(parts: Dictionary) -> void:
	_narrow(parts["row"])
	if parts.has("readout"):
		(parts["readout"] as Control).custom_minimum_size.x = BAR_VALUE_W
	if parts.has("slider"):
		(parts["slider"] as Control).custom_minimum_size.x = BAR_CONTROL_W

## For `choice`/`toggle`, which return the control rather than its parts.
func _narrow(ctrl: Control) -> Control:
	var row: Control = ctrl if ctrl is HBoxContainer else (ctrl.get_parent() as Control)
	if row != null and row.get_child_count() > 0:
		var label := row.get_child(0) as Control
		label.custom_minimum_size.x = BAR_LABEL_W
		## **The bar is Plex; the dock is prose.** Every artboard that draws a
		## tool options bar sets the whole row in
		## `font:11px 'IBM Plex Mono';color:#8d9296` and lets its labels inherit
		## -- `DCC shell 1920`'s `hardness` / `intensity` / `noise`, the tablet's
		## same three at 13 px, and the Paint Toolbar's `size` / `strength` /
		## `falloff` / `spacing` / `max Δ` / `pressure`. `DccWidgets._row()`
		## paints its label prose in `text_secondary`, which is right for the
		## *dock* (§48 DS-06 read that off the dock's own parameter row) and
		## wrong here. Repainted at the one place every bar row already passes
		## through rather than by giving `_row()` a second mode.
		if label is Label:
			(label as Label).add_theme_font_override("font", DccTheme.mono(0))
			(label as Label).add_theme_color_override("font_color",
				DccTheme.c("text_dim"))
	return ctrl

## The options row's own hint, same clipping rule as `_note` and without its
## leading spacer -- these sit inline with the controls, not hard right.
func _bar_hint(row: HBoxContainer, text: String, detail: String) -> void:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.clip_text = true
	l.tooltip_text = detail if detail != "" else text
	l.mouse_filter = Control.MOUSE_FILTER_STOP
	row.add_child(l)

func _note(row: HBoxContainer, text: String, detail: String = "") -> void:
	row.add_child(DccTheme.spacer())
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.clip_text = true
	l.tooltip_text = detail if detail != "" else text
	l.mouse_filter = Control.MOUSE_FILTER_STOP
	row.add_child(l)

# -- SCULPT --------------------------------------------------------------------

func _build_sculpt_tools(row: HBoxContainer) -> void:
	if not bridge.has_world or bridge.sculpt_get_globals().is_empty():
		_note(row, "no sculpt editor for this world", "Only a freshly generated world has a draft session; a loaded save carries none.")
		return
	var current := bridge.sculpt_get_freehand_mode()
	var on_freehand := bridge.sculpt_get_feature() == "freehand"
	for m in bridge.get_sculpt_freehand_modes():
		var key := String(m)
		_tool_segment(row, key.capitalize(), on_freehand and key == current,
			"Freehand ▸ %s. Raise/Lower/Smooth follow the drag; Cliff/Ridge/Canyon follow its direction; Mesa/Volcano stamp once at a tap." % key.capitalize(),
			_on_sculpt_mode.bind(key))
	_note(row, "hand edits are local — they mark downstream stages stale",
		"The canvas's Flatten, Noise and Mask have no engine mode at all; these eight are `FreehandMode`'s own list, read live. The 13 geological features live in the WORLD dock's Sculpt panel.")

func _on_sculpt_mode(key: String) -> void:
	bridge.sculpt_set_feature("freehand")
	bridge.sculpt_set_freehand_mode(key)
	app.arm_tool("sculpt")
	rebuild()

## The canvas's "options bar below carries brush size", against the engine's
## own global — the *same* `sculpt_set_globals({"brush_size": …})` the WORLD
## dock's Brush & noise section writes, not a second stored value.
func _build_sculpt_options(row: HBoxContainer) -> void:
	if not bridge.has_world:
		return
	var g: Dictionary = bridge.sculpt_get_globals()
	if g.is_empty():
		return
	## A feature button in the WORLD dock can arm Sculpt on one of the twelve
	## non-Freehand features, which have no sub-mode at all -- the header then
	## names the feature rather than an empty string.
	var head := bridge.sculpt_get_feature()
	if head == "freehand":
		head = bridge.sculpt_get_freehand_mode()
	row.add_child(DccTheme.mono_label("SCULPT · %s" % head.to_upper(),
		"accent", DccTheme.FS_SMALL, 2, true))
	for c in bridge.get_sculpt_globals_info():
		var cd: Dictionary = c
		var key := String(cd.get("key", ""))
		## The canvas's own three -- size, strength, falloff -- under the
		## engine's own names for them (`sculpt_bridge::global_controls`).
		## The other five globals are noise shaping and stay in the WORLD
		## dock's Brush & noise section, which is where §5.2 puts them.
		if not ["brush_size", "intensity", "hardness"].has(key):
			continue
		var is_int := String(cd.get("type", "float")) == "int"
		_fit(DccWidgets.slider(row, String(cd.get("label", key)),
			float(cd.get("min", 0.0)), float(cd.get("max", 1.0)), float(cd.get("step", 0.01)),
			float(g.get(key, cd.get("default", 0.0))), " px" if key == "brush_size" else "",
			func(v: float): bridge.sculpt_set_globals({key: (round(v) if is_int else v)})))
	var params: Dictionary = bridge.sculpt_get_feature_params()
	if params.has("amount"):
		_fit(DccWidgets.slider(row, "Amount", -1.0, 1.0, 0.01, float(params["amount"]), "",
			func(v: float): bridge.sculpt_set_feature_params({"amount": v})))
	row.add_child(DccTheme.spacer())
	var count := bridge.sculpt_stamp_count()
	row.add_child(DccTheme.mono_label("%d stamp%s" % [count, "" if count == 1 else "s"],
		"text_dim", DccTheme.FS_SMALL))
	var commit := DccWidgets.chip(row, "Commit", _on_sculpt_commit, true)
	commit.disabled = count == 0
	var discard := DccWidgets.chip(row, "Discard", _on_sculpt_discard)
	discard.disabled = count == 0

func _on_sculpt_commit() -> void:
	bridge.sculpt_commit("sculpt")
	app.viewport.map_view.texture = bridge.color_texture()
	app.viewport.set_preview_texture(null)
	rebuild()
	if app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

func _on_sculpt_discard() -> void:
	bridge.sculpt_discard()
	app.viewport.set_preview_texture(null)
	rebuild()
	if app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

# -- PAINT ---------------------------------------------------------------------

func _build_paint_tools(row: HBoxContainer) -> void:
	var layers := bridge.get_paint_layers() if bridge.has_world else PackedStringArray()
	if layers.is_empty():
		_note(row, "no paint editor for this world", "A loaded save has no draft session -- the same ceiling Sculpt has.")
		return
	var current := String(_paint_state().get("layer", "biome"))
	for l in layers:
		var key := String(l)
		_tool_segment(row, key.capitalize(), key == current,
			"Paint into the %s override layer." % key, _on_paint_layer.bind(key))
	var st := _paint_state()
	var erase := DccWidgets.segment(row, "Erase", func():
		st["erase"] = not bool(st.get("erase", false))
		_sync_paint()
		rebuild())
	erase.custom_minimum_size.y = 22
	erase.tooltip_text = "Every dab writes 0 (unpainted) regardless of the class. Holding ⇧ while painting does the same without latching this."
	DccWidgets.set_segment_on(erase, bool(st.get("erase", false)))
	_note(row, "painting a classification never changes elevation",
		"The canvas's Water and Lithology are not paintable layers -- neither has an override array for a dab to write into -- and Mask has no engine channel. These three are `PaintTarget`'s own list, read live.")

## The bar's own copy of the brush, mirrored for the same reason
## `world_workspace.gd` mirrors it: `paint_set_brush` applies and echoes back,
## there is no `paint_get_brush`. Kept **on `DccApp`** rather than on this
## object so the WORLD dock's panel and this bar share one dictionary instead
## of two that could drift.
## `register_workspace()` parents every domain panel under `left_dock_body`,
## not under `DccApp`, so this reads the shell's own registry rather than
## walking the tree -- the same leading-underscore, same-layer read
## `journey_planner_view.gd` already documents for `_workspace_panels`.
func _world_workspace() -> Node:
	return app._workspace_panels.get("world")

func _paint_state() -> Dictionary:
	var ws = _world_workspace()
	if ws != null and "_paint_brush" in ws:
		var d: Dictionary = ws._paint_brush.duplicate()
		d["layer"] = ws._paint_layer
		return d
	return {"layer": "biome", "value": 1, "radius": 6.0, "hardness": 1.0, "softness": 0.0, "erase": false, "land_only": true}

func _write_paint_state(key: String, value: Variant) -> void:
	var ws = _world_workspace()
	if ws == null:
		return
	if key == "layer":
		ws._paint_layer = String(value)
	else:
		ws._paint_brush[key] = value

func _sync_paint() -> void:
	var st := _paint_state()
	bridge.paint_set_brush(int(st.get("value", 1)), float(st.get("radius", 6.0)),
		float(st.get("hardness", 1.0)), float(st.get("softness", 0.0)),
		bool(st.get("erase", false)), bool(st.get("land_only", true)))

func _on_paint_layer(key: String) -> void:
	_write_paint_state("layer", key)
	_write_paint_state("value", 1)
	bridge.paint_set_layer(key)
	_sync_paint()
	app.arm_tool("paint")
	rebuild()

func _build_paint_options(row: HBoxContainer) -> void:
	if not bridge.has_world:
		return
	var st := _paint_state()
	var layer := String(st.get("layer", "biome"))
	var palette := bridge.get_paint_palette(layer)
	if palette.is_empty():
		return
	row.add_child(DccTheme.mono_label("PAINT · %s" % layer.to_upper(), "accent", DccTheme.FS_SMALL, 2, true))

	var options: Array = []
	var selected := 0
	for i in palette.size():
		var pd: Dictionary = palette[i]
		options.append(String(pd.get("label", "?")))
		if int(pd.get("index", -1)) == int(st.get("value", 1)):
			selected = i
	_narrow(DccWidgets.choice(row, "Class", options, selected, func(i: int):
		var pd: Dictionary = palette[i]
		_write_paint_state("value", int(pd.get("index", 1)))
		_sync_paint()))
	## The canvas's own "size" row -- the same brush radius the WORLD dock's
	## Biome-paint panel edits, in the engine's own cells.
	_fit(DccWidgets.slider(row, "Size", 1.0, 40.0, 1.0, float(st.get("radius", 6.0)), " c", func(v: float):
		_write_paint_state("radius", v)
		_sync_paint()))
	_fit(DccWidgets.slider(row, "Hardness", 0.0, 1.0, 0.01, float(st.get("hardness", 1.0)), "", func(v: float):
		_write_paint_state("hardness", v)
		_sync_paint(),
		"Stored and echoed back but never consumed -- a dab is a hard disc (paint_bridge.rs)."))
	_narrow(DccWidgets.toggle(row, "Land only", bool(st.get("land_only", true)), func(v: bool):
		_write_paint_state("land_only", v)
		_sync_paint()))
	row.add_child(DccTheme.spacer())
	var total := int(bridge.paint_painted_counts().get("total", 0))
	row.add_child(DccTheme.mono_label("%d painted" % total, "text_dim", DccTheme.FS_SMALL))
	## `GUI_GAP_REGISTER.md` WW-13 -- the pending draft, not the
	## committed-plus-pending composite the readout beside it shows.
	var pending := bridge.paint_draft_count()
	var commit := DccWidgets.chip(row, "Commit", _on_paint_commit, true)
	commit.disabled = pending == 0
	if pending == 0:
		commit.tooltip_text = "Nothing pending. Paint on the map to enable this." if total == 0 \
			else "Nothing pending -- the %d painted cells are already committed." % total

func _on_paint_commit() -> void:
	var summary: Dictionary = bridge.paint_commit()
	## Exactly what `_on_sculpt_commit` above does, and now for the same
	## reason: since 2026-08-24 `build_color_texture()` composites the
	## committed paint layers itself (`landColorCore`'s 0.60 tint), so the
	## raster has to be re-fetched or the commit is invisible -- and the
	## opaque draft overlay has to come off, or it covers the blend it was
	## standing in for with a flat sticker.
	app.viewport.map_view.texture = bridge.color_texture()
	app.viewport.set_preview_texture(null)
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	rebuild()
	## The WORLD dock draws its own Commit / Discard pair over the same draft
	## and is on screen at the same time (WW-13) -- refresh it, or it keeps a
	## live pair over a draft this button just emptied.
	var ws: WorldWorkspace = app._world_workspace()
	if ws != null:
		ws.rebuild_paint_panel()

# -- MEASURE -------------------------------------------------------------------

## `design/Cartalith Measurement Toolbar.dc.html` draws this row as **three
## button groups separated by rules**, identically in all three of its states:
##
##     [Distance Bearing Area Radius] │ CROSS-SECTION [Elevation … Custom▾] │ [Δ vertical  3D distance]
##
## The first revision of this file flattened all six `MEASURE_MODES` into one
## run -- Distance · Bearing · Area · Radius · Cross-section · Δ vertical -- and
## put the channel row behind a `Field` dropdown in the options bar that only
## appeared once Cross-section was already armed. Every button after Radius
## therefore sat at the wrong x, and five of the canvas's own quick-buttons were
## not on the bar at all. The owner reported it as the quick-buttons not being
## where the design puts them; measured live, `Δ vertical` was at x 533 against
## the canvas's third group.
##
## So the groups are explicit here rather than derived from `MEASURE_MODES`'
## declaration order: that const is the *engine's* list of six readings, and
## `_build_measure_options()` still looks up labels and hints through it. The
## canvas's grouping is a presentation fact about this one row.
const MEASURE_GROUP_POINT := ["distance", "bearing", "area", "radius"]
const MEASURE_GROUP_VERTICAL := ["vertical"]

func _measure_mode_dict(id: String) -> Dictionary:
	for m in GlobalTools.MEASURE_MODES:
		if String((m as Dictionary)["id"]) == id:
			return m
	return {}

func _measure_button(row: HBoxContainer, id: String, current: String) -> void:
	var d := _measure_mode_dict(id)
	if d.is_empty():
		return
	var b := _tool_segment(row, String(d["label"]), id == current, String(d["hint"]),
		func(): GlobalTools.set_measure_mode(app, id))
	if bool(d.get("needs_world", false)) and not (bridge.has_world and bridge.measure_api):
		b.disabled = true
		b.tooltip_text = "Reads the height field: generate a world first." if not bridge.has_world \
			else "This build's engine has no measure_section/area/radius/vertical binding."

func _build_measure_tools(row: HBoxContainer) -> void:
	var current := GlobalTools.measure_mode()
	for id in MEASURE_GROUP_POINT:
		_measure_button(row, String(id), current)

	row.add_child(DccTheme.rule(true))
	## The canvas lights this label with the group: dim while another reading is
	## armed, accent while the section is the live one (its state 2 against its
	## states 1 and 3).
	var on_section := current == "section"
	row.add_child(DccTheme.mono_label("CROSS-SECTION",
		"accent" if on_section else "text_dim", DccTheme.FS_SMALL, 2, true))
	## **A channel button is how the canvas arms Cross-section.** Its first group
	## has no Cross-section button -- there are four, and all three states draw
	## exactly four -- so picking a field is the entry point, and picking another
	## while the section is live only swaps what the strip draws. Both halves are
	## one call each and neither re-crosses the engine boundary: `set_measure_mode`
	## re-runs the reading, `set_section_channel` re-draws it.
	var have_section := bridge.has_world and bridge.measure_api
	for c in GlobalTools.SECTION_CHANNELS:
		var cd: Dictionary = c
		var cid := String(cd["id"])
		var b := _tool_segment(row, String(cd["label"]),
			on_section and cid == GlobalTools.section_channel(),
			"Cross-section ▸ %s. Picking a field arms Cross-section if it is not already; the profile is read in the strip under the map." % String(cd["label"]),
			func():
				GlobalTools.set_section_channel(app, cid)
				if GlobalTools.measure_mode() != "section":
					GlobalTools.set_measure_mode(app, "section"))
		if not have_section:
			b.disabled = true
			b.tooltip_text = "Reads the height field: generate a world first." if not bridge.has_world \
				else "This build's engine has no measure_section binding."

	row.add_child(DccTheme.rule(true))
	for id in MEASURE_GROUP_VERTICAL:
		_measure_button(row, String(id), current)

	_note(row, "measurements answer how far · the Sample dock answers what is here",
		"The canvas's own principle: Information is passive and always running; Measure is deliberate and persists until cleared; Cross-section is one line read in the strip below. " +
		"Two of its buttons in this row are not drawn because nothing exists behind them: `Custom ▾` has no user-defined field to bind to (`global_tools.gd`'s SECTION_CHANNELS), and `3D distance` is greyed as \"3D only\" in the canvas itself -- this shell has no 3D view, and Δ vertical already returns the 3D distance in the dock.")

func _build_measure_options(row: HBoxContainer) -> void:
	var mode_id := GlobalTools.measure_mode()
	var label := mode_id
	for m in GlobalTools.MEASURE_MODES:
		if String((m as Dictionary)["id"]) == mode_id:
			label = String((m as Dictionary)["label"])
	row.add_child(DccTheme.mono_label("MEASURE · %s" % label.to_upper(),
		"accent", DccTheme.FS_SMALL, 2, true))
	if mode_id == "section":
		## The channel picker used to be a `Field` dropdown here. It is the
		## canvas's own CROSS-SECTION button group in the row above now
		## (`_build_measure_tools`) -- where the canvas draws it, and visible
		## whether or not the section is already armed. Not duplicated: two
		## controls over one static would only ever be a chance to disagree.
		## `input` stores, `release` re-samples -- the same split every
		## generation control in this shell uses, and for the same reason: a
		## 1 024-sample read per drag tick is a boundary crossing per pixel.
		_fit(DccWidgets.slider(row, "Samples", 32.0, 1024.0, 32.0, float(GlobalTools.section_samples()), "",
			func(v: float): GlobalTools.set_section_samples(app, int(v)),
			"Capped at 1 024 engine-side -- a strip 1 920 px wide cannot show more.",
			func(): GlobalTools.recompute_section(app)))
		_fit(DccWidgets.slider(row, "V. exag", 1.0, SectionStrip.EXAG_MAX, 1.0,
			float(GlobalTools.section_exaggeration()), "×",
			func(v: float): GlobalTools.set_section_exaggeration(app, v),
			"Same horizontal scale, more vertical pixels per metre -- so the strip itself grows, capped at 42 % of the viewport. The value window always fits the whole profile; nothing clips."))
		## The canvas says "drag the line ends to re-sample". Line-end drag
		## handles are **not** built: a third click starts a new section
		## instead, which is the same two clicks and no new hit-testing.
		_bar_hint(row, "click A then B · a third click starts a new section · scrub the profile to track the map",
			"The canvas's draggable A/B handles are not built -- re-clicking is the same two clicks and needs no on-canvas hit test of its own.")
	elif mode_id == "area":
		_bar_hint(row, "the ring closes itself from three points on · water inside it is subtracted",
			"Projected area is the exact shoelace; the water split, the true surface and the mean elevation are sampled from the cells inside the ring.")
	elif mode_id == "radius":
		_bar_hint(row, "click the centre, then a point on the rim", "")
	elif mode_id == "vertical":
		_bar_hint(row, "two points · rise, run, 3D distance and grade", "")
	elif mode_id == "bearing":
		_bar_hint(row, "two points · the first is the observer", "")
	else:
		_bar_hint(row, "click to add a point · ⌫ drops the last · Esc clears", "")
	## Four of the canvas's own Distance options are not built, and each is a
	## real absence rather than an omission -- stated on hover instead of drawn
	## as a dead control. Registered as GUI gaps, not hidden.
	if mode_id == "distance" or mode_id == "bearing":
		_bar_hint(row, "— 4 canvas options unbuilt",
			"multi-segment / point-to-point: the six mode buttons above are that choice. " +
			"path ▸ great circle: this map is equirectangular and `cartalith_spatial::measure` is planar with a seam rule; there is no spherical path to offer. " +
			"snap ▸ settlements/rivers: DCC_SHELL_SPEC.md §4.5.1 lists no snap modifier for Measure, unlike Way/Route. " +
			"units ▸ km: the canvas itself says this inherits the app-wide unit switch (the reference's `_setUnits`, line 13722); no such preference exists in this shell yet, so every reading is km.")
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label(GlobalTools.measure_status_text(), "text_dim", DccTheme.FS_SMALL))
	DccWidgets.chip(row, "Clear", func(): GlobalTools.measure_reset(app))
