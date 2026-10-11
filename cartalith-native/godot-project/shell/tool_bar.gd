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
## **Since 2026-09-23 there is a second bar directly above this one** -- the
## tool palette (`DccApp._install_tool_palette_bar()`, owner's request to move
## each dock's TOOLS block to the top). It is inserted by `app.gd`, not by
## `dcc_shell.gd`'s compositions, and is off on a phone. Its Measure cell and
## this bar's MEASURE mode arm the same `"measure"` id, as do its WORLD Biome
## paint cell and PAINT here -- the same pairing the dock's TOOLS block had with
## this bar before, moved, not added.
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
## ## A third row, since ET-D5
##
## Paint and Sculpt gain a **brush row** under the canvas's two (`build_brush_controls`,
## near the end of this file's ET-D5 section): spacing, pressure, scatter and symmetry,
## drawn in a wrapping flow so it can never widen the window. The canvas draws spacing /
## pressure / symmetry inline in its one options row; this bar's row already carries
## class, size, land-only, Original and Commit/Discard, so they go below instead -- a
## departure from the canvas for width's sake, not a design decision. The Way tool's own
## row (`infrastructure_workspace.gd`) calls the same builder.
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
	## ET-D5: a pending "click the map to place the symmetry centre" belongs to the tool
	## it was asked on; arming ANY tool drops it (the Way tool is not one of `MODES`, so
	## this runs before the early return below), and the on-map marker follows the newly
	## armed tool's own symmetry (or clears for a tool that has none).
	sym_pick_tool = ""
	refresh_marker(app)
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

## **Two ruled 38 px rows, not two unruled ones inside a one-row bar.**
##
## `GUI_GAP_REGISTER.md` §51 row 61 recorded this as an owner call, because the
## two canvases disagreed: `DCC shell 1920` drew the tool options bar as **one**
## `height:34px` row, and `Cartalith Paint Toolbar.dc.html` draws **two**, each
## `height:38px;padding:0 14px;border-bottom:1px solid rgba(255,255,255,.10)`.
## `DCC_SHELL_SCOPE.md`'s rule 1 settled it: the Paint Toolbar is the later
## artboard and draws a component the other does not, so it wins. This built the
## two rows already -- at 22 and 24-28 px, with a 4 px gap and no rule.
##
## **Re-read against the governing canvas 2026-09-07, and two of the three
## figures above are stale.** `Cartalith DCC Environment.dc.html` -- the canvas
## the owner's 2026-08-25 ruling makes the newer authority, and the one every
## `ROLE` row is homed to -- draws this bar at `ENV:149` as
## `height:var(--tbH);gap:10px;padding:0 var(--pad);border-bottom:1px solid
## var(--hair)`, and `--tbH` is **40** (`ENV:25`) / **56** (`ENV:1819`). So:
##
##   - The **34** in the paragraph above is a retired artboard's number twice
##     over. `ROLE.h_tool_options` has read `[40, 56]` off `ENV` since
##     2026-08-31, and `dcc_shell.gd:1714` sets the bar's minimum from it -- so
##     the sentence that used to end this comment, "the shell's bar keeps its
##     own 34 px minimum", described nothing in the tree. Removed rather than
##     re-worded: `role_px("h_tool_options")` is the figure, and quoting it
##     here would be a second source of truth for it.
##   - The **one-row/two-row disagreement is now three-way**, and the newest
##     party is back on one row. The two rows stay because they are an owner
##     call and an owner decision outranks a canvas -- but the ruling was made
##     against `DCC shell 1920`, not against this canvas, so it is available to
##     be re-taken and that is a decision for the owner, not for this file.
##
## `BAR_ROW_H` itself is unchanged and still the Paint Toolbar's own 38.
const BAR_ROW_H := 38

func _build(row: HBoxContainer) -> void:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(col)

	var tools := HBoxContainer.new()
	tools.add_theme_constant_override("separation", 6)
	tools.custom_minimum_size.y = BAR_ROW_H
	col.add_child(tools)
	_build_mode_buttons(tools)
	tools.add_child(DccTheme.rule(true))
	match mode:
		"sculpt": _build_sculpt_tools(tools)
		"paint": _build_paint_tools(tools)
		_: _build_measure_tools(tools)

	## The canvas's `border-bottom` on the first row. Inset by the bar's own
	## 14 px content margin rather than full-bleed -- `tool_options_row` is the
	## one node `set_tool_options()` refills and it sits *inside* that margin, so
	## a full-width rule would mean restructuring the bar around a component that
	## four workspaces and `app.gd` all build into. Named because it is the one
	## figure in this block that is not the canvas's.
	col.add_child(DccTheme.rule())

	var options := HBoxContainer.new()
	options.add_theme_constant_override("separation", 8)
	options.custom_minimum_size.y = BAR_ROW_H
	col.add_child(options)
	match mode:
		"sculpt": _build_sculpt_options(options)
		"paint": _build_paint_options(options)
		_: _build_measure_options(options)

	## ET-D5: the brush row. A THIRD row under the canvas's two, only for the two modes
	## that draw with a brush and only when their own options row drew anything (no world,
	## or no editor for the world, leaves `options` empty and the brush has nothing to
	## apply to). A `HFlowContainer`, not an `HBoxContainer`: the canvas draws
	## spacing / pressure / symmetry inline in its one options row, but this bar's row
	## already carries class, size, land-only, Original and Commit/Discard, and an extra
	## ~1 200 px of controls in a plain box would raise the whole window's minimum width
	## (the hazard `_note()` documents). A flow wraps to a second line instead, so the
	## window keeps its width and the bar only grows taller on a narrow one.
	if (mode == "paint" or mode == "sculpt") and options.get_child_count() > 0:
		col.add_child(DccTheme.rule())
		var brush := HFlowContainer.new()
		brush.add_theme_constant_override("h_separation", 8)
		brush.add_theme_constant_override("v_separation", 0)
		brush.custom_minimum_size.y = BAR_ROW_H
		brush.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		col.add_child(brush)
		build_brush_controls(brush, app, mode)

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
## (`_fit` and `_narrow` below are `static` since ET-D5 so the brush controls, which
## the Way row also builds, share them; neither touches instance state.)
const BAR_LABEL_W := 74
const BAR_VALUE_W := 38
## `width:96px` / `78px` / `70px` on the three tracks in `Cartalith Paint
## Toolbar.dc.html`'s `Sculpt raise 1920`, and `width:70px` in the DCC shell's
## own tool options bar. 56 was a bar-specific narrowing of the dock's 78 that
## no artboard asks for; 70 is what the shell canvas draws, and the widest the
## bar can afford with four parameters and a commit pair on a 1600 px window.
const BAR_CONTROL_W := 70

static func _fit(parts: Dictionary) -> void:
	_narrow(parts["row"])
	if parts.has("readout"):
		(parts["readout"] as Control).custom_minimum_size.x = BAR_VALUE_W
	if parts.has("slider"):
		(parts["slider"] as Control).custom_minimum_size.x = BAR_CONTROL_W

## For `choice`/`toggle`, which return the control rather than its parts.
static func _narrow(ctrl: Control) -> Control:
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
		_note(row, "no sculpt editor for this world", "A world opened from a save without its hydrology and tectonic rasters (a legacy .zip, or a project saved before 2026-09-24) has no draft session.")
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
	## `world_workspace.gd::_on_sculpt_commit`'s third line, same reason: a
	## live LOD tile keeps its own captured `base_tex` from the OLD texture
	## (`OUTSTANDING_WORK.md`, "the in-session tile cache is not invalidated
	## by a sculpt").
	app.viewport.invalidate_lod_tiles()
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
		_note(row, "no paint editor for this world", "A world opened from a save without its hydrology and tectonic rasters (a legacy .zip, or a project saved before 2026-09-24) has no draft session -- the same ceiling Sculpt has.")
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
	## `Hardness` (and its `Softness` sibling, which never had a copy here)
	## lives in the WORLD dock's Biome paint panel only now -- `UNWIRED_
	## FUNCTIONS.md`'s "two copies on screen at once" row, ruled by the owner
	## 2026-08-31 (`LARGE_ITEM_RULINGS.md`): bind the falloff (`DECISIONS.md`
	## §7k) *and* resolve the duplicate. The dock panel owns the actual
	## `_paint_brush` state (this bar only mirrors it through `_paint_state`/
	## `_write_paint_state`) and is the one place both sliders already sat
	## side by side, so its copy survives and this bar's is deleted, not
	## hidden -- matching Sculpt's own tool-bar row, which draws a narrowed
	## subset of the dock's full parameter set rather than all of it.
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
	## `05-right-dock-and-bars.md` §2.2.6 (`tbBiome`, ENV:200-211) gives this row
	## **two** buttons -- `✓ Commit` primary and `Discard` secondary -- and this
	## bar drew only the first until 2026-09-05, so the toolbar the design draws
	## the pair on had half of it. Both the WORLD dock's Biome paint panel and
	## the right dock's own paint context already draw a Commit/Discard pair over
	## this same draft, and the engine call is `EngineBridge.paint_discard()`,
	## which both of them already use -- nothing new is bound here.
	##
	## `accent` false, matching Sculpt's own bar row above: the primary of the
	## pair is lit and the secondary is not, which is the design's
	## primary/secondary split in the vocabulary this bar already speaks.
	var discard := DccWidgets.chip(row, "Discard", _on_paint_discard)
	discard.disabled = pending == 0
	if pending == 0:
		var why := "Nothing pending. Paint on the map to enable this." if total == 0 \
			else "Nothing pending -- the %d painted cells are already committed." % total
		commit.tooltip_text = why
		discard.tooltip_text = why

func _on_paint_commit() -> void:
	var summary: Dictionary = bridge.paint_commit()
	## Exactly what `_on_sculpt_commit` above does, and now for the same
	## reason: since 2026-08-24 `build_color_texture()` composites the
	## committed paint layers itself (`landColorCore`'s 0.60 tint), so the
	## raster has to be re-fetched or the commit is invisible -- and the
	## opaque draft overlay has to come off, or it covers the blend it was
	## standing in for with a flat sticker.
	app.viewport.map_view.texture = bridge.color_texture()
	## Same third line as `_on_sculpt_commit` above, same reason: a live LOD
	## tile keeps its own captured `base_tex` from the OLD texture.
	app.viewport.invalidate_lod_tiles()
	app.viewport.set_preview_texture(null)
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	rebuild()
	## **All three surfaces over the one draft**, matching `_on_paint_discard`
	## below exactly: this bar (`rebuild()` above), the WORLD dock's Biome paint
	## panel (WW-13) and the right dock's paint context. The last of those was
	## missing until 2026-09-05 and its own defect was measured before it was
	## fixed (`_rdrefresh_probe.gd` R2): committing here left the right dock
	## reporting "1 dab pending" with a live Commit over an emptied draft, and
	## rebuilt that dock **zero** times.
	var ws: WorldWorkspace = app._world_workspace()
	if ws != null:
		ws.rebuild_paint_panel()
		ws._refresh_right_dock_paint()

## §2.2.6's second button. The sequence is `world_workspace.gd`'s own
## `_on_paint_discard()` -- the one that already owns this draft -- rather than
## a second reading of what a discard needs:
##
## - `paint_discard()` drops every pending dab across every layer, which is what
##   `paint_draft_count()` above counts.
## - The preview texture is re-set with `true`, not cleared. `_on_paint_commit`
##   clears it because a commit folds the paint into `color_texture()`; a
##   discard leaves the previously *committed* layer standing, and that layer is
##   the base the next dab's bounded window blits onto. Passing `null` here
##   would put the first dab after a discard back on the unbounded path.
## - All three surfaces over the one draft are refreshed: this bar, the WORLD
##   dock's Biome paint panel (WW-13) and the right dock's paint context.
##
## **`_on_paint_commit` above refreshed the first two and not the third**, which
## this comment reported as a separate finding rather than fixing in passing.
## It was filed, probed and fixed on 2026-09-05, so the two handlers now match
## line for line and the sentence above describes both.
func _on_paint_discard() -> void:
	bridge.paint_discard()
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture(), true)
	rebuild()
	var ws: WorldWorkspace = app._world_workspace()
	if ws != null:
		ws.rebuild_paint_panel()
		ws._refresh_right_dock_paint()

# -- ET-D5: the brush controls --------------------------------------------------
#
# `EDITING_TOOLS_SCOPE.md` ET-D5. The stroke engine (ET-D1..D4) has been taking
# options since it landed -- spacing, pressure, symmetry, scatter for Paint;
# symmetry and pressure for Sculpt; mode, smoothing, corridor, pull and symmetry for
# the Way drag -- but the shell only ever sent defaults. This section is the
# controls, and the rules they obey:
#
#  * **The tool's own seam is the single source of truth.** Every control READS its
#    value from, and WRITES its value to, the options Dictionary the tool already
#    hands the engine on each press (`world_workspace.gd::_paint_stroke_options`,
#    `::_sculpt_symmetry`, `infrastructure_workspace.gd::_way_drag_options`), reached
#    through each workspace's `brush_options(tool)` accessor. No control keeps a copy,
#    so a surface that is rebuilt, a second surface over the same state (the phone's
#    Paint/Sculpt columns are the second) and a probe that writes the seam directly
#    all agree by construction -- "two views of one state will disagree" was a
#    recorded mistake and this is how it is not repeated.
#  * **A default is the ABSENT key**, never a written default: an untouched tool still
#    hands the engine exactly `{}` (Paint, Sculpt) or `{}` merged over `{way_type,
#    spacing}` (Way), which is today's behaviour by control flow. A control moved back
#    to its default erases its key again.
#  * **No enabled-and-inert control.** A control the engine would ignore in the current
#    state (the angle of a radial symmetry, the corridor of a freehand road, the spread
#    of one scatter copy) is hidden or drawn disabled, never live.
#  * Pressure's "target" is a fact about the tool, not a choice: the engine offers Paint
#    `pressure_size` only (flow is meaningless for a categorical layer, `DECISIONS.md`
#    §7k) and Sculpt's pressure scales the stamp's strength only, so the toggle states
#    its target as text beside it instead of offering a one-item dropdown. There is no
#    softness target because the engine has no option for one.
#  * Nothing here calls the engine or the disk: writes are shell-side dictionary edits,
#    so none can run into a generation worker that owns `world_gen`.
#
# The three surfaces: the bar's third row (`build_brush_controls`, desktop and tablet),
# `world_workspace.gd`'s phone Paint and Sculpt columns (`_pg_brush_block`, which calls
# the writers below) and the Way row (`infrastructure_workspace.gd`, which calls
# `build_brush_controls` too).

## The tools that take brush options -- the ids `options()` and `refresh_marker()` accept.
const BRUSH_TOOLS := ["paint", "sculpt", "way"]
## The engine's symmetry names, in the order the control lists them (`lib.rs`'s
## `sculpt_begin_stroke_with` table), and the words the user reads.
const SYM_MODES := ["none", "vertical", "horizontal", "dual", "radial"]
const SYM_LABELS := ["Off", "Vertical", "Horizontal", "Dual", "Radial"]
## Radial copy count: the engine's own default (4) and its refusal bound (1..64) are
## `lib.rs`'s; 2..16 is a labelled JUDGEMENT for the control -- 1 copy is "off" and above
## 16 spokes no longer read as symmetry on a plate this size.
const RADIAL_DEFAULT := 4
const RADIAL_MIN := 2
const RADIAL_MAX := 16
## Mirror-axis rotation range, degrees (judgement): a mirror axis repeats every 180 deg,
## so -90..90 reaches every distinct axis once.
const ANGLE_MAX_DEG := 90.0
## Paint spacing, percent of the dab diameter. 18 is the Paint Toolbar artboard's own
## drawn value and the engine default (`ET-D1`); 1 and 100 are control bounds (the engine
## accepts 0.01..10), a labelled judgement.
const SPACING_PCT_DEFAULT := 18.0
const SPACING_PCT_MIN := 1.0
const SPACING_PCT_MAX := 100.0
## Scatter bounds. Count 1..16 is the engine's own cap (`ET-D1`, PhotoCraft's), spread in
## dab diameters 0..3 is a JUDGEMENT (the engine accepts 0..10; beyond ~3 the copies stop
## reading as one dab).
const SCATTER_COUNT_MAX := 16
const SCATTER_AMOUNT_MAX := 3.0
## Way drag: the two modes and their words; defaults are the engine's (`lib.rs`
## `way_stroke_begin` table): routed, smooth on, corridor 3.0 cells, pull 2.0. The control
## ranges are JUDGEMENTS inside the engine's (corridor > 0 and <= 64, pull 0..100).
const WAY_MODES := ["routed", "freehand"]
const WAY_MODE_LABELS := ["Routed", "Freehand"]
## Waypoint spacing of a routed drag, in grid cells: 2.0 is `infrastructure_workspace.gd`'s own
## `WAY_DRAG_SPACING_CELLS` (a labelled judgement there) and MUST equal it, because an untouched
## control erases its key and the shell's own default then applies. 1..8 is a JUDGEMENT for the
## control: below 1 the leg count explodes, and at 12 a short drag no longer makes the engine's
## minimum of waypoints (measured: nothing committed on a 24-cell drag).
const WAY_SPACING_DEFAULT := 2.0
const WAY_SPACING_MIN := 1.0
const WAY_SPACING_MAX := 8.0
const WAY_CORRIDOR_DEFAULT := 3.0
const WAY_CORRIDOR_MIN := 1.0
const WAY_CORRIDOR_MAX := 16.0
const WAY_PULL_DEFAULT := 2.0
const WAY_PULL_MAX := 20.0

## The tool whose NEXT map press sets the symmetry centre instead of drawing, or `""`.
## Process-wide and never persisted; cleared by arming any tool (`_on_tool_armed`) and
## by `sym_pick_try`. Static because three files' click handlers read it.
static var sym_pick_tool := ""
## Whether the Way row's corridor/pull block is open (UI-only, never persisted).
static var way_advanced_open := false

## The workspace that owns `tool`'s seam, or `null` (before the workspaces are built).
static func seam_owner(app: DccApp, tool: String) -> Object:
	if app == null:
		return null
	match tool:
		"paint", "sculpt":
			return app._workspace_panels.get("world")
		"way":
			var civ = app._workspace_panels.get("civilization")
			if civ != null and "_infra" in civ:
				return civ._infra
	return null

## `tool`'s live options Dictionary -- the very object the tool hands the engine, so a
## write to it IS the control taking effect. `{}` (a throwaway) when the owner is not
## there, so a caller never has to null-check and writes to it are simply lost.
static func options(app: DccApp, tool: String) -> Dictionary:
	var owner := seam_owner(app, tool)
	if owner != null and owner.has_method("brush_options"):
		return owner.call("brush_options", tool)
	return {}

## Re-draws every surface that shows `tool`'s brush state: the on-map marker, then the
## tool's own surfaces through its `brush_surfaces_changed()` (the bar and the phone
## column for Paint/Sculpt, the Way row for Way). Called after every DISCRETE change; a
## slider calls it only on release, never rebuilding a control under the finger.
static func refresh_surfaces(app: DccApp, tool: String) -> void:
	refresh_marker(app)
	var owner := seam_owner(app, tool)
	if owner != null and owner.has_method("brush_surfaces_changed"):
		owner.call("brush_surfaces_changed")

## Points the on-map symmetry marker at the ARMED tool's symmetry (or clears it for a
## tool with none). Cheap; safe to call while the engine is busy (it reads shell
## dictionaries only).
static func refresh_marker(app: DccApp) -> void:
	if app == null or app.viewport == null or app.viewport.tool_overlay == null:
		return
	var ov: ToolOverlay = app.viewport.tool_overlay
	var tool := String(app.armed_tool)
	if not BRUSH_TOOLS.has(tool):
		ov.set_symmetry("none", RADIAL_DEFAULT, null, 0.0)
		return
	var o := options(app, tool)
	ov.set_symmetry(sym_mode(o), sym_count(o), sym_centre(o), float(o.get("symmetry_angle", 0.0)))

# -- Readers and writers over an options Dictionary (pure; no node, no engine) ----

## The seam's symmetry name; an absent or unknown value reads as `"none"`.
static func sym_mode(o: Dictionary) -> String:
	var m := String(o.get("symmetry", "none"))
	return m if SYM_MODES.has(m) else "none"

static func sym_count(o: Dictionary) -> int:
	return int(o.get("radial_count", RADIAL_DEFAULT))

## The seam's mirror-axis angle in degrees (the seam stores radians, the engine's unit).
static func sym_angle_deg(o: Dictionary) -> float:
	return rad_to_deg(float(o.get("symmetry_angle", 0.0)))

## The seam's symmetry centre as a `Vector2` (grid cells), or `null` when absent (= the
## engine's lattice centre). `null`, never a zero vector: (0, 0) is a real corner cell.
static func sym_centre(o: Dictionary) -> Variant:
	var c = o.get("symmetry_center")
	return c if c is Vector2 else null

## Sets the symmetry mode. `"none"` erases EVERY symmetry key (mode, count, centre, angle), so
## Off is exactly the untouched seam -- `{}` for the tools whose seam holds nothing else -- which
## is today's behaviour by construction rather than by the engine ignoring leftovers (the cost: a
## centre picked earlier is gone when the user turns symmetry off). Switching between the four
## ON modes keeps the centre and angle. `"radial"` writes an explicit integer `radial_count` so
## the readback is self-evident and the engine gets the int it requires. An unknown name is ignored.
static func sym_write_mode(o: Dictionary, mode: String) -> void:
	if not SYM_MODES.has(mode):
		return
	if mode == "none":
		for k in ["symmetry", "radial_count", "symmetry_center", "symmetry_angle"]:
			o.erase(k)
		return
	o["symmetry"] = mode
	if mode == "radial" and not o.has("radial_count"):
		o["radial_count"] = RADIAL_DEFAULT

static func sym_write_count(o: Dictionary, n: float) -> void:
	o["radial_count"] = clampi(int(round(n)), RADIAL_MIN, RADIAL_MAX)

## Writes the mirror angle from degrees; 0 erases the key. Stored in radians, as the engine
## reads it.
static func sym_write_angle_deg(o: Dictionary, deg: float) -> void:
	var d := clampf(deg, -ANGLE_MAX_DEG, ANGLE_MAX_DEG)
	if is_zero_approx(d):
		o.erase("symmetry_angle")
	else:
		o["symmetry_angle"] = deg_to_rad(d)

## Writes the centre (a `Vector2` in grid cells) or, for `null`, erases it (back to the
## engine's lattice centre).
static func sym_write_centre(o: Dictionary, c: Variant) -> void:
	if c is Vector2:
		o["symmetry_center"] = c
	else:
		o.erase("symmetry_center")

## Paint spacing. `spacing_enabled` absent/true = spacing ON at `spacing` (default 18 %);
## false = OFF, the one-dab-per-sample picture. "Off" is its own flag, never a 0 %.
static func paint_spacing_on(o: Dictionary) -> bool:
	return bool(o.get("spacing_enabled", true))

static func paint_spacing_pct(o: Dictionary) -> float:
	return float(o.get("spacing", SPACING_PCT_DEFAULT / 100.0)) * 100.0

static func paint_write_spacing_on(o: Dictionary, on: bool) -> void:
	if on:
		o.erase("spacing_enabled")
	else:
		o["spacing_enabled"] = false

static func paint_write_spacing_pct(o: Dictionary, pct: float) -> void:
	var p := clampf(pct, SPACING_PCT_MIN, SPACING_PCT_MAX)
	if is_equal_approx(p, SPACING_PCT_DEFAULT):
		o.erase("spacing")
	else:
		o["spacing"] = p / 100.0

## Paint pressure-to-size (the engine's `pressure_size`, default true).
static func paint_pressure_on(o: Dictionary) -> bool:
	return bool(o.get("pressure_size", true))

static func paint_write_pressure(o: Dictionary, on: bool) -> void:
	if on:
		o.erase("pressure_size")
	else:
		o["pressure_size"] = false

## Paint scatter: copies per dab (1 = off) and the offset range in dab diameters.
static func paint_scatter_count(o: Dictionary) -> int:
	return int(o.get("scatter_count", 1))

static func paint_scatter_amount(o: Dictionary) -> float:
	return float(o.get("scatter_amount", 0.0))

static func paint_write_scatter_count(o: Dictionary, n: float) -> void:
	var c := clampi(int(round(n)), 1, SCATTER_COUNT_MAX)
	if c == 1:
		o.erase("scatter_count")
	else:
		o["scatter_count"] = c

static func paint_write_scatter_amount(o: Dictionary, a: float) -> void:
	var v := clampf(a, 0.0, SCATTER_AMOUNT_MAX)
	if is_zero_approx(v):
		o.erase("scatter_amount")
	else:
		o["scatter_amount"] = v

## Way drag mode / smoothing / guide steering. Defaults are the engine's.
static func way_mode(o: Dictionary) -> String:
	var m := String(o.get("mode", "routed"))
	return m if WAY_MODES.has(m) else "routed"

static func way_write_mode(o: Dictionary, mode: String) -> void:
	if not WAY_MODES.has(mode):
		return
	if mode == "routed":
		o.erase("mode")
	else:
		o["mode"] = mode

static func way_smooth_on(o: Dictionary) -> bool:
	return bool(o.get("smooth", true))

static func way_write_smooth(o: Dictionary, on: bool) -> void:
	if on:
		o.erase("smooth")
	else:
		o["smooth"] = false

static func way_spacing(o: Dictionary) -> float:
	return float(o.get("spacing", WAY_SPACING_DEFAULT))

static func way_write_spacing(o: Dictionary, v: float) -> void:
	var c := clampf(v, WAY_SPACING_MIN, WAY_SPACING_MAX)
	if is_equal_approx(c, WAY_SPACING_DEFAULT):
		o.erase("spacing")
	else:
		o["spacing"] = c

static func way_corridor(o: Dictionary) -> float:
	return float(o.get("corridor", WAY_CORRIDOR_DEFAULT))

static func way_pull(o: Dictionary) -> float:
	return float(o.get("pull", WAY_PULL_DEFAULT))

static func way_write_corridor(o: Dictionary, v: float) -> void:
	var c := clampf(v, WAY_CORRIDOR_MIN, WAY_CORRIDOR_MAX)
	if is_equal_approx(c, WAY_CORRIDOR_DEFAULT):
		o.erase("corridor")
	else:
		o["corridor"] = c

static func way_write_pull(o: Dictionary, v: float) -> void:
	var p := clampf(v, 0.0, WAY_PULL_MAX)
	if is_equal_approx(p, WAY_PULL_DEFAULT):
		o.erase("pull")
	else:
		o["pull"] = p

# -- Picking the symmetry centre on the map ----------------------------------------

## Starts (or, pressed again, cancels) "the next map press on `tool` sets the centre".
## Starting ARMS the tool first (a press only reaches a tool's handler while it is armed,
## and arming any tool drops a pending pick, `_on_tool_armed`, so the pick is set AFTER).
## The map is hidden behind the phone sheet, so the phone surfaces drop it to `peek` after
## calling this (`world_workspace.gd::_pg_brush_block`); this function stays surface-neutral.
static func sym_pick_begin(app: DccApp, tool: String) -> void:
	if sym_pick_tool == tool:
		sym_pick_tool = ""
		app.set_status("hint", "", "text_ghost")
	else:
		app.arm_tool(tool)
		sym_pick_tool = tool
		app.set_status("hint", "Press the map to place the symmetry centre -- the next press only; it draws nothing.", "accent")
	refresh_surfaces(app, tool)

## The click hook every brush tool's PRESS handler calls first: if a centre pick is pending
## for `tool`, consumes the press, stores the centre (clamped onto the grid) in the seam,
## clears the pick and returns `true` -- the caller must then draw NOTHING for this gesture
## (and swallow its drag/release). Returns `false` and changes nothing otherwise, so a tool
## with no pick pending behaves exactly as before. Never calls the engine.
static func sym_pick_try(app: DccApp, tool: String, gx: float, gy: float) -> bool:
	if sym_pick_tool == "" or sym_pick_tool != tool:
		return false
	sym_pick_tool = ""
	var cells := Vector2(float(gx), float(gy))
	if app != null and app.viewport != null and app.viewport.tool_overlay != null:
		var g: Vector2i = (app.viewport.tool_overlay as ToolOverlay).grid_cells()
		if g.x > 0 and g.y > 0:
			cells = Vector2(clampf(cells.x, 0.0, float(g.x - 1)), clampf(cells.y, 0.0, float(g.y - 1)))
	sym_write_centre(options(app, tool), cells)
	app.set_status("hint", "Symmetry centre set to (%.1f, %.1f)." % [cells.x, cells.y], "text_ghost")
	refresh_surfaces(app, tool)
	return true

# -- The controls (desktop / tablet bar, and the Way row) ---------------------------

## One-line explanation beside a control, mono micro text. NOT `clip_text` (see `_bar_hint`'s
## sibling hazard in `MISTAKES.md`: a clipped Label collapses to 1 px beside an expanding
## sibling); the flow rows these sit in have no expanding sibling.
static func _hint(parent: Control, text: String, detail: String) -> void:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.tooltip_text = detail
	l.mouse_filter = Control.MOUSE_FILTER_STOP
	parent.add_child(l)

## The touch floor, in authored px, on BOTH axes (`MISTAKES.md`: "floor at the call site on both
## axes" -- `tablet_fit()` floors height only and runs once, and the hazard was a 36 px square and
## a 7 px chevron where every HEIGHT was 44). 44 is the project's floor (`DccTheme.PHONE_TAP_MIN`,
## `role_px("btn_min_h")`). Applied to every NEW control of this section at its call site, on
## tablet and phone only: a pointer density keeps the bar's compact 24-26 px rows, and the
## existing bar rows (Size, Land only ...) are deliberately not touched.
const TOUCH_FLOOR := 44.0

## Raises `target` (a `Control`, or a `DccWidgets.slider()` parts Dictionary, whose slider is
## raised) to `TOUCH_FLOOR` on both axes on a touch density; a no-op for a pointer. Returns
## `target` so a factory call can be wrapped in it.
static func _touch_floor(target: Variant) -> Variant:
	if not (DccTheme.is_tablet() or DccTheme.is_phone()):
		return target
	var c: Control = (target["slider"] as Control) if target is Dictionary else (target as Control)
	if c != null:
		c.custom_minimum_size = Vector2(maxf(c.custom_minimum_size.x, TOUCH_FLOOR),
			maxf(c.custom_minimum_size.y, TOUCH_FLOOR))
	return target

## Draws `tool`'s brush controls into `parent` (an `HFlowContainer`): Paint -- spacing,
## pressure, scatter, symmetry; Sculpt -- pressure, symmetry; Way -- mode, smooth, the
## advanced corridor/pull, symmetry. Each control reads its seam on the way in and writes it
## on the way out through the writers above; the callbacks re-fetch the seam on EVERY
## call (`options()`), so a seam replaced after the row was built is the one written.
## Must never call the engine, and must never draw a control the engine would ignore.
static func build_brush_controls(parent: Control, app: DccApp, tool: String) -> void:
	var lead := "ROAD DRAG" if tool == "way" else "BRUSH"
	parent.add_child(DccTheme.mono_label(lead, "accent", DccTheme.FS_SMALL, 2, true))
	match tool:
		"paint":
			_controls_paint(parent, app)
		"sculpt":
			_controls_sculpt(parent, app)
		"way":
			_controls_way(parent, app)
	_controls_symmetry(parent, app, tool)

static func _controls_paint(parent: Control, app: DccApp) -> void:
	var o := options(app, "paint")
	var spacing_on := paint_spacing_on(o)
	_touch_floor(_narrow(DccWidgets.toggle(parent, "Spacing", spacing_on, func(v: bool):
		paint_write_spacing_on(options(app, "paint"), v)
		refresh_surfaces(app, "paint"),
		"Dabs are placed along the drag every N % of the brush diameter, so a fast drag stays connected. Off = one dab per pointer sample (gaps on a fast drag). Default 18 %, the Paint Toolbar's own value.")))
	var gap := DccWidgets.slider(parent, "Gap", SPACING_PCT_MIN, SPACING_PCT_MAX, 1.0,
		paint_spacing_pct(o), " %",
		func(v: float): paint_write_spacing_pct(options(app, "paint"), v),
		"Distance between dabs, percent of the brush diameter.")
	_fit(gap)
	_touch_floor(gap)
	if not spacing_on:
		_disable_slider(gap, "Spacing is off: one dab per pointer sample.")
	_touch_floor(_narrow(DccWidgets.toggle(parent, "Pressure", paint_pressure_on(o), func(v: bool):
		paint_write_pressure(options(app, "paint"), v)
		refresh_surfaces(app, "paint"),
		"A pen's pressure scales the dab SIZE (never the class: paint is categorical). A mouse reports no pressure, so this changes nothing for one. There is no softness target: the engine has no option for it.")))
	_hint(parent, "→ size", "Pressure drives the dab size.")
	var count := paint_scatter_count(o)
	var scatter := DccWidgets.slider(parent, "Scatter", 1.0, float(SCATTER_COUNT_MAX), 1.0, float(count), "",
		func(v: float): paint_write_scatter_count(options(app, "paint"), v),
		"Copies per dab, each offset across the drag. 1 = off.",
		func(): refresh_surfaces(app, "paint"))
	_fit(scatter)
	_touch_floor(scatter)
	var spread := DccWidgets.slider(parent, "Spread", 0.0, SCATTER_AMOUNT_MAX, 0.1,
		paint_scatter_amount(o), " ⌀",
		func(v: float): paint_write_scatter_amount(options(app, "paint"), v),
		"How far a scatter copy may land from the stroke, in brush diameters.")
	_fit(spread)
	_touch_floor(spread)
	if count <= 1:
		_disable_slider(spread, "Needs more than one scatter copy.")

static func _controls_sculpt(parent: Control, app: DccApp) -> void:
	var owner := seam_owner(app, "sculpt")
	var on := true
	if owner != null and owner.has_method("sculpt_pressure_on"):
		on = bool(owner.call("sculpt_pressure_on"))
	_touch_floor(_narrow(DccWidgets.toggle(parent, "Pressure", on, func(v: bool):
		var ws := seam_owner(app, "sculpt")
		if ws != null and ws.has_method("set_sculpt_pressure_on"):
			ws.call("set_sculpt_pressure_on", v)
		refresh_surfaces(app, "sculpt"),
		"A pen's pressure scales each stamp's STRENGTH along the stroke. A mouse reports no pressure, so this changes nothing for one.")))
	_hint(parent, "→ strength", "Pressure drives the stamp strength.")

static func _controls_way(parent: Control, app: DccApp) -> void:
	var o := options(app, "way")
	var mode := way_mode(o)
	_touch_floor(_narrow(DccWidgets.choice(parent, "Mode", WAY_MODE_LABELS, WAY_MODES.find(mode), func(i: int):
		way_write_mode(options(app, "way"), String(WAY_MODES[i]))
		refresh_surfaces(app, "way"),
		"Routed: every drawn leg is routed by least cost (the default, as the click path). Freehand: the road is exactly the cells you draw, unrouted; it is refused across water.")))
	_touch_floor(_narrow(DccWidgets.toggle(parent, "Smooth", way_smooth_on(o), func(v: bool):
		way_write_smooth(options(app, "way"), v)
		refresh_surfaces(app, "way"),
		"Round the drawn path with a Catmull-Rom curve before it becomes waypoints. Off = the raw polyline.")))
	var adv := DccWidgets.segment(parent, "Advanced ▾" if way_advanced_open else "Advanced ▸", func():
		way_advanced_open = not way_advanced_open
		refresh_surfaces(app, "way"))
	adv.custom_minimum_size.y = 22
	_touch_floor(adv)
	adv.tooltip_text = "Corridor and pull steer the ROUTED mode: the corridor is how far the route may stray from your drawn line, the pull how hard it is drawn back to it."
	DccWidgets.set_segment_on(adv, way_advanced_open and mode == "routed")
	if mode != "routed":
		adv.disabled = true
		adv.tooltip_text = "Corridor and pull steer the routed mode only; freehand follows the drawn cells."
		return
	if way_advanced_open:
		## Waypoint spacing is here because corridor and pull only act BETWEEN waypoints: measured on a
		## drawn line 3 cells above an existing road, at the shell's 2-cell spacing every leg is 2 cells
		## long and the road stayed on the line for every corridor (3..15) and pull (0..100); at 6 cells
		## a pull of 0 let it leave the line by a cell and a strong pull did not. Without this control
		## the two below would be live and (almost) inert.
		var waypoints := DccWidgets.slider(parent, "Waypoints", WAY_SPACING_MIN, WAY_SPACING_MAX, 0.5,
			way_spacing(o), " c",
			func(v: float): way_write_spacing(options(app, "way"), v),
			"Distance between the waypoints a drag is turned into, in cells. Corridor and pull steer the road BETWEEN waypoints, so they matter more the farther apart these are.")
		_fit(waypoints)
		_touch_floor(waypoints)
		var corridor := DccWidgets.slider(parent, "Corridor", WAY_CORRIDOR_MIN, WAY_CORRIDOR_MAX, 0.5,
			way_corridor(o), " c",
			func(v: float): way_write_corridor(options(app, "way"), v),
			"How far, in cells, the routed road may stray from the drawn line.")
		_fit(corridor)
		_touch_floor(corridor)
		var pull := DccWidgets.slider(parent, "Pull", 0.0, WAY_PULL_MAX, 0.5, way_pull(o), "",
			func(v: float): way_write_pull(options(app, "way"), v),
			"How strongly the route is drawn back toward the drawn line (0 = plain least cost inside the corridor).")
		_fit(pull)
		_touch_floor(pull)

static func _controls_symmetry(parent: Control, app: DccApp, tool: String) -> void:
	var o := options(app, tool)
	var mode := sym_mode(o)
	_touch_floor(_narrow(DccWidgets.choice(parent, "Symmetry", SYM_LABELS, SYM_MODES.find(mode), func(i: int):
		sym_write_mode(options(app, tool), String(SYM_MODES[i]))
		refresh_surfaces(app, tool),
		"Mirrors every stroke about a centre: vertical, horizontal, both (dual) or N copies around it (radial). Each copy is committed with the stroke -- one undo step. Off by default.")))
	if mode == "none":
		return
	var amount: Dictionary
	if mode == "radial":
		amount = DccWidgets.slider(parent, "Copies", float(RADIAL_MIN), float(RADIAL_MAX), 1.0,
			float(sym_count(o)), "",
			func(v: float):
				sym_write_count(options(app, tool), v)
				refresh_marker(app),
			"How many copies are placed around the centre.",
			func(): refresh_surfaces(app, tool))
	else:
		amount = DccWidgets.slider(parent, "Angle", -ANGLE_MAX_DEG, ANGLE_MAX_DEG, 1.0,
			sym_angle_deg(o), "°",
			func(v: float):
				sym_write_angle_deg(options(app, tool), v)
				refresh_marker(app),
			"Rotates the mirror axis (or axes) about the centre.",
			func(): refresh_surfaces(app, tool))
	_fit(amount)
	_touch_floor(amount)
	var c = sym_centre(o)
	var pending := sym_pick_tool == tool
	var pick := DccWidgets.chip(parent, "Press the map…" if pending else "Pick centre",
		func(): sym_pick_begin(app, tool), pending)
	pick.custom_minimum_size.y = 22
	_touch_floor(pick)
	pick.tooltip_text = "Then press the map where the symmetry centre should be. That press places the centre and draws nothing."
	_hint(parent, "centre: grid middle" if c == null else "centre (%.1f, %.1f)" % [(c as Vector2).x, (c as Vector2).y],
		"Where the mirror falls. The grid's own middle until you pick one.")
	if c != null:
		var reset := DccWidgets.chip(parent, "Reset centre", func():
			sym_write_centre(options(app, tool), null)
			refresh_surfaces(app, tool))
		reset.custom_minimum_size.y = 22
		_touch_floor(reset)
		reset.tooltip_text = "Back to the grid's middle."

## Greys one `DccWidgets.slider()` row out and makes it inert, with the reason on hover.
## `editable = false` is the control's own off state; the row's alpha is the shell's
## convention for "unavailable, not merely low" (`_pg_paint_slider`'s greyed Original).
static func _disable_slider(parts: Dictionary, why: String) -> void:
	(parts["slider"] as HSlider).editable = false
	var row := parts["row"] as Control
	row.modulate.a = 0.5
	row.tooltip_text = why

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
			## Names the missing `measure_section` binding.
			b.tooltip_text = "Reads the height field: generate a world first." if not bridge.has_world \
				else "This build of the app is missing the cross-section feature -- rebuild it to enable this."

	row.add_child(DccTheme.rule(true))
	for id in MEASURE_GROUP_VERTICAL:
		_measure_button(row, String(id), current)

	## `Custom ▾`'s missing field lives in `global_tools.gd`'s SECTION_CHANNELS.
	_note(row, "measurements answer how far · the Sample dock answers what is here",
		"The canvas's own principle: Information is passive and always running; Measure is deliberate and persists until cleared; Cross-section is one line read in the strip below. " +
		"Two of its buttons in this row are not drawn because nothing exists behind them: `Custom ▾` has no user-defined field to choose from yet, and `3D distance` is greyed as \"3D only\" in the canvas itself -- this shell has no 3D view, and Δ vertical already returns the 3D distance in the dock.")

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
	## Three of the canvas's own Distance options are not built, and each is a
	## real absence rather than an omission -- stated on hover instead of drawn
	## as a dead control. Registered as GUI gaps, not hidden.
	##
	## **Was four until 2026-09-03.** The fourth read "units ▸ km: ... no such
	## preference exists in this shell yet, so every reading is km". False in
	## both clauses: `Preferences ▸ Units` is live with km / mi / nmi
	## (`menus.gd`'s `_units_popup`), and every measure readout this hover sits
	## above already formats through `DccUnits`. The count lives in the label as
	## well as the hover, so both change together.
	if mode_id == "distance" or mode_id == "bearing":
		## Backed by `cartalith_spatial::measure`, a planar measurer with a seam rule.
		_bar_hint(row, "— 3 canvas options unbuilt",
			"multi-segment / point-to-point: the six mode buttons above are that choice. " +
			"path ▸ great circle: this map is a flat projection, so there is no spherical path to offer. " +
			"snap ▸ settlements/rivers: the design spec lists no snap modifier for Measure, unlike Way/Route. " +
			"units ▸ km is BUILT, and is why this reads 3 rather than 4: Preferences ▸ Units carries km / mi / nmi, and every reading below converts accordingly.")
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label(GlobalTools.measure_status_text(), "text_dim", DccTheme.FS_SMALL))
	DccWidgets.chip(row, "Clear", func(): GlobalTools.measure_reset(app))
