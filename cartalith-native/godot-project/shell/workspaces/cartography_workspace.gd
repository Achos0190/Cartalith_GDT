extends Workspace
class_name CartographyWorkspace

## CARTO domain (§7 + §4.5.5): layer list, layer properties, colour ramp, stop
## editor, and the Icon/Label manual annotation tools.
##
## Presentation only. No control here may alter heightmap, climate, hydrology,
## biome classification, settlements, routes or seed, and none marks a
## generation stage stale -- which is why nothing in this file touches
## `bridge.mark_dirty()`. Icons and labels are the one exception §7's own
## prohibition allows (`DCC_SHELL_SPEC.md` §4.5.5): "cartographic annotation
## ... adds nothing to and takes nothing from the world model."
##
## §7 specifies three panes and a ten-row layer stack with three children under
## Terrain. Those three children -- Terrain, Colour relief, Hillshade -- are a
## real stack as of 2026-09-03 (`render::LayerStack`, CA-03/CA-04): visibility,
## opacity, blend mode and order, drawn by `render_workspace.gd`'s
## `_build_layer_stack()` into this file's own Layers category. §7's remaining
## rows are still whole overlays with a visibility switch each and no slot in
## the raster to order against; `_build_layer_gaps()` states that split.
##
## §4.5.5's Icon and Label tools (`UNIFIED_TOOL_PLAN.md` milestone F) are wired
## here in full: `icon_bridge.rs`/`label_bridge.rs` are bound and tested, and
## `map_overlay.gd`'s `set_manual_icons`/`set_labels` already render whatever
## `icon_list()`/`label_list()` return. CARTO has no dedicated right-dock
## context (`DCC_SHELL_SPEC.md` §6), so both tools' full property forms live
## in this dock's own "Annotation" category instead, and the tool options bar
## (`app.set_tool_options`) carries only the fast, frequently-changed subset
## the spec's own table names -- the two overlap by design, matching how the
## reference itself gives Label both a toolbar row and a fuller side panel.
##
## **Domain merge (2026-08-20, owner instruction: "And render into carto").**
## This dock now also carries the former RENDER domain's one subject
## (terrain appearance groups), via `_render` below -- a real
## `RenderWorkspace` instance, unmodified in what it does. This
## directly resolves the CA-01/RN-01 ambiguity `GUI_GAP_REGISTER.md` §8.6
## flagged: CARTO and RENDER were both proposing to own the same future
## `set_appearance()`-shaped `#[func]`; merging the domains removes the split.
##
## **v3 (2026-08-24, `design/Cartalith Menu Structure v3.dc.html`).** This dock
## was three L2 categories (Layers · Layer properties · Annotation) followed by
## RENDER's flat run of L3 sections. v3 names ten categories for CARTO and
## re-parents the same controls into them:
##
## | v3 category | Where its content came from |
## |---|---|
## | Map style | RENDER's Map style presets + Map view + Painter styles |
## | Terrain appearance | RENDER's Colour relief ramp + relief/sheet/materials/atmosphere |
## | Colours | RENDER's Colour grade + Grade field influence |
## | Layers | this file's Visible layers + Settlements-by-class + the gap notes |
## | Roads & routes | this file's Ways-by-type filter, out of Layers |
## | Labels | this file's Annotation ▸ label panel |
## | Assets & landmarks | this file's Annotation ▸ icon panel |
## | Political display | the provinces/territory rows, out of Visible layers |
## | Visibility / zoom | the analysis-field overlays (the Layers popover) |
## | Map presets | RENDER's Saved looks + Still owed |
##
## "Layer properties" is gone as a category: v3 folds per-layer opacity/blend/
## zoom into LAYERS' own rows, which is where they now are. When v3 landed it
## was one honest note about a capability that did not exist; the capability
## exists for the raster's three categories as of 2026-09-03, so LAYERS carries
## the rows themselves and the note beneath them is now about what is left.
## Nothing was rewritten -- every builder below is the one that was already
## here, called with a different parent.

## The layers the shell can actually toggle, in §7's own draw order:
## topmost first, matching how the layer list reads.
##
## Each entry's `on` is design documentation, not live state -- the checkboxes
## built below are seeded from the engine (`app.viewport.layer_visible()`),
## never from this field directly (`_layer_checks`'s own doc comment). It
## earns its keep by being asserted rather than left to drift silently:
## `_verify_layers_probe.gd` builds the same eight checkboxes this file does
## and fails if any of them disagrees with the engine's own default at launch
## (`godot --headless --script _verify_layers_probe.gd`; ALL PASS as of
## 2026-09-03). Re-run it after moving a default here, in `map_overlay.gd`'s
## `_show_*`/`_landmark*_visible` field initializers, or in
## `viewport_host.gd`'s `territory_view`/`province_view` initial `.visible` --
## whichever side moves, the probe is what says whether the other still agrees.
const LIVE_LAYERS: Array = [
	{"id": "settlements", "label": "Settlements", "on": true},
	{"id": "roads", "label": "Ways & routes", "on": true},
	{"id": "sea_routes", "label": "Sea routes", "on": true},
	## The reference's `civUrbanLayoutsChk`. **On by default here, where the
	## reference's own checkbox is off** — `map_overlay.gd`'s "Urban layouts"
	## block owns that divergence and its reasoning. In short: the reveal band
	## (the reference's own 24 km → 10 km span crossfade) means nothing is
	## generated or drawn until you deliberately zoom to town scale, so the
	## toggle is not carrying a cost the band does not already carry — and off
	## by default, on a row only reachable from this rail dock, is why the
	## layer went unseen.
	{"id": "urban_layouts", "label": "Town layouts (deep zoom)", "on": true},
	## Generated landmarks (`LANDMARK_GENERATION_SCOPE.md`). A layer row rather
	## than a control in the CIVIL panel that generates them, for the same
	## reason Settlements is one: this category answers "what is drawn", and the
	## domain that produces a thing is not the one that decides whether it is
	## visible. On by default — a pass the user deliberately ran should show its
	## result without a second step — and drawing nothing until one has run,
	## which is exactly how Sea routes behaves before any exist.
	{"id": "landmarks", "label": "Landmarks", "on": true},
	## The landmark funnel's rejected candidates (`LARGE_ITEM_RULINGS.md`'s
	## Landmark-funnel ruling, second half). **Off by default**, unlike the row
	## above it and for the opposite reason: a placement is the output of a pass
	## the user ran, while a rejection answers "why fewer than I asked for" --
	## a question they have to ask. A default run at 2048x1311 rejects 547 281
	## candidates and lists the best-scoring 3 216 of them, which is a real
	## diagnostic and would be pure noise arriving unasked on every world.
	{"id": "landmark_rejects", "label": "Landmark rejects (diagnostic)", "on": false},
	{"id": "provinces", "label": "Political — provinces", "on": false},
	{"id": "territory", "label": "Political — territory", "on": false},
]

## Which of `LIVE_LAYERS` v3 moves out of Layers and into its own **Political
## display** category. Named by id rather than by index so adding a layer above
## them cannot silently re-home one.
const POLITICAL_LAYERS: Array = ["provinces", "territory"]

## §4.5.5's Icon family/variant vocabulary -- mirrors
## `cartalith-assets/src/slots.rs`'s `PACK_SETTLEMENT_SLOTS`/`PACK_ICON_SLOTS`/
## `PACK_POI_SLOTS` exactly (frozen, order load-bearing: `icon_bridge::
## resolve_variant` indexes a family's own list positionally, by `variant`,
## so reordering this array would silently arm the wrong slot). "Custom" is
## excluded -- `IconEditor::arm` cannot address it through this numeric API
## (`icon_bridge.rs`'s own doc comment: its vocabulary is open, two levels
## deep, and not expressible as one index).
const ICON_FAMILIES: Array = [
	{"key": "settlement", "label": "Settlement", "slots": [
		"hamlet", "village", "town", "city", "capital",
		"monastery", "fortress", "university", "industrial"]},
	{"key": "feature", "label": "Feature", "slots": [
		"mountain", "hill", "tree_conifer", "tree_broadleaf", "tree_rainforest",
		"tree_savanna", "tree_wetland", "shrub", "cactus", "boulder"]},
	{"key": "poi", "label": "POI", "slots": [
		"ruin", "landmark", "mountain_peak", "named_forest",
		"battlefield", "shrine", "cave", "other"]},
]

## The per-class settlement filter (`#explSettlementFilterList`) and the
## by-way-type road filter (`#explShowRoads`'s own sub-list) the reference's
## layer popover carries. Both are real here: `get_settlements()` emits a
## `kind` on every row and `get_roads()` a `way_type`, so the filter is a
## draw-time test in `map_overlay.gd`, not a missing engine capability.
##
## `SETTLEMENT_KINDS` is the engine's own six tiers in metropolis-first
## order (`civ_tools_bridge::kind_from_str`). `metropolis` joined it on
## 2026-08-20 with the port of `_civSelectMetropolises`, closing
## `GUI_GAP_REGISTER.md` CV-04. `WAY_TYPES` is the reference's own
## `CIV_WAY_TYPES` (reference line 14743) minus its `sea-lane` row: sea lanes
## are drawn from `_sea_routes`, which already has its own top-level layer row
## above, so listing them here would give one thing two switches that disagree.
##
## **The two generated trunk tiers were missing until 2026-08-24.** This list
## held only `road`/`track`/`ancient` -- `infra_tools_bridge::parse_way_type`'s
## *manual* vocabulary -- but the switches drive `map_overlay.gd`'s filter on
## `get_roads()`' `way_type`, and the generated network classifies by
## `cartalith_civ::WayType`, whose two busiest tiers are `highway` and
## `regional`. Measured on a real 384x288 world: 13 highways and 17 regional
## roads against 4 roads and 1 track, so "Roads" off hid 4 of 35 ways and the
## other 30 could not be hidden at all. The reference lists all five and has
## since `CIV_WAY_TYPES` existed.
const SETTLEMENT_KINDS: Array = ["metropolis", "capital", "city", "town", "village", "hamlet"]
const WAY_TYPES: Array = [
	{"key": "highway", "label": "Trade highways"},
	{"key": "regional", "label": "Regional roads"},
	{"key": "road", "label": "Roads"},
	{"key": "track", "label": "Tracks"},
	{"key": "ancient", "label": "Ancient routes"},
]

const ICON_SCALE_MIN := 0.2   ## `cartalith_assets::manual::ICON_SCALE_MIN`.
const ICON_SCALE_MAX := 4.0   ## `cartalith_assets::manual::ICON_SCALE_MAX`.
const LABEL_SIZE_MIN := 8.0   ## `cartalith_civ::labels::LABEL_SIZE_MIN`.
const LABEL_SIZE_MAX := 48.0  ## `cartalith_civ::labels::LABEL_SIZE_MAX`.

## The density brush's two ranges -- `icon_bridge.rs`'s `ICON_BRUSH_R_MIN`/
## `_MAX` and `ICON_BRUSH_DENSITY_MIN`/`_MAX`, which are the reference's own
## `#carIconBrushR` (`min="2" max="60"`) and `#carIconBrushD` (`min="5"
## max="200"`, divided by 100 by its own listener) slider attributes. The
## engine clamps to exactly these, so a slider that could ask for more would
## only ever show a number the map did not use.
const ICON_BRUSH_R_MIN := 2.0
const ICON_BRUSH_R_MAX := 60.0
const ICON_BRUSH_DENSITY_MIN := 0.05
const ICON_BRUSH_DENSITY_MAX := 2.0

## What an in-progress canvas drag on the Label tool is doing -- set on
## `map_clicked` (a handle grab or a hit on an existing label's box), read on
## every subsequent `map_dragged` sample, cleared on `map_released`. `RESIZE`/
## `ROTATE`/`ARC` mirror the three handles `label_handles()` returns.
enum DragMode { NONE, MOVE, RESIZE, ROTATE, ARC }

## The Icon tool's own drag state -- `IconDragMode`'s one-handle mirror of
## `DragMode` above (`GUI_GAP_REGISTER.md` CA-05: `icon_handles()` returns
## exactly one circle, `"resize"`, since a manually-placed icon has no
## rotate/arc field at all -- `icon_bridge.rs`'s own doc comment). `MOVE`
## has no counterpart here: a box hit **selects** (2026-09-01 -- see
## `_on_icon_click`, which now calls `icon_hit_test` the way the Label tool
## calls `label_hit_test`) but does not start a move-drag, because there is
## no `icon_move` binding to drag against -- `icon_bridge.rs` exposes
## place, hit-test, resize and delete and nothing that rewrites an icon's
## `x`/`y`. Deleting and re-placing is the wired alternative.
enum IconDragMode { NONE, RESIZE }

# -- Icon tool state (UI-side only -- the engine holds the authoritative armed
# selection via `icon_arm`; this is just what the tool options row shows and
# re-arms from on every change). --------------------------------------------
var _icon_family_idx := 0
var _icon_variant_idx := 0
var _icon_scale := 1.0
var _icon_rotation := 0.0
var _icon_jitter := 0.0

# -- Density brush (`icon_bridge/brush.rs`, `UNIFIED_TOOL_PLAN.md` milestone
# E's last open half). The reference's own `_carIconBrush={on,r,density,
# painting}` split across the two sides it belongs on: the three settings live
# in the engine (`icon_brush_set`/`icon_brush`), and `painting` -- pointer
# state -- lives here, where the pointer is.
#
# Defaults mirror `IconBrush::default()` / the reference's own slider `value`
# attributes (line 1656-1657: r=12, density 60/100), so an untouched row and an
# untouched engine agree before the first `icon_brush_set` -- the same
# discipline `_paint_brush` and the settlement-class filter already follow. --
var _icon_brush_on := false
var _icon_brush_r := 12.0
var _icon_brush_density := 0.6
## Live only between `map_clicked` and `map_released` on a brush stroke.
var _icon_brush_painting := false
var _icon_list_body: VBoxContainer
## `DCC_SHELL_SPEC.md` §4.5.5 asks for both list panels "with counts and
## Clear-all". The count was never drawn and the button was live at zero, so
## the one control in each panel that acts on the whole list invited a press
## that could not do anything -- `GUI_GAP_REGISTER.md` **CA-20**. Held here so
## `_rebuild_icon_panel()`/`_rebuild_label_panel()`, which already run on every
## place, delete, clear and world change, own its state.
var _icon_clear_btn: Button

# -- Icon tool resize-handle drag state (CA-05) -- mirrors the Label tool's
# `_label_drag_*` fields one handle down; see `IconDragMode`'s own doc
# comment. --------------------------------------------------------------
var _icon_drag_mode := IconDragMode.NONE
var _icon_drag_index := -1
var _icon_drag_cx := 0.0
var _icon_drag_cy := 0.0
var _icon_drag_start_dist := 0.0

# -- Label tool state ---------------------------------------------------------
var _label_drag_mode := DragMode.NONE
var _label_drag_index := -1
var _label_drag_cx := 0.0   ## Grid-space label centre, `lb.x + 0.5` -- see
	## `_begin_label_handle_drag()`'s own comment for why the +0.5 matters.
var _label_drag_cy := 0.0
var _label_drag_start_dist := 0.0
var _label_drag_start_size := 0.0
var _label_drag_grab_angle := 0.0
var _label_drag_side := 0.0
var _label_list_body: VBoxContainer
var _label_edit_body: VBoxContainer
## See `_icon_clear_btn` -- **CA-20**, the same finding one panel down.
var _label_clear_btn: Button
## `GUI_GAP_REGISTER.md` **RF-05** -- see `_refresh_trade_load_row()`.
var _trade_load_toggle: CheckBox

## `LIVE_LAYERS` id -> the CheckBox both the "Visible layers" and "Political
## layers" loops build for it. Each is seeded from the engine's own
## `app.viewport.layer_visible(id)` at build time -- not from `LIVE_LAYERS`'s
## own `on` field, which the two loops never read; `on` is design
## documentation the engine agrees with today, checked by
## `_verify_layers_probe.gd` (see that field's own doc comment), not the
## value actually wired into the toggle. `_sync_layers()` is the read-back
## half, added for the identical defect
## `render_workspace.gd::_sync_color_space()` fixed: built once at launch, a
## checkbox never re-read the visibility a second writer --
## `civilization_workspace.gd`'s landmark-funnel "Show rejected" chip, calling
## `set_layer_visible()` directly -- could change out from under it.
var _layer_checks: Dictionary = {}

## The former RENDER domain, nested into this dock -- see this file's own
## class doc and `RenderWorkspace`'s own class doc for the mechanism.
var _render: RenderWorkspace


func _build() -> void:
	_render = RenderWorkspace.new()
	_render._nested = true

	DccWidgets.tools_block(self, app, app.tool_group, [
		{"id": "icon", "glyph": "tool_icon", "label": "Icon (I)"},
		{"id": "label", "glyph": "tool_label", "label": "Label (L)"},
	])

	## Added and set up *before* the categories, because those categories are
	## what it draws into now: `_render.setup()` runs its own `_build()`, which
	## returns immediately while `_nested`, leaving this node holding RENDER's
	## state (ramp, preset chips, appearance rows, the water-anim layer) and
	## drawing nothing of its own. See `RenderWorkspace._host`.
	add_child(_render)
	_render.setup(app, bridge)

	## 1 -- MAP STYLE
	_render.build_map_style_into(
		DccWidgets.category(self, "Map style", categories, true))

	## 2 -- TERRAIN APPEARANCE
	_render.build_terrain_appearance_into(
		DccWidgets.category(self, "Terrain appearance", categories))

	## 3 -- COLOURS
	_render.build_colours_into(
		DccWidgets.category(self, "Colours", categories))

	## 4 -- LAYERS
	var cat := DccWidgets.category(self, "Layers", categories)
	var body := DccWidgets.section(cat, "Visible layers")
	for layer in LIVE_LAYERS:
		if POLITICAL_LAYERS.has(String(layer.id)):
			continue   ## v3 gives these their own category -- see Political display.
		_layer_checks[layer.id] = DccWidgets.toggle(body, layer.label,
			app.viewport.layer_visible(layer.id),
			func(on: bool): app.viewport.set_layer_visible(layer.id, on))
	DccWidgets.note(body,
		"Each row above is a whole overlay with nothing inside it to order. "
		+ "The terrain raster's own three categories are the stack below.")
	## §7's layer list, for the part of it that is a *stack* -- Terrain, Colour
	## relief and Hillshade, with visibility, opacity, blend mode and order.
	## `render_workspace.gd` owns it beside the ramp and the tunables (it is
	## `TerrainAppearance` state); this is where §7 draws it.
	_render.build_layer_stack_into(cat)
	_build_settlement_class_filter(cat)
	_build_layer_gaps(cat)

	## 5 -- ROADS & ROUTES
	_build_way_style(DccWidgets.category(self, "Roads & routes", categories))

	## 6 -- LABELS. The rail's `labels` node lands here (`RAIL_NODES`).
	##
	## Two panels, in the order the design draws them: the ENV prototype's
	## per-class typography block first (`ENV:698`-`721`, engine-blocked, drawn
	## disclosed -- see `_build_label_classes()`), then this shell's own
	## hand-placed region-label list, which is live.
	var labels_cat := DccWidgets.category(self, "Labels", categories)
	_build_label_classes(labels_cat)
	_build_label_panel(labels_cat)

	## 7 -- ASSETS & LANDMARKS. The rail's `icons` node lands here.
	##
	## Same shape: the prototype's automatic-placement block (`ENV:731`-`755`),
	## then the live list of icons the Icon tool has stamped.
	var icons_cat := DccWidgets.category(self, "Assets & landmarks", categories)
	_build_icon_placement(icons_cat)
	_build_icon_panel(icons_cat)

	## 8 -- POLITICAL DISPLAY
	_build_political_display(DccWidgets.category(self, "Political display", categories))

	## 9 -- VISIBILITY / ZOOM
	_build_visibility(DccWidgets.category(self, "Visibility / zoom", categories))

	## 10 -- MAP PRESETS
	_render.build_presets_into(
		DccWidgets.category(self, "Map presets", categories))

	_register_tools()
	bridge.generation_finished.connect(func(ok: bool): if ok: _on_world_changed())
	bridge.world_loaded.connect(_on_world_changed)
	## A layer flipped from OUTSIDE this dock -- the landmark funnel's "Show
	## rejected" chip (`civilization_workspace.gd::_lm_show_rejects()`) is the
	## one real caller today -- now reaches these checkboxes live rather than
	## only on the next world change. `_sync_layers()` already does the actual
	## read-back correctly (verified by `_verify_layers_probe.gd`); it used to
	## run only from `_on_world_changed()`. That was the gap: because
	## `app.gd::_register_workspaces()` builds every workspace eagerly at
	## launch, these checkboxes already exist before any click can happen, so a
	## build-time read cannot cover one and a click before the next regenerate
	## had nothing to trigger the sync.
	app.viewport.layer_visibility_changed.connect(func(_layer: String, _shown: bool): _sync_layers())

	## A world may already exist when this dock is first built -- CARTO is not
	## the workspace the app opens on, so `generation_finished` has usually
	## already fired by the time anyone comes here. Without this the class
	## counts would sit at `--` over a fully populated world until the user
	## happened to move a dial. Deferred because `_regenerate_labels()` ends by
	## refreshing the viewport's annotation layers, and nothing else in `_build()`
	## touches `app.viewport` before the tree is up.
	call_deferred("_regenerate_labels")


## The reference's own two sub-filters, wired live (see `SETTLEMENT_KINDS`'
## doc comment for why these are real rather than engine-blocked). Every row
## starts on, matching `map_overlay.gd`'s "empty hidden-set means show
## everything" default -- so an untouched panel and an untouched overlay
## agree before the first click, the same discipline `world_workspace.gd`'s
## `_paint_brush` mirror already follows.
func _build_settlement_class_filter(parent: Control) -> void:
	var kinds := DccWidgets.group(parent, "Settlements · by class", false)
	for kind in SETTLEMENT_KINDS:
		DccWidgets.toggle(kinds, String(kind).capitalize() + "s", true,
			func(on: bool): app.viewport.set_settlement_kind_visible(String(kind), on))
	DccWidgets.note(kinds,
		"A hidden class is not drawn but stays hoverable and clickable -- hiding "
		+ "a tier is a cartographic choice, not a reason to make a place "
		+ "unselectable. The master Settlements switch above still gates the "
		+ "whole layer.")


## v3 CARTO ▸ ROADS & ROUTES. The per-type visibility switches are real and
## have been since the way-type filter shipped; they moved out of Layers
## because v3 gives ways and routes a category of their own.
##
## **Its geometry is not here, and that is the category's whole point**: v3's
## own footnote reads *"Geometry, class and cost belong to CIVIL ▸ Routes &
## Ways. Nothing here changes where a road runs."*
func _build_way_style(parent: Control) -> void:
	var types := DccWidgets.section(parent, "Ways · by type")
	for t in WAY_TYPES:
		DccWidgets.toggle(types, String(t["label"]), true,
			func(on: bool): app.viewport.set_way_type_visible(String(t["key"]), on))
	DccWidgets.note(types,
		"Every land way type get_roads() can emit: the generated network's four "
		+ "usage tiers (cartalith_civ::WayType) plus hand-drawn ancient routes. "
		+ "Sea lanes are a whole-layer switch under Layers rather than a sixth "
		+ "row here.")
	DccWidgets.action(types, "Draw and edit ways → Civilization ▸ Routes & ways",
		func(): app.select_domain_category("civilization", "Routes & ways")).alignment = HORIZONTAL_ALIGNMENT_LEFT

	## `GUI_GAP_REGISTER.md` **CA-16**, the reference's own two per-layer way
	## style controls (`#civWayScaleR` line 1485, `#wayOpacityR` line 1491).
	## Registered as unbacked on the reading that "map_overlay.gd draws every
	## way with one hardcoded width-and-colour pair per type" -- by then §36 had
	## already replaced that flat pair with the reference's five two-stroke
	## styles, and what was genuinely missing was the user multiplier those two
	## sliders are. Both are now the third term of the reference's own `rsc`
	## and its `globalAlpha`.
	var style := DccWidgets.section(parent, "Way style")
	DccWidgets.slider(style, "Line width", 0.2, 2.5, 0.05,
		app.viewport.overlay.way_scale(), "×",
		func(v: float): app.viewport.overlay.set_way_scale(v),
		"state.viz.civWayScale -- the user multiplier on every way, sea lane, route and journey line width, and on every dash length with it. The per-type colour, casing and dash pattern underneath are the reference's own five styles and are not editable: they are what makes a track read as a track.")
	DccWidgets.slider(style, "Opacity", 0.0, 1.0, 0.01,
		app.viewport.overlay.way_opacity(), "",
		func(v: float): app.viewport.overlay.set_way_opacity(v),
		"state.viz.wayOpacity -- one alpha multiplier over the whole way layer, on top of each stroke's own authored alpha. 0 is a hidden layer, which the Ways & routes switch under Layers already is.")
	DccWidgets.toggle(style, "Drop minor ways when zoomed out",
		app.viewport.overlay.way_lod(),
		func(on: bool): app.viewport.overlay.set_way_lod(on),
		"CIV_LOD_ROAD, the reference's own per-type zoom ladder (GUI_GAP_REGISTER.md CA-18): tracks and ancient routes stop drawing below 0.7× zoom, where they are a 1 px scratch. Roads sit at 0.35× and this camera's floor is 0.4×, so they never drop; the two trunk tiers are always drawn.")

	## `GUI_GAP_REGISTER.md` **IN-13**'s map surface. Here and not in the
	## Layers popover: that popover is the one picker for *field rasters*
	## (`set_debug_layer`), and trade load is not a field — it is a value on a
	## way, drawn by the way layer this section already owns. Two pickers over
	## one concept is the shape this shell keeps having to undo.
	var load_sec := DccWidgets.section(parent, "Trade load")
	_trade_load_toggle = DccWidgets.toggle(load_sec, "Thicken ways by carried volume",
		app.viewport.overlay.show_trade_load(),
		func(on: bool): app.viewport.overlay.set_show_trade_load(on),
		"Draws each way at up to 2.6x its normal width in proportion to the trade it carries, on its own colour -- width and not hue, because a way's colour is already its type. Relative to the busiest way on this world, since volume is a population sum and populations are not comparable between worlds.")
	## `GUI_GAP_REGISTER.md` **RF-05**. This category is built once, at launch,
	## against an engine with no world in it -- so the row was born disabled and
	## the match that makes it valid (CIVIL ▸ Trade ▸ Match trade flows, a
	## different workspace) had no way to say so. Driven from the overlay's own
	## `set_trade_load`, which is the single funnel both the match and the
	## world-change clear already pass through, so the row cannot disagree with
	## the data it draws in either direction.
	app.viewport.overlay.trade_load_changed.connect(_refresh_trade_load_row)
	_refresh_trade_load_row(app.viewport.overlay.has_trade_load())
	DccWidgets.note(load_sec,
		"The numbers behind it -- which ways carry what, and which carry nothing -- "
		+ "are in Civilization ▸ Trade ▸ Way load.")

	var gaps := DccWidgets.section(parent, "Not built")
	DccWidgets.note(gaps,
		"Per-class colour, casing, dash pattern and route glow "
		+ "(GUI_GAP_REGISTER.md CA-16). Width and opacity above are the two "
		+ "controls the reference ships and they act on the whole layer; the "
		+ "five per-type styles under them are ported literals (§36), and making "
		+ "one editable means a style record keyed by way type for the overlay "
		+ "to read instead of its own WAY_STYLE constant.")


## Layer surfaces the reference had, or the design asks for, that this shell
## genuinely does not carry -- stated rather than left to be inferred from
## their absence, per `menus.gd`'s own honesty rule.
func _build_layer_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	## **This note said the opposite until 2026-09-03**, and the reason it gave
	## was wrong about the renderer as well as stale: it claimed terrain,
	## hillshade and colour relief were "composited into one raster by render.rs
	## before it crosses the boundary, so there are no separable outputs to order
	## or blend". Both composites already existed inside `land_color`, hardcoded
	## -- a normal-over lerp and a multiply -- and `render::LayerStack` moved the
	## operator and the slot out of source and into data. Opacity, blend mode and
	## order are live for those three above. What is still missing is the *other*
	## thirteen rows of v3's stack, and they are missing for a different reason.
	DccWidgets.note(sec,
		"Per-layer opacity, blend mode and order (GUI_GAP_REGISTER.md CA-04) are "
		+ "live for the terrain raster's three categories -- the stack above. "
		+ "They are not live for the rest of the design's layer list, and that is "
		+ "a different problem, not the same one half-finished: Water is a sibling "
		+ "of Terrain rather than one of its children (sea colour folds its own "
		+ "shade in and has no ramp at all), the design's fourth child "
		+ "\"Hand-drawn hillshade\" is the Painter block and is already switchable "
		+ "under Map style, and the annotation and civilisation rows are separate "
		+ "overlay passes drawn after the raster, with a visibility switch each "
		+ "and no slot in it to order. Per-layer zoom range and the picking/clip "
		+ "switches rest on that second separation, not on the one that landed.")
	DccWidgets.note(sec,
		"Show rivers in biome view (#showRivers) and Rivers as ways: both are "
		+ "reference RENDER filters, and neither is wired here -- but not for the "
		+ "reason this note gave until 2026-09-03. The network does cross the "
		+ "boundary: WorldGen.get_rivers(min_order) returns every traced run as an "
		+ "entity with its own polyline, and river_at() selects one (the right "
		+ "dock's River context does exactly that). What is missing is on the "
		+ "drawing side, and differs per filter: the biome raster's rivers are the "
		+ "simple channel-mask tint baked into the terrain texture, with no "
		+ "parameter to switch it off; and rivers-as-ways is drawRiverWays, the one "
		+ "thing render.rs's module doc still lists as excluded -- a vector overlay "
		+ "over get_rivers()' polylines that nothing draws yet.")
	DccWidgets.note(sec,
		"Sharper ecotones (biome-detail sharpening) is not parameterised: biome "
		+ "classification runs off the finished temperature/rainfall fields with no "
		+ "dials of its own -- see World ▸ Biomes for the same finding.")


## v3 CARTO ▸ POLITICAL DISPLAY. The two political layer switches, out of the
## Layers list, plus the honest statement of what the rest of v3's category
## (border line style, claim hatching, influence gradient, legend) rests on.
##
## v3's own rule for this category: identity colour is CIVIL's, *how* it paints
## is CARTO's. Today the overlay owns both -- `map_overlay.gd` derives a
## faction's tint from its index and takes no style argument -- which is why
## nothing below offers a colour.
func _build_political_display(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Political layers")
	for layer in LIVE_LAYERS:
		if not POLITICAL_LAYERS.has(String(layer.id)):
			continue
		_layer_checks[layer.id] = DccWidgets.toggle(sec, layer.label,
			app.viewport.layer_visible(layer.id),
			func(on: bool): app.viewport.set_layer_visible(layer.id, on))
	DccWidgets.note(sec,
		"Territory is the per-cell claim map; provinces are its partition into "
		+ "named administrative units. Both are recomputed by Civilization ▸ "
		+ "Territories, never by anything in this dock.")
	var edit := DccWidgets.action(sec, "Edit territories → Civilization ▸ Territories",
		func(): app.select_domain_category("civilization", "Territories"))
	edit.alignment = HORIZONTAL_ALIGNMENT_LEFT

	## `GUI_GAP_REGISTER.md` **CA-17**, the CARTO half of v3's split: CIVIL
	## owns *which* colour a faction is, this owns *how heavily* it is laid on.
	## The reference's own `#territoryOpacityR` (line 1490), which this port had
	## as a hardcoded 82/255 in `build_territory_texture`.
	var tint := DccWidgets.section(parent, "Territory tint")
	var op := DccWidgets.slider(tint, "Fill opacity", 0.0, 1.0, 0.01,
		bridge.territory_opacity(), "",
		func(v: float):
			bridge.set_territory_opacity(v)
			app.viewport.territory_view.texture = bridge.territory_texture(),
		"state.viz.territoryOpacity. This port starts at %.2f rather than the reference's 0.51: there is a hillshade, a splat and a colour grade under this wash that the reference's flat biome fill does not have, and a heavier tint buries them." % bridge.territory_opacity_default())
	var reset := DccWidgets.text_button(tint, "Reset to %.2f" % bridge.territory_opacity_default(),
		func():
			bridge.set_territory_opacity(-1.0)
			## Drive the slider rather than rebuild the category: `value` fires
			## `value_changed`, which re-applies the same number and repaints,
			## so the control and the map cannot disagree about what Reset did.
			(op["slider"] as HSlider).value = bridge.territory_opacity_default())
	reset.tooltip_text = "Back to this port's own default fill opacity."
	var colour := DccWidgets.action(tint, "Faction identity colours → Civilization ▸ Factions",
		func(): app.select_domain_category("civilization", "Factions"))
	colour.alignment = HORIZONTAL_ALIGNMENT_LEFT
	colour.tooltip_text = "v3's own rule for this category: which colour a faction *is* belongs to CIVIL, how heavily it is painted belongs here. The roster's colour picker writes the identity colour this wash draws in."

	## **Rewritten 2026-09-01: two of these three stopped being data gaps and
	## nobody moved the note.** It said claim hatching and the influence
	## gradient "rest on data that does not exist" -- that
	## `CivData::territory` is one plurality owner per cell "with no
	## contested-claim value and no influence field for a gradient to ramp".
	## CV-23 closed exactly that: `sample_bridge::territory_influence` builds
	## owner, rival, influence and contested per cell on demand, `#[func]
	## civ_territory_influence` aggregates it, and `sample_bridge`'s own
	## `"contested"` debug raster already *draws* it -- dimmed owner tint
	## inside, the rival's colour hatched in past `CONTEST_HATCH_T`, with a
	## four-row legend. So what is missing here is a control and a legend in
	## THIS panel, not a quantity in the engine, and the note says which.
	var gaps := DccWidgets.section(parent, "Not built")
	var contested := DccWidgets.action(gaps, "Claim hatching and the influence ramp → Layers ▸ Civilization ▸ Contested borders",
		func():
			## `set_debug_layer` then open the popover, the same pair
			## `render_workspace.gd`'s own "Biome colour table → Layers ▸
			## Biomes" row uses: setting the layer behind the picker's back
			## would leave its rows naming a different view, and the popover
			## rebuilds its rows on open.
			app.viewport.set_debug_layer("contested")
			app.layers_popover.open())
	contested.alignment = HORIZONTAL_ALIGNMENT_LEFT
	contested.tooltip_text = "territory_influence(): how evenly the owner and its nearest rival reach each cell. Secure interiors keep a dimmed owner tint, frontiers hatch into the rival's colour, and the popover carries the ramp's own legend. Built on demand from the capitals -- one Dijkstra per capital -- and held nowhere."
	DccWidgets.note(gaps,
		"Border line width and style (GUI_GAP_REGISTER.md CA-17). The faction "
		+ "wash has no outline at all -- build_territory_texture() is a per-cell "
		+ "fill and nothing traces its edge -- and the province line that does "
		+ "exist (build_province_boundary_texture) is one cell wide in one "
		+ "hard-coded ink tone, with no argument for either. Claim hatching and "
		+ "the influence gradient are NOT in that state: both are real and both "
		+ "are drawn, as the Contested borders view above. What they lack is a "
		+ "styling control and a legend inside this category, which is a "
		+ "different kind of gap from a missing quantity. Fill opacity and "
		+ "identity colour, above, are the two that already have one.")


## v3 CARTO ▸ VISIBILITY / ZOOM. Its `§ Data overlays` band is the reference's
## own Analysis field, which v3's migration audit moves here from View -- and
## which this shell already has, in full, as the map canvas's Layers popover
## (`layers_popover.gd`, built from the engine's own `debug_layers()` table).
##
## A second copy of that list in this dock would be two pickers over one
## `set_debug_layer()`, and this shell has been bitten by exactly that shape
## before (the bake button, the recompute rows). So this is one button onto the
## one picker, not a reimplementation.
func _build_visibility(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Data overlays")
	var groups := bridge.debug_layers()
	var n := 0
	for g in groups:
		n += (g as Dictionary).get("items", []).size()
	if n == 0:
		DccWidgets.note(sec,
			"No field views: this build's engine has no debug_layers() binding.")
	else:
		DccWidgets.note(sec,
			("%d analysis fields across %d groups -- elevation, slope, aspect, "
			+ "curvature, flow accumulation, temperature, rainfall, wind, currents, "
			+ "soil, lithology, biome and political control, each with its own "
			+ "legend and a shared opacity. A view whose input this world lacks is "
			+ "greyed with its reason.") % [n, groups.size()])
	var open := DccWidgets.action(sec, "Data overlays…", func(): app.layers_popover.open(), true)
	open.tooltip_text = "The map canvas's Layers popover -- the one picker for every analysis field, anchored under the viewport's own Layers button. Hotkeys 1-8 select the first eight available views."

	var gaps := DccWidgets.section(parent, "Partly built")
	DccWidgets.note(gaps,
		"The zoom ladder exists for the two layers the reference ships one for: "
		+ "ways drop by type below their CIV_LOD_ROAD threshold (Roads & routes "
		+ "▸ Way style), and a town's drawn layout crossfades in over a 24-10 km "
		+ "span. Neither is a *user* range, and the other fourteen layers v3 "
		+ "lists have none at all (GUI_GAP_REGISTER.md CA-18) -- a per-layer "
		+ "zoom range needs each of those layers to be a stack row, which the "
		+ "overlay passes are not. CA-04 landed for the terrain raster's three "
		+ "categories (Layers - Terrain raster) and does not reach them.")
	DccWidgets.note(gaps,
		"Declutter budget  ·  still not built\n"
		+ "Collision itself IS resolved -- this note claimed otherwise until "
		+ "2026-09-03, on the day the culler landed. Labels are culled against each "
		+ "other by cartalith-civ's labels.rs label_cull_rect, the labelling pass "
		+ "reports its own drawn/culled counts above, and the icon placement pass's "
		+ "'avoid label boxes' rule suppresses a glyph that lands on a label's box "
		+ "using that same culler. What is not built is a BUDGET: a cap on how much "
		+ "annotation a given zoom may spend. That needs a per-layer zoom range to "
		+ "spend it over, and that needs the annotation overlays to be stack rows -- "
		+ "which is the half of CA-04 the 2026-09-03 raster stack did not do.\n"
		+ "Two ladders do exist, both ported: way types by CIV_LOD_ROAD, and the "
		+ "24-10 km urban-layout crossfade.")
	DccWidgets.note(gaps,
		"Population density, political control as a *choropleth*, and trade "
		+ "influence: control is a real debug view above; the other two have no "
		+ "field. Settlement population is per-place, not rasterised, and no trade "
		+ "influence field is computed anywhere in cartalith-civ.")


# ===========================================================================
# Tool arming / click-drag wiring
# ===========================================================================

func _register_tools() -> void:
	app.register_tool_click_handler("icon", _on_icon_click)
	app.register_tool_drag_handler("icon", _on_icon_drag)
	app.register_tool_release_handler("icon", _on_icon_release)

	app.register_tool_click_handler("label", _on_label_click)
	app.register_tool_drag_handler("label", _on_label_drag)
	app.register_tool_release_handler("label", _on_label_release)

	## Neither tool needs its own Escape handler: `DCC_SHELL_SPEC.md` §4.5.6's
	## "otherwise disarms back to Inspect" is exactly `app.gd`'s own default
	## fallback (no handler registered -> disarm to inspect), and that default
	## still calls `arm_tool("inspect")`, which fires `tool_armed` -- routed
	## to `_on_any_tool_armed` below, which is what actually disarms the
	## engine-side icon selection and clears the label drag/handle state. A
	## bespoke escape handler here would only duplicate that cleanup.
	app.tool_armed.connect(_on_any_tool_armed)


## The single place both tools' engine-side/overlay cleanup happens, reached
## whether a tool was armed by clicking its own button, by clicking a
## *different* tool's button, or by the default Escape fallback in `app.gd`
## (which also calls `arm_tool`, and so also emits this signal) -- see
## `_register_tools()`'s own comment.
func _on_any_tool_armed(id: String) -> void:
	if id != "icon":
		bridge.icon_disarm()
	## `05-right-dock-and-bars.md` §1.10/§1.9, GUI replacement stage 5.
	## `rdMode4()`'s own fall-through: Label/Icon (rule 3) always win the
	## right dock over Stops (rule 6), which is why the Anno calls sit inside
	## their own branches below and the Stops call sits only in `_`, gated on
	## `id == "inspect"` so it never fires while Measure/Region are arming
	## through this same signal (`global_tools.gd`'s own handler owns those
	## two, and connection order between the two listeners is not something
	## to depend on -- `leave_stops_context()`/`leave_anno_context()` below
	## are no-ops unless THIS file's own context is the live one, so this is
	## safe regardless of which handler runs first).
	match id:
		"icon":
			_arm_icon_from_ui()
			app.set_tool_options(_build_icon_tool_options_row)
			_rebuild_icon_panel()
			if app.right_dock_ctrl.has_method("show_anno"):
				app.right_dock_ctrl.show_anno()
		"label":
			app.set_tool_options(_build_label_tool_options_row)
			_rebuild_label_panel()
			if app.right_dock_ctrl.has_method("show_anno"):
				app.right_dock_ctrl.show_anno()
		_:
			_label_drag_mode = DragMode.NONE
			_label_drag_index = -1
			_icon_drag_mode = IconDragMode.NONE
			_icon_drag_index = -1
			## The reference's own `pointercancel` (line 9753) drops
			## `_carIconBrush.painting` alongside every other in-flight drag,
			## and disarming the tool is this shell's equivalent event: a
			## stroke interrupted by Escape must not leave the flag set, or
			## the next `map_dragged` on a re-armed Icon tool would resume
			## painting without a press.
			_icon_brush_painting = false
			app.viewport.tool_overlay.set_handles([])
			if app.active_domain() == "cartography":
				_show_style_tool_options()
			if app.right_dock_ctrl.has_method("leave_anno_context"):
				app.right_dock_ctrl.leave_anno_context()
			if id == "inspect" and app.active_domain() == "cartography" and app.right_dock_ctrl.has_method("show_stops"):
				app.right_dock_ctrl.show_stops()
			elif app.right_dock_ctrl.has_method("leave_stops_context"):
				app.right_dock_ctrl.leave_stops_context()


## `right_dock.gd`'s CTX_ANNO (§1.10) reads `label_get_selected()`/
## `label_list()`/`icon_list()` fresh on every rebuild, so a selection or a
## count change here (create, select, drag-release, delete, clear-all) has
## to re-announce it the same way `_refresh_right_dock_paint()`
## (`world_workspace.gd`) does for Paint. Called from both `_rebuild_label_
## panel()`'s and `_rebuild_icon_panel()`'s own tails rather than from every
## individual click/drag/delete handler that already funnels into one of
## those two -- gated on the armed tool so `_on_world_changed`'s own
## unconditional rebuild of both panels on every regenerate cannot steal the
## right dock from whatever context was actually showing.
func _refresh_right_dock_anno() -> void:
	if (app.armed_tool == "label" or app.armed_tool == "icon") and app.right_dock_ctrl.has_method("show_anno"):
		app.right_dock_ctrl.show_anno()


## `GUI_GAP_REGISTER.md` **RF-05**: the row follows the data, in both
## directions. Enabled the moment a match produces per-way volumes; disabled
## again, with the reason back on it, the moment a world change drops them --
## and the switch itself is turned off, because leaving it *on* over an empty
## reading would be a live toggle that draws nothing.
func _refresh_trade_load_row(available: bool) -> void:
	if _trade_load_toggle == null or not is_instance_valid(_trade_load_toggle):
		return
	_trade_load_toggle.disabled = not available
	if available:
		_trade_load_toggle.tooltip_text = ""
	else:
		if _trade_load_toggle.button_pressed:
			_trade_load_toggle.button_pressed = false
			app.viewport.overlay.set_show_trade_load(false)
		_trade_load_toggle.tooltip_text = ("No trade match has been run on this world. "
			+ "Civilization ▸ Trade ▸ Match trade flows produces the per-way volume this "
			+ "draws; it is computed on demand and held nowhere, so a generate clears it.")


func _on_world_changed() -> void:
	## RENDER first: it is nested in this workspace (`_render`), so
	## `app.gd`'s own `on_world_changed` broadcast never reaches it -- that
	## walks the *registered* workspaces, and RenderWorkspace stopped being
	## one at the 2026-08-20 domain merge. Until 2026-09-01 nothing else
	## reached it either: `grep "world_loaded|generation_finished" ` over
	## `render_workspace.gd` matched nothing, so five of the six things
	## `project_bridge.rs`'s `AppearanceDoc` restores on File ▸ Open showed
	## launch-time values afterwards -- and the ramp editor, which pushes its
	## whole shell-side list on any stop edit, would then write the pre-open
	## ramp back over the restored one. See `RenderWorkspace.on_world_changed`.
	if _render != null:
		_render.on_world_changed()
	_sync_layers()
	app.viewport.tool_overlay.set_handles([])
	_label_drag_mode = DragMode.NONE
	_label_drag_index = -1
	_icon_drag_mode = IconDragMode.NONE
	_icon_drag_index = -1
	if app.armed_tool == "icon":
		_arm_icon_from_ui()
	_rebuild_icon_panel()
	## `filled` is a count over the *loaded pack*, and File ▸ Open restores one
	## with the project. Re-reading here is the same reason the labelling pass
	## re-runs below: the previous answer describes a world that no longer
	## exists. The chips and sliders themselves are built once and stay.
	_refresh_icon_placement_rows()
	_rebuild_label_panel()
	## Every feature the generated pass names -- continents, provinces,
	## settlements, lakes, landmarks -- has just been replaced, so the previous
	## run describes a world that no longer exists. Re-running is also what
	## restores the class typography table, which `WorldGen::absorb` resets with
	## the rest of the label bridge (see `LabelBridge::typography`).
	_regenerate_labels()

## Re-read every layer checkbox from `app.viewport.layer_visible()` -- the
## read-back half of the pair `render_workspace.gd::_sync_color_space()`'s own
## comment describes. `set_pressed_no_signal`, not the plain `button_pressed`
## property: writing the value back into the engine it was just read from is
## the asymmetry `_sync_appearance`'s own note describes, and here it would
## also fire `set_layer_visible()` right back with the value this loop just
## read out of it.
func _sync_layers() -> void:
	for id in _layer_checks:
		(_layer_checks[id] as CheckBox).set_pressed_no_signal(app.viewport.layer_visible(String(id)))


## Duplicates `app.gd`'s own `_tool_options_simple("CARTOGRAPHY · STYLE", ...)`
## text -- `app.gd` is off-limits to edit in this pass, and only it rebuilds
## the bar on a workspace switch, not on arming Measure/Region/Inspect while
## already in Cartography, which this file's own tool arming now needs to.
func _show_style_tool_options() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		row.add_child(DccTheme.mono_label("CARTOGRAPHY · STYLE", "accent", DccTheme.FS_SMALL, 2, true))
		row.add_child(DccTheme.label(
			"presentation only — no control here marks a generation stage stale",
			"text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer()))


# ===========================================================================
# Icon tool (§4.5.5)
# ===========================================================================

func _arm_icon_from_ui() -> void:
	if not bridge.has_asset_pack():
		return
	var fam: Dictionary = ICON_FAMILIES[_icon_family_idx]
	bridge.icon_arm(fam.key, _icon_variant_idx, _icon_scale, _icon_rotation, _icon_jitter)
	## The brush's three settings ride along on every re-arm rather than only on
	## their own sliders' `value_changed`, for the reason the armed selection
	## does: the engine's editor is rebuilt by `absorb()` on every generate, so
	## a brush set before a regenerate would otherwise silently revert to
	## `IconBrush::default()` while this row still showed the user's numbers.
	bridge.icon_brush_set(_icon_brush_on, _icon_brush_r, _icon_brush_density)


## Display names for one manual-icon family's slots, from the engine's own
## `slot_title()` table rather than from `String.capitalize()`.
##
## The ids in `ICON_FAMILIES` above are the engine's frozen vocabularies
## verbatim (`slots.rs`'s `PACK_SETTLEMENT_SLOTS` / `PACK_ICON_SLOTS` /
## `PACK_POI_SLOTS`, checked element for element) and `icon_arm`'s
## `variant` indexes them, so the ORDER here is load-bearing and is not
## touched -- only the wording is. Capitalising the id disagreed with the
## engine on about a quarter of them: `tree_conifer` reads "Conifer tree",
## `ruin` reads "Ruin / old settlement", `shrine` reads "Shrine / temple",
## `cave` reads "Cave / tunnel". `library.rs`'s own module doc calls those
## titles functionally load-bearing, and the asset library window shows
## them, so the same slot was named two ways in two panels.
##
## `feature` → `icons` is the one family rename
## (`manual.rs::ManualIconFamily::pack_family`); settlement and poi carry
## across unchanged. Falls back to `capitalize()` per slot, so an older
## cdylib with no `as_family_slots` (or a slot the library does not carry)
## reads exactly as it did before rather than blank.
func _slot_labels(family_key: String, slots: Array) -> Array:
	var titles := {}
	var pack_family := "icons" if family_key == "feature" else family_key
	for row in bridge.as_family_slots(pack_family):
		var d: Dictionary = row
		## `title`, not `name`: `Node.name` is a base-class property and a local
		## called that shadows it.
		var title := String(d.get("name", ""))
		if not title.is_empty():
			titles[String(d.get("id", ""))] = title
	var out: Array = []
	for sl in slots:
		out.append(String(titles.get(String(sl), String(sl).capitalize())))
	return out

## Tool options row: `CARTO · ICON` · family · variant · scale · rotation ·
## jitter (`DCC_SHELL_SPEC.md` §4.5.5's own table). Live: every control
## re-arms immediately, matching the reference's "the armed selection is
## whatever the row currently shows" model -- there is no separate Arm button.
func _build_icon_tool_options_row(row: HBoxContainer) -> void:
	row.add_child(DccTheme.mono_label("CARTO · ICON", "accent", DccTheme.FS_SMALL, 2, true))
	if not bridge.has_asset_pack():
		row.add_child(DccTheme.label(
			"no asset pack loaded — File ▸ Import asset pack to place icons",
			"stale", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		return

	var fam_labels: Array = []
	for f in ICON_FAMILIES:
		fam_labels.append(String(f.label))
	DccWidgets.choice(row, "Family", fam_labels, _icon_family_idx, func(i: int):
		_icon_family_idx = i
		_icon_variant_idx = 0
		_arm_icon_from_ui()
		## Rebuilds the whole row so the Variant list follows the new family --
		## `set_tool_options`' own contract ("replace the bar's contents").
		app.set_tool_options(_build_icon_tool_options_row))

	var slots: Array = ICON_FAMILIES[_icon_family_idx].slots
	var slot_labels: Array = _slot_labels(String(ICON_FAMILIES[_icon_family_idx].key), slots)
	DccWidgets.choice(row, "Variant", slot_labels, _icon_variant_idx, func(i: int):
		_icon_variant_idx = i
		_arm_icon_from_ui())

	DccWidgets.slider(row, "Scale", ICON_SCALE_MIN, ICON_SCALE_MAX, 0.1, _icon_scale, "×", func(v: float):
		_icon_scale = v
		_arm_icon_from_ui())
	DccWidgets.slider(row, "Rotation", -180.0, 180.0, 1.0, _icon_rotation, "°", func(v: float):
		_icon_rotation = v
		_arm_icon_from_ui())
	DccWidgets.slider(row, "Jitter", 0.0, 1.0, 0.05, _icon_jitter, "", func(v: float):
		_icon_jitter = v
		_arm_icon_from_ui())

	_build_icon_brush_controls(row)

	var armed := bridge.icon_armed()
	if not armed.is_empty():
		row.add_child(DccTheme.mono_label(
			"→ %s/%s ×%.2f" % [armed.get("family", ""), armed.get("slot", ""), float(armed.get("scale", 1.0))],
			"text_dim", DccTheme.FS_MICRO))
	row.add_child(DccTheme.spacer())


## The density brush's own three controls -- `carIconBrushChk` and, revealed
## by it, `carIconBrushR`/`carIconBrushD` (reference lines 1654-1657).
##
## **Progressive disclosure is the reference's own**, not an invention here:
## `#carIconBrushOpts` ships `style="display:none"` and its checkbox listener
## (line 13511) is the only thing that shows it. Rebuilding the row is how
## every other structural change in this bar is made -- the Family choice one
## screen up does the same when the Variant list has to follow it.
##
## Drawn only against a cdylib that carries the binding. `icon_brush()` is
## empty both before any world and on an older library, and three sliders that
## silently reach nothing would be exactly the dead control this shell's own
## unwired audit exists to catch. A world is guaranteed by then: the sliders
## write through `icon_brush_set`, which needs the editor `absorb()` creates.
func _build_icon_brush_controls(row: HBoxContainer) -> void:
	if bridge.icon_brush().is_empty():
		return
	DccWidgets.toggle(row, "Brush", _icon_brush_on, func(on: bool):
		_icon_brush_on = on
		_arm_icon_from_ui()
		app.set_tool_options(_build_icon_tool_options_row),
		"Density brush (_carIconBrushStamp): drag to paint a blue-noise stand of the armed icon instead of stamping one per click. Unlike click-placement it never paints into water, and each icon takes its own size from the slot's scatter rule rather than the Scale dial.")
	if not _icon_brush_on:
		return
	DccWidgets.slider(row, "Radius", ICON_BRUSH_R_MIN, ICON_BRUSH_R_MAX, 1.0,
		_icon_brush_r, " cells", func(v: float):
			_icon_brush_r = v
			_arm_icon_from_ui())
	DccWidgets.slider(row, "Density", ICON_BRUSH_DENSITY_MIN, ICON_BRUSH_DENSITY_MAX, 0.05,
		_icon_brush_density, "", func(v: float):
			_icon_brush_density = v
			_arm_icon_from_ui(),
		"How tightly one stamp packs: spacing is max(1.2, 3/sqrt(density)) cells, so this is a floor on separation rather than a count. One stamp is capped at 1500 darts whatever the radius.")


## Pointerdown-on-the-handle-starts-a-resize, a-miss-falls-through-to-place
## precedence (reference lines 9664-9671: `_carIconHitTest` checked before
## the click handler's own place/select branch) -- `IconEditor::handles`
## doesn't require `sel` to already be selected, but the reference's own
## `_iconHandle` is only ever set for `_iconSelected`, so checking it here
## against whatever `icon_get_selected()` currently names reproduces that.
##
## **The handle branch is skipped while a modifier is down.** Shift or Ctrl
## means "change what is selected", and a modified press that landed on the
## handle circle would otherwise start a resize instead -- the one gesture in
## this handler that cannot be undone by clicking again.
##
## **The brush branch is checked before all of that**, which is the
## reference's own precedence and not a choice made here: its pointerdown
## listener tests `_carIconBrush.on` at line 9657 and only reaches the resize/
## select block at 9664 if the brush is off. A consequence worth stating,
## because it looks like a bug from inside the shell: **with the brush on, the
## resize handle is unreachable.** Turning the brush off gets it back.
func _on_icon_click(gx: float, gy: float) -> void:
	var mode := EngineBridge.selection_mode_from_input()
	if _icon_brush_on and mode == EngineBridge.SEL_REPLACE and not bridge.icon_armed().is_empty():
		_icon_brush_painting = true
		if bridge.icon_brush_stamp(gx, gy) > 0:
			app.viewport.refresh_annotations()
			_rebuild_icon_panel()
		return

	var sel := bridge.icon_get_selected()
	if sel >= 0 and mode == EngineBridge.SEL_REPLACE and bridge.icon_get_selection().size() == 1:
		var h: Dictionary = bridge.icon_handles(sel, app.viewport.zoom()).get("resize", {})
		if not h.is_empty() and Vector2(gx, gy).distance_to(Vector2(h["x"], h["y"])) <= float(h["r"]):
			_begin_icon_handle_drag(sel, gx, gy)
			return

	## Hit an existing icon before falling through to place, mirroring
	## `_on_label_click` one section down and the reference's own
	## hit-then-select click sequencing (`_carIconHitTest`, line 9664).
	## Without this branch there was no way to re-select a placed icon at
	## all: every click on one stamped a *duplicate* on top of it, and the
	## resize handle was reachable only for whatever icon happened to be
	## selected by having been placed last. `icon_hit_test` was bound and
	## wrapped from the start and called by nothing until 2026-09-01.
	##
	## Before the asset-pack guard on purpose: selecting an icon that is
	## already on the map does not need a pack loaded, and refusing it
	## there would make an unloaded pack look like a dead map.
	## `IconEditor::hit_test` performs the selection itself, in whichever mode
	## the modifier named (`EngineBridge.SEL_*`): plain click replaces,
	## Ctrl/Cmd adds or removes, Shift takes the range from the last one.
	if bridge.icon_hit_test_mode(gx, gy, mode) >= 0:
		_update_icon_handles_overlay()
		_rebuild_icon_panel()
		return

	## A modified click that hit nothing is a missed selection gesture, not a
	## request to stamp another icon on top of empty ground. Falling through
	## would make Ctrl-click-on-empty place an icon, which is neither what the
	## modifier means anywhere else in this shell nor recoverable in one step.
	if mode != EngineBridge.SEL_REPLACE:
		return

	if not bridge.has_asset_pack():
		app.set_status("hint", "load an asset pack first — File ▸ Import asset pack", "accent")
		return
	var idx := bridge.icon_place(gx, gy)
	if idx < 0:
		return
	app.viewport.refresh_annotations()
	_rebuild_icon_panel()


## Captures the resize drag's fixed reference values -- `_iconResize`'s own
## `{icon,cx,cy,startScale,startDist}` (reference line 9669), mirroring
## `_begin_label_handle_drag`'s pattern one handle down: an icon has no
## rotate/arc, so `IconDragMode` only ever has the one mode to capture, and
## `icon_resize` (unlike `label_resize_size`) already writes the result
## straight into the icon's own `scale` -- no separate `icon_set` commit
## call is needed the way `label_set({"size":...})` is.
func _begin_icon_handle_drag(index: int, gx: float, gy: float) -> void:
	_icon_drag_mode = IconDragMode.RESIZE
	_icon_drag_index = index
	var ic := bridge.icon_get(index)
	_icon_drag_cx = float(ic.get("x", gx)) + 0.5
	_icon_drag_cy = float(ic.get("y", gy)) + 0.5
	_icon_drag_start_dist = maxf(1.0,
		Vector2(gx + 0.5, gy + 0.5).distance_to(Vector2(_icon_drag_cx, _icon_drag_cy)))


func _on_icon_drag(gx: float, gy: float) -> void:
	## The brush's own `pointermove` (reference line 9719), which redraws only
	## when a stamp actually placed something -- `if(_carIconBrushStamp(gx,gy))
	## drawCivLayerAuto()`. The list panel is deliberately NOT rebuilt per
	## sample: a stroke can add dozens of icons across a few dozen moves, and
	## rebuilding a row per icon per sample is the one thing here that would
	## drop frames. `_on_icon_release` rebuilds it once at the end.
	if _icon_brush_painting:
		if bridge.icon_brush_stamp(gx, gy) > 0:
			app.viewport.refresh_annotations()
		return
	if _icon_drag_mode != IconDragMode.RESIZE or _icon_drag_index < 0:
		return
	bridge.icon_resize(_icon_drag_index, _icon_drag_cx, _icon_drag_cy, gx, gy, _icon_drag_start_dist)
	app.viewport.refresh_annotations()
	_update_icon_handles_overlay()


func _on_icon_release(_gx: float, _gy: float, _valid: bool) -> void:
	## Reference line 9739: the stroke ends and the map gets one full render,
	## "so the finished stand composites like any other icon edit".
	if _icon_brush_painting:
		_icon_brush_painting = false
		app.viewport.refresh_annotations()
		_rebuild_icon_panel()
		return
	if _icon_drag_mode != IconDragMode.NONE:
		_icon_drag_mode = IconDragMode.NONE
		_icon_drag_index = -1
		_rebuild_icon_panel()   ## Syncs the list row's own `×scale` readout to the drag's final value.


func _build_icon_panel(parent: Control) -> void:
	## `#carIconList` (`DCC_SHELL_SPEC.md` §4.5.5): "Both keep their list
	## panels ... with counts and Clear-all."
	var sec := DccWidgets.section(parent, "Placed icons")
	_icon_list_body = VBoxContainer.new()
	_icon_list_body.add_theme_constant_override("separation", 2)
	sec.add_child(_icon_list_body)
	_icon_clear_btn = DccWidgets.action(sec, "Clear all icons", func():
		bridge.icon_clear_all()
		app.viewport.refresh_annotations()
		_rebuild_icon_panel())
	DccWidgets.note(sec,
		"Arm the Icon tool above, then click the map to stamp it. Family, "
		+ "variant, scale, rotation and jitter live in the tool options bar "
		+ "while Icon is armed. Placing an icon selects it and shows its own "
		+ "on-canvas resize handle -- drag it to rescale in place; delete and "
		+ "re-place to change family/slot.\n"
		+ "Brush, in the same bar, switches click-to-stamp for drag-to-paint: "
		+ "a stand of the armed icon scattered under the pointer, thinned to a "
		+ "minimum separation and never painted into water. Painting does not "
		+ "select, and while the brush is on the resize handle is out of reach "
		+ "-- switch it off to get the handle back.")


## `GUI_GAP_REGISTER.md` **CA-20**: `DCC_SHELL_SPEC.md` §4.5.5 asks for "counts
## and Clear-all", and both panels shipped the Clear-all without the count and
## without gating it, so the button was live over an empty list -- a press that
## could not change anything, which is the same class of defect as a dead
## binding even though the binding here is real. The count goes on the button
## because it is the button's own subject, and the disabled state carries its
## reason the way every other disclosed gap in this shell does.
##
## Deliberately **not** gated on `bridge.has_world`: a label or icon can only
## exist over a world, so the list count already answers that, and a second
## condition would just be able to disagree with the first.
static func _set_clear_state(btn: Button, noun: String, n: int, why: String) -> void:
	if btn == null or not is_instance_valid(btn):
		return
	btn.text = "Clear all %s" % noun if n == 0 else "Clear all %s (%d)" % [noun, n]
	btn.disabled = n == 0
	btn.tooltip_text = why if n == 0 else "Removes all %d placed %s. Not undoable." % [n, noun]


func _rebuild_icon_panel() -> void:
	if _icon_list_body == null:
		return
	for child in _icon_list_body.get_children():
		_icon_list_body.remove_child(child)
		child.queue_free()
	var list: Array = bridge.icon_list()
	_set_clear_state(_icon_clear_btn, "icons", list.size(),
		"No icons placed yet -- arm the Icon tool above and click the map. "
		+ "There is nothing to clear.")
	if list.is_empty():
		_icon_list_body.add_child(DccTheme.label("none placed", "text_ghost", DccTheme.FS_MICRO))
	## Read once, not per row: the fallback for an older cdylib whose
	## `icon_list()` carries neither `selected` nor `primary`.
	var primary_idx := bridge.icon_get_selected()
	for entry in list:
		var d: Dictionary = entry
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var text := "%s / %s  ·  ×%.2f" % [String(d.get("family", "")), String(d.get("slot", "")), float(d.get("scale", 1.0))]
		var l := DccTheme.mono_label(text, "text_dim", DccTheme.FS_SMALL)
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.clip_text = true
		## `icon_list()` now carries `selected` (in the selection set) and
		## `primary` (the one the resize handle belongs to). The accent is
		## membership, so a Ctrl-click that added a second icon is visible;
		## the primary keeps the brighter `text` weight on top of it. Falls
		## back to the old index comparison against a cdylib whose `icon_list`
		## carries neither key -- which is exactly `primary`.
		var is_primary := bool(d.get("primary", int(d.get("index", -1)) == primary_idx))
		var is_sel := bool(d.get("selected", is_primary))
		if is_sel:
			l.add_theme_color_override("font_color", DccTheme.c("accent" if is_primary else "text"))
		row.add_child(l)
		var idx: int = int(d.get("index", -1))
		var del := Button.new()
		del.text = "×"
		del.tooltip_text = "Delete"
		del.focus_mode = Control.FOCUS_NONE
		del.custom_minimum_size = Vector2(22, 20)
		del.pressed.connect(func():
			bridge.icon_delete(idx)
			app.viewport.refresh_annotations()
			_rebuild_icon_panel())
		row.add_child(del)
		_icon_list_body.add_child(row)
	_update_icon_handles_overlay()
	_refresh_right_dock_anno()


## Draws the selected icon's resize handle (`icon_handles`,
## `GUI_GAP_REGISTER.md` CA-05) through the same `tool_overlay.gd` primitive
## the Label tool's own handles already use -- `_update_label_handles_
## overlay`'s one-handle mirror.
##
## **Drawn only for a selection of exactly one.** The handle rescales one
## icon's own `scale` from a baseline snapshotted when it became the primary
## (`IconEditor::resize` refuses any index that is not the primary), so on a
## multi-selection there is no honest single circle to draw: one that acted on
## the primary alone would be a handle pointing at part of what is highlighted.
## Handles on a multi-selection are a design question this step deliberately
## does not answer -- and clearing them costs the single-select behaviour
## nothing, since a set of one is exactly what it always had.
func _update_icon_handles_overlay() -> void:
	var idx := bridge.icon_get_selected()
	if idx < 0 or bridge.icon_get_selection().size() != 1:
		app.viewport.tool_overlay.set_handles([])
		return
	var h: Dictionary = bridge.icon_handles(idx, app.viewport.zoom()).get("resize", {})
	app.viewport.tool_overlay.set_handles([h] if not h.is_empty() else [])


# ===========================================================================
# Label tool (§4.5.5)
# ===========================================================================

## The handle branch is skipped while a modifier is down, and while more than
## one label is selected -- `_on_icon_click`'s own note on why, plus: the three
## handles resize/rotate/arc **one** label's geometry, and applying them to a
## set is a design question, not a free consequence of holding one.
func _on_label_click(gx: float, gy: float) -> void:
	var sel_mode := EngineBridge.selection_mode_from_input()
	var sel := bridge.label_get_selected()
	if sel >= 0 and sel_mode == EngineBridge.SEL_REPLACE and bridge.label_get_selection().size() == 1:
		var mode := _handle_hit(sel, gx, gy)
		if mode != DragMode.NONE:
			_begin_label_handle_drag(sel, mode, gx, gy)
			return

	var hit := bridge.label_hit_test_mode(gx, gy, sel_mode)
	if hit >= 0:
		## A modified click selects; it does not also arm a position drag. A
		## Ctrl-drag that moved the label it had just added to the set would
		## make the two gestures fight over one press.
		_label_drag_mode = DragMode.MOVE if sel_mode == EngineBridge.SEL_REPLACE else DragMode.NONE
		_label_drag_index = hit if sel_mode == EngineBridge.SEL_REPLACE else -1
		app.set_tool_options(_build_label_tool_options_row)
		_rebuild_label_panel()
	else:
		_label_drag_mode = DragMode.NONE
		_label_drag_index = -1
		## A modified click on empty ground missed a selection; it is not a
		## request for the New label dialog (`_on_icon_click`'s own reasoning
		## against falling through to place).
		if sel_mode == EngineBridge.SEL_REPLACE:
			_prompt_label_name(gx, gy)


## Grid-space hit test against the currently-selected label's own three
## handle circles (`label_handles`), checked *before* the plain box hit test
## above -- matching the reference's own load-bearing test order (resize,
## then rotate, then arc, then the label boxes; `labels.rs`'s own
## `LabelHandles` doc comment). Each handle's `x`/`y`/`r` are already in the
## same grid-coordinate space `gx`/`gy` live in (`label_handles`'s own doc
## comment), so no offset is needed here -- only the drag-math calls below
## need the `+0.5` cell-centred `cx`/`cy`.
func _handle_hit(index: int, gx: float, gy: float) -> int:
	var h := bridge.label_handles(index, app.viewport.zoom())
	if h.is_empty():
		return DragMode.NONE
	for pair in [["resize", DragMode.RESIZE], ["rotate", DragMode.ROTATE], ["arc", DragMode.ARC]]:
		var hd: Dictionary = h.get(pair[0], {})
		if hd.is_empty():
			continue
		if Vector2(gx, gy).distance_to(Vector2(hd["x"], hd["y"])) <= float(hd["r"]):
			return pair[1]
	return DragMode.NONE


## Captures the drag's fixed reference values -- `label_resize_size`/
## `label_rotate_deg`/`label_arc_value` all take a `cx`/`cy` and internally
## add their own `+0.5` to whatever `gx`/`gy` they're later called with
## (`labels.rs`'s `wx = gx + 0.5 - cx`), because `label_box`'s own `px`/`py`
## are `lb.x + 0.5`/`lb.y + 0.5`, not the raw stored position
## (`label_box`'s own doc comment). `cx`/`cy` here reproduce that offset so
## the drag math lines up with the same geometry `label_handles` drew.
func _begin_label_handle_drag(index: int, mode: int, gx: float, gy: float) -> void:
	_label_drag_mode = mode
	_label_drag_index = index
	var lb := bridge.label_get(index)
	_label_drag_cx = float(lb.get("x", gx)) + 0.5
	_label_drag_cy = float(lb.get("y", gy)) + 0.5
	match mode:
		DragMode.RESIZE:
			_label_drag_start_size = float(lb.get("size", 16.0))
			_label_drag_start_dist = maxf(1.0,
				Vector2(gx + 0.5, gy + 0.5).distance_to(Vector2(_label_drag_cx, _label_drag_cy)))
		DragMode.ARC:
			_label_drag_grab_angle = float(lb.get("angle", 0.0))
			_label_drag_side = _label_side_from_handles(index)
		_:
			pass


## The label box's own `side` isn't a field `label_get`/`label_handles`
## expose directly, but `handle_circles`' resize handle sits at local
## `(side/2, side/2)` regardless of the label's rotation, so its distance
## from the box centre is always `side/2 * sqrt(2)` -- solved back out here
## rather than adding a new Rust accessor for one derived number.
func _label_side_from_handles(index: int) -> float:
	var h := bridge.label_handles(index, app.viewport.zoom())
	var resize_h: Dictionary = h.get("resize", {})
	if resize_h.is_empty():
		return 40.0
	return Vector2(resize_h["x"], resize_h["y"]).distance_to(Vector2(_label_drag_cx, _label_drag_cy)) * sqrt(2.0)


func _on_label_drag(gx: float, gy: float) -> void:
	if _label_drag_index < 0:
		return
	match _label_drag_mode:
		DragMode.MOVE:
			bridge.label_move(_label_drag_index, gx, gy)
		DragMode.RESIZE:
			var s := bridge.label_resize_size(
				_label_drag_start_size, _label_drag_cx, _label_drag_cy, gx, gy, _label_drag_start_dist)
			bridge.label_set(_label_drag_index, {"size": s})
		DragMode.ROTATE:
			var deg := bridge.label_rotate_deg(_label_drag_cx, _label_drag_cy, gx, gy)
			bridge.label_set(_label_drag_index, {"angle": deg})
		DragMode.ARC:
			var a := bridge.label_arc_value(
				_label_drag_cx, _label_drag_cy, _label_drag_grab_angle, _label_drag_side, gx, gy)
			bridge.label_set(_label_drag_index, {"arc": a})
		_:
			return
	app.viewport.refresh_annotations()
	_update_label_handles_overlay()


func _on_label_release(_gx: float, _gy: float, _valid: bool) -> void:
	if _label_drag_mode != DragMode.NONE:
		_label_drag_mode = DragMode.NONE
		_label_drag_index = -1
		_rebuild_label_panel()   ## Syncs the dock's own field readouts to the drag's final values.


func _apply_label_field(idx: int, key: String, value) -> void:
	if idx < 0:
		return
	bridge.label_set(idx, {key: value})
	app.viewport.refresh_annotations()
	_update_label_handles_overlay()


func _confirm_label_edit() -> void:
	bridge.label_confirm_edit()
	app.viewport.refresh_annotations()
	app.set_tool_options(_build_label_tool_options_row)
	_rebuild_label_panel()


func _cancel_label_edit() -> void:
	bridge.label_cancel_edit()
	app.viewport.refresh_annotations()
	app.set_tool_options(_build_label_tool_options_row)
	_rebuild_label_panel()


## Reference's own click-on-empty-ground branch: prompt for a name, then
## `label_create` (which selects the new label itself). No existing
## LineEdit-based prompt pattern exists elsewhere in `shell/` to reuse
## (`new_world_dialog.gd` is a full multi-section form, not a one-field
## prompt) -- this is the minimal `AcceptDialog` + `LineEdit` shape instead.
## Added to `app`, not `self`: this workspace panel is a dock body, not a
## viewport root, and `app.gd`'s own `_pick_file`/`open_new_world` already
## establish that popups belong on the app root.
func _prompt_label_name(gx: float, gy: float) -> void:
	var dlg := AcceptDialog.new()
	dlg.title = "New label"
	dlg.get_ok_button().text = "Create"
	dlg.min_size = Vector2i(320, 0)

	var margin := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 12)
	var edit := LineEdit.new()
	edit.placeholder_text = "Region name"
	edit.custom_minimum_size = Vector2(260, 0)
	margin.add_child(edit)
	dlg.add_child(margin)
	app.add_child(dlg)

	var create := func():
		var idx := bridge.label_create(gx, gy, edit.text)
		if idx >= 0:
			app.viewport.refresh_annotations()
			app.set_tool_options(_build_label_tool_options_row)
			_rebuild_label_panel()
		dlg.queue_free()
	edit.text_submitted.connect(func(_t: String): create.call())
	dlg.confirmed.connect(create)
	dlg.canceled.connect(func(): dlg.queue_free())
	dlg.popup_centered()
	edit.grab_focus()


## Tool options row: `CARTO · LABEL` -- per §4.5.5's table this carries
## text/size-mode/arc/letter-spacing/anchor/font-role for whichever label is
## selected. The full seven-field form (plus Confirm/Cancel) lives in the
## dock below; this row is the fast-glance summary plus the two commit
## actions, matching how the spec's own right-dock description already
## overlaps the toolbar row's fields.
func _build_label_tool_options_row(row: HBoxContainer) -> void:
	row.add_child(DccTheme.mono_label("CARTO · LABEL", "accent", DccTheme.FS_SMALL, 2, true))
	var idx := bridge.label_get_selected()
	if idx < 0:
		row.add_child(DccTheme.label(
			"click empty ground to create a label, or an existing one to edit — full form in the dock below",
			"text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		return
	var lb := bridge.label_get(idx)
	row.add_child(DccTheme.mono_label("editing #%d" % idx, "text_dim", DccTheme.FS_SMALL))
	row.add_child(DccTheme.label(String(lb.get("text", "")), "text_faint", DccTheme.FS_MICRO))
	DccWidgets.action(row, "Confirm", _confirm_label_edit, true)
	DccWidgets.action(row, "Cancel", _cancel_label_edit)
	row.add_child(DccTheme.spacer())



# -- The ENV prototype's two new CARTO panels (stage 2, 2026-08-31) -----------
#
# BUILD_ANSWERS §2.1 turned CARTO's four rail nodes into four real destinations
# and named what two of them hold: "LABELS and ICONS are new and real". They are
# new *to the design*; what follows is the whole of what the prototype specifies
# for each, drawn here in full and **disabled, with the reason on the control**,
# because this port's engine has no call behind any of it.
#
# The disclosure is not a hedge. `cartalith-civ::labels::MapLabel` is one
# hand-placed label -- text, position, size, arc, angle, font, colour
# (`lib.rs:7782`'s `label_dict`) -- and `label_create` is the only way one comes
# into existence (`lib.rs:7826`).
#
# **The Icons half stopped being true too, and is replaced here rather than
# softened (2026-09-03).** It read: "Icons are still that story: `icon_bridge.rs`
# arms a family/slot for the Icon tool to *stamp*, and every icon on the map got
# there by a click. There is no generated-icon pass for a minimum spacing or a
# placement rule to constrain, so `_build_icon_placement()` below is unchanged
# and still drawn disabled." Both clauses are now false. The owner's 2026-09-02
# ruling built the pass -- `icon_bridge/generate.rs`, `IconEditor::generate`,
# `PlacementFamily`'s four families and a sea-marks asset family -- and
# `_build_icon_placement()` is bound to it through `_run_icon_placement()`, with
# the spacing slider and both cull toggles live. And a click is no longer the
# only manual route either: the density brush (`icon_bridge/brush.rs`) paints a
# stand of the armed icon under a drag.
#
# **The Labels half of this paragraph is no longer true, and was replaced
# rather than softened (2026-09-02).** It read: "There is no label *class*, no
# automatic labelling pass to assign one, no `halo` or `tracking` field to set,
# and no collision test." `LARGE_ITEM_RULINGS.md`'s owner ruling built the first
# three -- `MapLabel::class`, `labels::generate_labels`, and `LabelTypography`
# carrying size/halo/tracking -- and `_build_label_classes()` is bound to them.
# The fourth is still absent, deliberately: the same ruling sequences collision
# culling *behind* this pass ("culling a set nothing generates is half a
# feature"), so the toggle stays disabled and now says the narrower true thing.
#
# So the choice was between omitting these panels, faking them against local
# state that reaches nothing, and drawing them disabled with their reason. The
# house rule settles it: "a control with nothing behind it is a defect: draw it
# disabled WITH its reason." A faked slider is worse than an absent one, because
# a user who moves it and sees no change learns that the *map* is broken.
#
# The design's own default values are kept on the disabled controls rather than
# zeroed, because they are real design values (`parts.js:395`'s `LABD()` and
# `:398`'s `ICOD()`) and the next pass, the one that binds these, should not have
# to re-read the prototype to find them.

## The five label classes and their typography (`ENV:698`-`721`,
## `parts.js:363`/`:376`-`:387`).
##
## `CL` in `parts.js:363` is `[id, label, swatch, spec, count]` and this is that
## array transcribed. The `spec` column is the prototype's own compact notation:
## `26/2.5 · .28 em` reads size / halo / tracking, which is exactly the three
## sliders below it, so the row doubles as the class's own summary.
##
## **Now the fallback, not the source.** `bridge.label_class_table()` serves the
## same five specs from `cartalith_civ::labels::LABEL_TYPOGRAPHY_DEFAULTS`, and
## `_label_class_specs_from_engine()` prefers it; this array is what a cdylib
## predating that binding gets. The two are kept identical on purpose, and the
## Rust side is the one with a test pinning them.
##
## **The counts are still not transcribed.** `parts.js:363`'s
## `4 · 11 · 48 · 22 · 37` and `:372`'s `122 drawn · 9 culled` are the
## prototype's mock data over its mock world. The counts drawn in the dock are
## the real ones, from `labels_generated_counts()`, and read `--` until the pass
## has actually run over this world.
const LABEL_CLASSES: Array = [
	{"id": "continental", "label": "Continental", "swatch": "#e0a34a",
		"spec": "26/2.5 · .28 em", "size": 26.0, "halo": 2.5, "track": 0.28},
	{"id": "region", "label": "Region", "swatch": "#c8cbcd",
		"spec": "18/2 · .20 em", "size": 18.0, "halo": 2.0, "track": 0.20},
	{"id": "settlement", "label": "Settlement", "swatch": "#a9adb0",
		"spec": "13/1.5 · .06 em", "size": 13.0, "halo": 1.5, "track": 0.06},
	{"id": "water", "label": "Water", "swatch": "#6f9fb5",
		"spec": "15/1.5 · .14 em italic", "size": 15.0, "halo": 1.5, "track": 0.14},
	{"id": "landmark", "label": "Landmark", "swatch": "#8d9296",
		"spec": "11/1.2 · .06 em", "size": 11.0, "halo": 1.2, "track": 0.06},
]

## The prototype's own slider domains, read off the inverse maps in
## `parts.js:383`-`:385`: `size` is `Math.round(8+p*26)` so 8-34 px, `halo` is
## `p*4` so 0-4 px, `track` is `p*0.4` so 0-0.40 em. Stated as constants because
## a range is a design value like any other and guessing one later would be
## guessing at a design value.
const LABEL_SIZE_RANGE := Vector2(8.0, 34.0)
const LABEL_HALO_RANGE := Vector2(0.0, 4.0)
const LABEL_TRACK_RANGE := Vector2(0.0, 0.40)

## Which class the panel's three sliders describe. `parts.js:395`'s
## `LABD().sel` is `'settlement'`, and the design's own fallback when `sel`
## matches nothing is `CL[2]` -- also settlement (`parts.js:378`).
var _label_class := "settlement"
var _label_class_rows: Dictionary = {}     ## key -> the row's name Label.
var _label_class_count_cells: Dictionary = {}  ## key -> the row's count Label.
var _label_class_title: Label
var _label_class_fields: Array = []        ## The three `DccWidgets.slider()` dicts.

## The five type specs currently in force, in engine order. Each entry is
## `label_class_table()`'s own row shape (`key`, `label`, `size`, `halo`,
## `tracking`, `italic`, `ink`), and this array is what
## `_regenerate_labels()` pushes back to the engine.
##
## **The engine is the source of these, not this file.** `LABEL_CLASSES` above
## survives only as the fallback for a cdylib that predates
## `label_class_table()`, which is why its literals are still there and still
## match: `cartalith_civ::labels::LABEL_TYPOGRAPHY_DEFAULTS` carries the same
## five specs and is pinned digit for digit by its own test.
var _label_class_specs: Array = []
## `key -> {available, drawn, over_cap, suppressed}` from the last run. Empty
## before the first one, which is what makes the count column read `--`.
var _label_class_counts: Dictionary = {}
var _label_gen_ran := false
var _label_class_summary: Label

## The `collision culling` toggle's state.
##
## **This declaration was missing**, and its absence took the whole file down:
## `_label_cull` is read and written by the toggle built in
## `_build_label_class_panel` and was declared nowhere, so
## `cartography_workspace.gd` failed to load with *"Parse Error: Identifier
## `_label_cull` not declared in the current scope"* -- the entire CARTO
## workspace, not just the label block. Found 2026-09-02 by
## `godot --headless --check-only`, at `0f0fe55` and not before it.
##
## `true` because that is where the engine starts (`LabelBridge::new` turns
## culling on at the shell's boundary where `LabelGenSettings::default()` has it
## off) and where the design draws it (`parts.js:387`).
var _label_cull := true

## `label_class_table()`'s rows, or this file's own transcription of the same
## design values when the binding is absent.
func _label_class_specs_from_engine() -> Array:
	var table: Dictionary = bridge.label_class_table()
	var rows: Array = table.get("classes", [])
	if not rows.is_empty():
		return rows.duplicate(true)
	return LABEL_CLASSES.map(func(c: Dictionary) -> Dictionary:
		return {"key": c["id"], "label": c["label"], "size": c["size"],
			"halo": c["halo"], "tracking": c["track"], "italic": c["id"] == "water",
			"ink": c["swatch"]})

## `parts.js:363`'s own compact notation, rebuilt from the live numbers rather
## than carried as a frozen string -- a dial the user moved has to show in the
## row it belongs to, or the list and the dials disagree on screen.
func _label_spec_text(spec: Dictionary) -> String:
	var size := float(spec.get("size", 0.0))
	var halo := float(spec.get("halo", 0.0))
	var track := float(spec.get("tracking", 0.0))
	var size_s := "%d" % int(round(size))
	var halo_s := ("%d" % int(round(halo))) if is_equal_approx(halo, round(halo)) else ("%.1f" % halo)
	var out := "%s/%s · %s em" % [size_s, halo_s, String("%.2f" % track).trim_prefix("0")]
	return out + " italic" if bool(spec.get("italic", false)) else out

## One `[min, max]` pair out of `label_class_table()`, or `fallback` when the
## binding is absent or the pair is malformed. A range is two numbers or it is
## not a range -- a one-element array would otherwise become a slider whose
## maximum is whatever `[1]` returned.
func _label_range(table: Dictionary, key: String, fallback: Vector2) -> Vector2:
	var r: Array = table.get(key, [])
	if r.size() != 2:
		return fallback
	return Vector2(float(r[0]), float(r[1]))

func _label_class_spec(key: String) -> Dictionary:
	for entry in _label_class_specs:
		var d: Dictionary = entry
		if String(d.get("key", "")) == key:
			return d
	return {}


func _build_label_classes(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Label classes")
	_label_class_specs = _label_class_specs_from_engine()
	DccWidgets.note(sec,
		"The engine places these. Continent, province, settlement, lake and "
		+ "landmark names are generated from the world's own features and styled "
		+ "per class; the counts on the right are what the last run actually "
		+ "drew. Moving a dial re-runs the pass when you let go of it. Region "
		+ "labels you place by hand are the section below and are never replaced "
		+ "by a run -- they take their class's halo and tracking, and keep their "
		+ "own size, font and colour.")

	var rows := DccWidgets.group(sec, "Classes", true)
	for entry in _label_class_specs:
		var cl: Dictionary = entry
		var key := String(cl.get("key", ""))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		row.custom_minimum_size.y = 22
		## `width:11px;height:11px;border-radius:3px` (`ENV:702`). A `ColorRect`
		## rather than a themed swatch: these five colours are the design's own
		## literals and are NOT tokens -- `#a9adb0` and `#6f9fb5` appear nowhere
		## in `DccTheme.PALETTE`, and routing them through `c()` would silently
		## substitute the nearest token. They now arrive from the engine's own
		## `LabelTypography::ink`, which is also what a generated label is drawn
		## in, so the swatch and the map cannot disagree.
		var sw := ColorRect.new()
		sw.color = Color(String(cl.get("ink", "#ffffff")))
		sw.custom_minimum_size = Vector2(11, 11)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		row.add_child(sw)
		var name_l := DccTheme.label(String(cl.get("label", key)), "text", DccTheme.FS_SMALL)
		name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(name_l)
		var spec_l := DccTheme.mono_label(_label_spec_text(cl), "text_dim", DccTheme.FS_MICRO)
		row.add_child(spec_l)
		## The drawn-count column (`ENV:706`, `width:44px;text-align:right`).
		var count := DccTheme.mono_label("--", "text_ghost", DccTheme.FS_MICRO)
		count.custom_minimum_size.x = 44
		count.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		row.add_child(count)
		rows.add_child(row)
		_label_class_rows[key] = name_l
		_label_class_count_cells[key] = {"count": count, "spec": spec_l}

	## One line under the list rather than a per-row second number: the design
	## draws a single summary (`parts.js:372`, "122 drawn · 9 culled") and the
	## rows themselves stay one number wide.
	_label_class_summary = DccTheme.mono_label("", "text_faint", DccTheme.FS_MICRO, 2)
	## DS-03. This slot carries two very different strings: the one-line
	## "122 drawn - 9 culled - N ms" the design draws, and -- when there is no
	## world yet -- the engine's own `reason`, which is a 155-character
	## sentence (`label_bridge/generate.rs`). Without autowrap a `Label`'s
	## minimum width is that whole sentence: measured 1 546 px inside a 400 px
	## dock, and because the dock's `ScrollContainer` disables its horizontal
	## axis that minimum propagated outward and grew the left dock to 1 589 px,
	## eating 1 189 px of the map. Its twin two hundred lines below,
	## `_icon_gen_summary`, is filled from the same `res["reason"]` shape and
	## has always wrapped; this one was simply missed.
	_label_class_summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	rows.add_child(_label_class_summary)

	## `labSelTitle` (`parts.js:378`): the selected class's own name plus
	## ` · TYPE`.
	var fields := DccWidgets.group(sec, "Type", true)
	_label_class_title = DccTheme.mono_label("", "text_faint", DccTheme.FS_MICRO, 2)
	fields.add_child(_label_class_title)
	DccWidgets.choice(fields, "Class",
		_label_class_specs.map(func(c: Dictionary) -> String: return String(c.get("label", ""))),
		_label_class_index(_label_class),
		func(i: int): _set_label_class(String((_label_class_specs[i] as Dictionary).get("key", ""))),
		"Which class the three dials below describe.")

	## The domains come from the engine too (`LABEL_CLASS_*_RANGE`), so a value
	## the engine would clamp cannot be reachable on the dial that sends it.
	## The three constants above are the fallback for a cdylib without the
	## binding, exactly as `LABEL_CLASSES` is for the specs.
	var table: Dictionary = bridge.label_class_table()
	var cl0: Dictionary = _label_class_spec(_label_class)
	_label_class_fields = [
		_label_class_dial(fields, "size", "size", _label_range(table, "size_range", LABEL_SIZE_RANGE),
			1.0, float(cl0.get("size", 13.0)), " px"),
		_label_class_dial(fields, "halo", "halo", _label_range(table, "halo_range", LABEL_HALO_RANGE),
			0.1, float(cl0.get("halo", 1.5)), " px"),
		_label_class_dial(fields, "tracking", "tracking", _label_range(table, "tracking_range", LABEL_TRACK_RANGE),
			0.01, float(cl0.get("tracking", 0.06)), " em"),
	]

	## `hLabColl` / `labCollNote` (`parts.js:387`-`:389`). **Live.** The pass
	## measures every label's box and suppresses one that lands on a label
	## already placed; what it suppresses is counted, not silently dropped, and
	## the summary line below the class list is where the count goes.
	DccWidgets.toggle(fields, "collision culling", _label_cull,
		func(on: bool):
			_label_cull = on
			_regenerate_labels(),
		"On, a label whose box lands on one already placed is suppressed and counted below. Off, every candidate is drawn and names overlap.")
	## Two sentences, and the second one is the one that has to be there.
	##
	## Boxes are ESTIMATED. The engine has no font -- glyph advances belong to
	## the loaded face, which is the seam `cartalith-civ/src/labels.rs`'s header
	## has always drawn -- so a box is `size * (glyphs * mean advance +
	## tracking) `, with the mean advance measured off this shell's own font by
	## `_label_advance_ratio()` and sent with every run. That is a good estimate
	## and it is not a measurement of the actual string, so a name one or two
	## glyphs wider than the mean can still touch its neighbour. Saying so is
	## cheaper than a user discovering it and concluding the toggle is broken.
	DccWidgets.note(fields,
		"Suppression is by rank: within a class the heavier feature keeps its "
		+ "name, and a bigger class wins over a smaller one under it. Labels "
		+ "you placed by hand are never suppressed -- they take space from the "
		+ "pass, not the other way round.\n"
		+ "Boxes are estimated from this font's average glyph width, not "
		+ "measured per name, so an unusually wide name can still touch its "
		+ "neighbour.")
	_sync_label_class()


## The mean glyph advance of the font this shell draws with, as a fraction of
## the font size -- the one number `labels::label_cull_rect` cannot work out for
## itself, and the reason its boxes are an estimate rather than a guess.
##
## Measured once and cached: `get_string_size` over a fixed probe string at a
## large size, divided by `size * length`. Large deliberately -- at 12 px the
## per-glyph rounding in the returned width is a percent or two of the answer.
##
## `get_theme_default_font()` is the same call `map_overlay.gd` draws labels
## with, so this measures the face that will actually be on screen. A null font
## (no theme yet) falls back to the engine's own shipped estimate by sending
## nothing.
const LABEL_ADVANCE_PROBE := "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
var _label_advance_ratio_cache := 0.0

func _label_advance_ratio() -> float:
	if _label_advance_ratio_cache > 0.0:
		return _label_advance_ratio_cache
	var font := get_theme_default_font()
	if font == null:
		return 0.0
	var px := 64
	var w := font.get_string_size(LABEL_ADVANCE_PROBE, HORIZONTAL_ALIGNMENT_LEFT, -1, px).x
	if w <= 0.0:
		return 0.0
	_label_advance_ratio_cache = w / (float(px) * float(LABEL_ADVANCE_PROBE.length()))
	return _label_advance_ratio_cache


## One live class dial. `on_change` writes the spec and repaints the row's own
## `26/2.5 · .28 em` summary; the re-run is deferred to `on_release`.
##
## **The split is the whole reason this is not a plain `slider()` call.** The
## pass sweeps every named feature in the world and, for the Water class, runs a
## connected-component fill over the whole `build_water_bodies` raster
## (`labels::lake_features`). Re-running that on every `value_changed` sample of
## a drag would put an O(gw*gh) pass on a per-frame path.
func _label_class_dial(parent: Control, label_text: String, field: String,
		range_: Vector2, step: float, value: float, unit: String) -> Dictionary:
	return DccWidgets.slider(parent, label_text, range_.x, range_.y, step, value, unit,
		func(v: float):
			var spec := _label_class_spec(_label_class)
			if not spec.is_empty():
				spec[field] = v
				_refresh_label_class_row(_label_class),
		"Applies to every label of this class, generated or hand-placed. Released, not dragged: letting go re-runs the labelling pass.",
		_regenerate_labels)


func _label_class_index(id: String) -> int:
	for i in _label_class_specs.size():
		if String((_label_class_specs[i] as Dictionary).get("key", "")) == id:
			return i
	return 2   ## `parts.js:378`'s own fallback -- `CL[2]`, settlement.

func _set_label_class(id: String) -> void:
	_label_class = id
	_sync_label_class()

## Repaint the class list's ink and re-seat the three dials on the newly
## selected class's current values.
##
## The dials are live now, so `s.value = ...` fires `value_changed` and with it
## the `on_change` above -- which would write the *incoming* class's number onto
## whichever spec `_label_class` names. Since `_label_class` is already the new
## class by the time this runs, that write is a no-op re-assignment of the value
## being seated, not a cross-class leak. It is still worth knowing, which is why
## it is written down rather than defended with a re-entrancy guard nothing else
## in this file uses.
func _sync_label_class() -> void:
	var cl := _label_class_spec(_label_class)
	if cl.is_empty():
		return
	for key in _label_class_rows:
		var l: Label = _label_class_rows[key]
		if is_instance_valid(l):
			l.add_theme_color_override("font_color",
				DccTheme.c("accent") if key == _label_class else DccTheme.c("text"))
	if _label_class_title != null and is_instance_valid(_label_class_title):
		_label_class_title.text = "%s · TYPE" % String(cl.get("label", "")).to_upper()
	if _label_class_fields.size() == 3:
		for pair in [[0, "size"], [1, "halo"], [2, "tracking"]]:
			var d: Dictionary = _label_class_fields[int(pair[0])]
			var s: HSlider = d["slider"]
			if is_instance_valid(s):
				s.value = float(cl.get(String(pair[1]), s.value))
				(d["readout"] as Label).text = (d["format"] as Callable).call(s.value)


## Repaint one class row's spec string and drawn count.
func _refresh_label_class_row(key: String) -> void:
	var cells: Dictionary = _label_class_count_cells.get(key, {})
	if cells.is_empty():
		return
	var spec_l: Label = cells["spec"]
	if is_instance_valid(spec_l):
		spec_l.text = _label_spec_text(_label_class_spec(key))
	var count_l: Label = cells["count"]
	if not is_instance_valid(count_l):
		return
	if not _label_gen_ran:
		count_l.text = "--"
		count_l.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		count_l.tooltip_text = "The labelling pass has not run over this world yet. Generate or open a world, or move a dial, and this becomes a count."
		return
	var c: Dictionary = _label_class_counts.get(key, {})
	var drawn := int(c.get("drawn", 0))
	var available := int(c.get("available", 0))
	count_l.text = "%d" % drawn
	count_l.add_theme_color_override("font_color",
		DccTheme.c("text") if drawn > 0 else DccTheme.c("text_ghost"))
	## Zero drawn is a real answer and the tooltip has to say which kind it is:
	## a world with no lakes over the floor and a world whose lakes were all
	## capped away are different situations.
	if available == 0:
		count_l.tooltip_text = "Nothing in this world to label at this class."
	elif drawn < available:
		count_l.tooltip_text = "%d drawn of %d available; %d over the cap." % [drawn, available, available - drawn]
	else:
		count_l.tooltip_text = "%d drawn, every candidate this class found." % drawn


## Push the five specs to the engine, run the pass, and repaint what it said.
##
## Called from `_on_world_changed` (the world's features have all been replaced)
## and from any class dial's release. Never from a drag sample -- see
## `_label_class_dial`.
func _regenerate_labels() -> void:
	var typography := {}
	for entry in _label_class_specs:
		var d: Dictionary = entry
		typography[String(d.get("key", ""))] = {
			"size": float(d.get("size", 0.0)),
			"halo": float(d.get("halo", 0.0)),
			"tracking": float(d.get("tracking", 0.0)),
		}
	## **`cull` was not being sent**, so the `collision culling` toggle moved a
	## variable and nothing else: `labels_generate`'s own three-state fold reads
	## an absent `cull` key as "keep whatever the last run used", and the last
	## run was always `LabelBridge::new`'s on-by-default. The toggle read as
	## live, was not, and neither was the font measurement --
	## `_label_advance_ratio()` was written, documented three lines above as
	## "sent with every run", and called from nowhere. Both fixed here, in the
	## one call that was always meant to carry them.
	var res: Dictionary = bridge.labels_generate({
		"typography": typography,
		"cull": {"on": _label_cull, "advance_ratio": _label_advance_ratio()},
	})
	_label_class_counts.clear()
	_label_gen_ran = bool(res.get("ok", false))
	for entry in res.get("classes", []):
		var c: Dictionary = entry
		_label_class_counts[String(c.get("key", ""))] = c
	for key in _label_class_count_cells:
		_refresh_label_class_row(String(key))
	if _label_class_summary != null and is_instance_valid(_label_class_summary):
		if not _label_gen_ran:
			## The engine's own sentence, not a rewrite of it -- it names the
			## precondition, and paraphrasing it here would be a second copy to
			## drift.
			_label_class_summary.text = String(res.get("reason", ""))
		else:
			var total := int(res.get("total", 0))
			var suppressed := 0
			for key2 in _label_class_counts:
				suppressed += int((_label_class_counts[key2] as Dictionary).get("suppressed", 0))
			_label_class_summary.text = "%d drawn · %d culled · %d ms" % [
				total, suppressed, int(res.get("elapsed_ms", 0))]
	app.viewport.refresh_annotations()


## The four placement families -- **read from the engine now, not transcribed.**
##
## This used to be `parts.js:364`'s `FAM` table copied in by hand, above a
## comment stating why it could not be wired: *"SEA MARKS has no counterpart in
## the engine's three families at all. Mapping one onto the other would be
## inventing a correspondence the design does not state."* Owner ruling
## 2026-09-02 answered that by building the missing family rather than the
## mapping (`cartalith_assets::slots::Family::SeaMark`,
## `PACK_SEAMARK_SLOTS`), so the design's four placement families are now four
## real engine families and `bridge.icon_placement_families()` is the table.
##
## The two lists still answer different questions and are still not reconciled:
## `ICON_FAMILIES` above is the *arming* vocabulary the Icon tool stamps from,
## indexed positionally by `icon_bridge::resolve_variant`; this one is the
## *placement* vocabulary a generated pass runs over. TREES is the clearest
## case -- it is the five `tree_*` slots, not the whole `icons` art family that
## also holds mountains and boulders.
##
## The design's own `[filled, total]` pairs are gone with the transcription,
## and deliberately: they described the design's art, `filled` was never a
## measurement of anything in this project, and the engine now answers both
## halves for the pack actually loaded.
var _icon_placement_rows: Array = []

## `parts.js:398`'s `ICOD()` defaults and `:391`-`:392`'s inverse maps:
## `scale` is `0.5+p*1.5` so 0.50-2.00, `spacing` is `p*40` so 0-40 px.
const ICON_SCALE_RANGE := Vector2(0.5, 2.0)
const ICON_SPACING_RANGE := Vector2(0.0, 40.0)

var _icon_placement_family := "PLACES"
var _icon_family_chips: Dictionary = {}   ## id -> Button
var _icon_slot_line: Label
var _icon_gen_summary: Label
var _icon_gen_scale := 1.0
var _icon_gen_spacing := 14.0
var _icon_gen_avoid_labels := true
var _icon_gen_enforce_spacing := true
var _icon_gen_snap_coast := false

func _build_icon_placement(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Automatic placement")
	_icon_placement_rows = bridge.icon_placement_families()
	if _icon_placement_rows.is_empty():
		## An older cdylib. Say which half is missing rather than falling back
		## to the design figures this block used to print: a number nobody can
		## check is worse than no number.
		DccWidgets.note(sec,
			"This build's engine has no generated placement pass "
			+ "(icon_placement_families is absent), so the controls below are "
			+ "drawn inert. Rebuild the extension to use it.")
	else:
		DccWidgets.note(sec,
			"Places a whole family at once: the engine picks the cells, then "
			+ "thins what it picked against the rules below. Each family draws "
			+ "from its own source -- PLACES from the settlements, TREES from "
			+ "the biome scatter rules, SEA MARKS from a sweep of the water, "
			+ "POI from the landmarks -- so a family with no source in this "
			+ "world places nothing and says so. Generated icons join the same "
			+ "list hand-placed ones are in: select, resize and delete them the "
			+ "same way, and Clear all removes both. Running it twice over one "
			+ "world places nothing the second time.")

	var fam := DccWidgets.group(sec, "Family", true)
	var chips := HBoxContainer.new()
	chips.add_theme_constant_override("separation", 4)
	fam.add_child(chips)
	for entry in _icon_placement_rows:
		var f: Dictionary = entry
		var id := String(f.get("key", ""))
		var b := DccWidgets.segment(chips, id, func(): _set_icon_placement_family(id))
		b.tooltip_text = "Place the %s family. Draws from the %s asset family." % [id, String(f.get("family", ""))]
		_icon_family_chips[id] = b
	if _icon_placement_rows.is_empty():
		_mark_inert(chips)

	## `icoSlotLine` (`parts.js:390`), rewritten against the engine's own answer.
	## `slots` is the family's frozen vocabulary size and `filled` is how many of
	## those the loaded pack has art for -- both measured, where the design's
	## `[filled, total]` pair described the design's own art and had to be
	## labelled "(design figures)" because nothing here could check it.
	_icon_slot_line = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO)
	_icon_slot_line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	fam.add_child(_icon_slot_line)

	if _icon_placement_rows.is_empty():
		_dead_slider(fam, "icon scale", ICON_SCALE_RANGE, 0.01, _icon_gen_scale, "×")
		_dead_slider(fam, "min spacing", ICON_SPACING_RANGE, 1.0, _icon_gen_spacing, " cells")
	else:
		DccWidgets.slider(fam, "icon scale", ICON_SCALE_RANGE.x, ICON_SCALE_RANGE.y, 0.01,
			_icon_gen_scale, "×", func(v: float): _icon_gen_scale = v,
			"Per-instance size of every icon this pass places, clamped to the same bounds a hand-placed icon uses.")
		## **Cells, not pixels.** `parts.js:392`'s inverse map is `p*40` with no
		## unit stated; the engine measures spacing in grid cells, because that
		## is the frame an icon's own x/y and its box are in and it is the only
		## one that does not change under zoom.
		DccWidgets.slider(fam, "min spacing", ICON_SPACING_RANGE.x, ICON_SPACING_RANGE.y, 1.0,
			_icon_gen_spacing, " cells", func(v: float): _icon_gen_spacing = v,
			"Minimum centre-to-centre separation, in grid cells. Only applied while `enforce min spacing` is on; icons never overlap each other's glyph regardless.")

	## `icoRules` (`parts.js:393`), all three, in the design's order and with its
	## own wording. `snapCoast` starts off; the other two start on
	## (`parts.js:398`). All three are live now.
	var rules := DccWidgets.group(sec, "Placement rules", true)
	var rule_specs: Array = [
		["avoid label boxes", _icon_gen_avoid_labels,
			"On, an icon whose glyph lands on a label's box is suppressed and counted. Measured with the same culler the labelling pass uses, against every label on the map, generated or hand-placed.",
			func(on: bool): _icon_gen_avoid_labels = on],
		["enforce min spacing", _icon_gen_enforce_spacing,
			"On, the run is thinned to the minimum spacing above. Off, icons are still kept from overlapping each other's own glyph -- what this adds is the slider.",
			func(on: bool): _icon_gen_enforce_spacing = on],
		["snap sea marks to coast", _icon_gen_snap_coast,
			"On, every SEA MARKS glyph is pulled to the nearest coast cell -- water with land against it -- and one with no coast in reach is dropped rather than left in open sea. Applies to SEA MARKS only; the other three families are never moved.",
			func(on: bool): _icon_gen_snap_coast = on],
	]
	for r in rule_specs:
		var cb := DccWidgets.toggle(rules, String(r[0]), bool(r[1]), r[3] as Callable, String(r[2]))
		cb.tooltip_text = String(r[2])
		if _icon_placement_rows.is_empty():
			cb.disabled = true
			_mark_inert(cb.get_parent() as Control)

	var run := DccWidgets.action(sec, "Place this family", _run_icon_placement)
	run.disabled = _icon_placement_rows.is_empty()

	## Where every counter the pass returns goes. Four rejection reasons, and
	## they are disjoint -- a candidate is charged to exactly one -- so a run
	## that placed nothing still says which wall it hit.
	_icon_gen_summary = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO)
	_icon_gen_summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	sec.add_child(_icon_gen_summary)

	_sync_icon_placement()

## Re-read the engine's family table and repaint the slot line from it.
##
## Only the *numbers* move: which four families exist is a design table that
## does not change during a session, so the chips are not rebuilt (rebuilding
## them would drop the user's selection on every generate()).
func _refresh_icon_placement_rows() -> void:
	var rows: Array = bridge.icon_placement_families()
	if rows.is_empty():
		return
	_icon_placement_rows = rows
	_sync_icon_placement()

func _set_icon_placement_family(id: String) -> void:
	_icon_placement_family = id
	_sync_icon_placement()

## Run the pass and report every counter it returned.
func _run_icon_placement() -> void:
	var res: Dictionary = bridge.icon_generate({
		"family": _icon_placement_family,
		"scale": _icon_gen_scale,
		"min_spacing": _icon_gen_spacing,
		"avoid_labels": _icon_gen_avoid_labels,
		"enforce_spacing": _icon_gen_enforce_spacing,
		"snap_coast": _icon_gen_snap_coast,
	})
	if _icon_gen_summary != null and is_instance_valid(_icon_gen_summary):
		if not bool(res.get("ok", false)):
			## The engine's own sentence, not a rewrite of it -- same rule
			## `_regenerate_labels` follows for the same reason.
			_icon_gen_summary.text = String(res.get("reason", ""))
		else:
			var parts: Array[String] = ["%d placed" % int(res.get("placed", 0))]
			for pair in [["culled_spacing", "spaced out"], ["culled_label", "on a label"],
					["off_coast", "no coast"], ["unknown_slot", "wrong family"]]:
				var n := int(res.get(String(pair[0]), 0))
				if n > 0:
					parts.append("%d %s" % [n, String(pair[1])])
			if int(res.get("snapped", 0)) > 0:
				parts.append("%d snapped" % int(res.get("snapped", 0)))
			parts.append("%d ms" % int(res.get("elapsed_ms", 0)))
			_icon_gen_summary.text = " · ".join(parts)
	_rebuild_icon_panel()
	app.viewport.refresh_annotations()

func _sync_icon_placement() -> void:
	for id in _icon_family_chips:
		DccWidgets.set_segment_on(_icon_family_chips[id], id == _icon_placement_family)
	for entry in _icon_placement_rows:
		var f: Dictionary = entry
		if String(f.get("key", "")) == _icon_placement_family and _icon_slot_line != null:
			var filled := int(f.get("filled", 0))
			var slots := int(f.get("slots", 0))
			if filled == 0:
				_icon_slot_line.text = "%d slots · no pack art loaded, every slot draws its procedural glyph" % slots
			else:
				_icon_slot_line.text = "%d of %d slots carry pack art · unfilled slots fall back to the procedural glyph" \
					% [filled, slots]


## **Make an inert control LOOK inert.**
##
## `disabled = true` / `editable = false` stops a control moving; it does not
## reliably say so. Godot draws a disabled `CheckBox` with its `checked` icon at
## full strength and a `chip()` with `set_segment_on()`'s accent border still
## painted, so the first screenshot of these two panels showed an accent-filled
## "collision culling" box and a lit PLACES chip that a user would reasonably try
## to press. The tooltip carries the reason, but a tooltip is only found by
## someone who already suspects something is wrong.
##
## A `modulate` on the whole row is the cheapest signal that covers every control
## type at once -- label, box, chip, slider grip -- and it composes with the
## `text_ghost` readout `_dead_slider()` already sets rather than fighting it.
## `.55` is the same ratio `DccTheme`'s own `text_ghost`/`text` pair sits at
## against `panel`, so a dimmed row lands where a disclosed-gap note already
## does instead of at a new, fourth ink level.
const INERT_DIM := 0.55

static func _mark_inert(node: Control) -> void:
	if node != null and is_instance_valid(node):
		node.modulate = Color(1.0, 1.0, 1.0, INERT_DIM)

## One dial the design specifies and the engine cannot serve: drawn at its
## design default, with its design range, and inert.
##
## `editable = false` rather than omitting the slider, and rather than a plain
## readout: the range is part of what the design states (a 0-4 px halo is a
## different claim from a 0-40 px one), and only a real `HSlider` shows it. The
## `tooltip` is the same string on the row and the grip so a hover anywhere over
## it answers the question.
func _dead_slider(parent: Control, label_text: String, range_: Vector2, step: float,
		value: float, unit: String) -> Dictionary:
	var why := "Not bound -- see the note at the top of this section. Drawn at the design's own default and range."
	var d := DccWidgets.slider(parent, label_text, range_.x, range_.y, step, value, unit,
		func(_v: float): pass, why)
	var s: HSlider = d["slider"]
	s.editable = false
	s.tooltip_text = why
	## `text_ghost` is what every other disclosed-gap readout in this shell uses
	## (`_set_clear_state`'s disabled button, `Workspace._not_built`'s note), so
	## an inert value reads the same way here as it does there.
	(d["readout"] as Label).add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	_mark_inert(d["row"])
	return d

func _build_label_panel(parent: Control) -> void:
	## `#carLabelList` (`DCC_SHELL_SPEC.md` §4.5.5).
	##
	## **"Region labels", not "Placed labels"** (`GUI_GAP_REGISTER.md` CA-13,
	## owner report 2026-08-24: "it isn't possible to drop a name for a region
	## on the map as in the HTML version"). It always was possible -- arm Label
	## in TOOLS, click empty ground, type the name -- and a live drive confirmed
	## the whole path works, handles and all. What was missing was any word
	## anywhere that connected this section to the thing the owner was looking
	## for. The reference calls them region labels in every place it names them
	## (`FUNCTION_INDEX.md`: `_civPopulateLabelEditor` "Build the region-label
	## editor", `_civRenderLabelList`, `clearLabels` "Clear region labels"), so
	## this is the reference's own vocabulary, not a new coinage.
	var sec := DccWidgets.section(parent, "Region labels")
	_label_list_body = VBoxContainer.new()
	_label_list_body.add_theme_constant_override("separation", 2)
	sec.add_child(_label_list_body)
	_label_clear_btn = DccWidgets.action(sec, "Clear all labels", func():
		bridge.label_clear_all()
		app.viewport.refresh_annotations()
		app.viewport.tool_overlay.set_handles([])
		_rebuild_label_panel())
	sec.add_child(DccTheme.rule())
	_label_edit_body = VBoxContainer.new()
	_label_edit_body.add_theme_constant_override("separation", 2)
	sec.add_child(_label_edit_body)


func _rebuild_label_panel() -> void:
	if _label_list_body == null:
		return
	for child in _label_list_body.get_children():
		_label_list_body.remove_child(child)
		child.queue_free()
	var list: Array = bridge.label_list()
	_set_clear_state(_label_clear_btn, "labels", list.size(),
		"No region labels placed yet -- arm Label (L) in TOOLS above and click "
		+ "empty ground. There is nothing to clear.")
	if list.is_empty():
		## An empty state that says how to leave it, the same way Logistics'
		## own "No committed routes yet -- draw one with the Route tool above"
		## does (`infrastructure_workspace.gd`). "none placed" was true and
		## useless: it named the absence without naming the tool that ends it.
		DccWidgets.note(_label_list_body,
			"None yet -- arm Label (L) in TOOLS above, then click empty ground " +
			"and type the region's name. Drag its handles to size, rotate and " +
			"arc it.")
	for entry in list:
		var d: Dictionary = entry
		var idx: int = int(d.get("index", -1))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var name_text: String = String(d.get("text", ""))
		var l := DccTheme.mono_label(name_text if not name_text.is_empty() else "(untitled)", "text_dim", DccTheme.FS_SMALL)
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.clip_text = true
		## `selected` is membership of the selection set and `primary` is the
		## one carrying the edit session -- at one selected label they are the
		## same bool and this reads exactly as it always did.
		if bool(d.get("selected", false)):
			l.add_theme_color_override("font_color",
				DccTheme.c("accent" if bool(d.get("primary", true)) else "text"))
		row.add_child(l)
		var sel := Button.new()
		sel.text = "edit"
		sel.tooltip_text = "Select for editing"
		sel.focus_mode = Control.FOCUS_NONE
		sel.custom_minimum_size = Vector2(34, 20)
		sel.pressed.connect(func():
			bridge.label_select(idx)
			app.set_tool_options(_build_label_tool_options_row)
			_rebuild_label_panel())
		row.add_child(sel)
		var del := Button.new()
		del.text = "×"
		del.tooltip_text = "Delete"
		del.focus_mode = Control.FOCUS_NONE
		del.custom_minimum_size = Vector2(22, 20)
		del.pressed.connect(func():
			bridge.label_delete(idx)
			app.viewport.refresh_annotations()
			app.set_tool_options(_build_label_tool_options_row)
			_rebuild_label_panel())
		row.add_child(del)
		_label_list_body.add_child(row)

	_rebuild_label_edit_form()
	_refresh_right_dock_anno()


## The full seven-field form (`DCC_SHELL_SPEC.md` §4.5.5's right-dock
## description, repurposed into this dock per this file's own top comment)
## for whichever label `label_get_selected()` names, or a placeholder note
## when none is selected. Rebuilt wholesale only from coarse events (select/
## deselect/create/delete/confirm/cancel) -- a live slider drag or keystroke
## calls `_apply_label_field` directly instead of rebuilding, so the control
## being dragged/typed into is never torn down mid-gesture.
func _rebuild_label_edit_form() -> void:
	for child in _label_edit_body.get_children():
		_label_edit_body.remove_child(child)
		child.queue_free()
	var idx := bridge.label_get_selected()
	if idx < 0:
		DccWidgets.note(_label_edit_body,
			"Select a label above, or click one on the map, to edit its text and style.")
		_update_label_handles_overlay()
		return
	var lb: Dictionary = bridge.label_get(idx)
	if lb.is_empty():
		_update_label_handles_overlay()
		return

	var text_row := HBoxContainer.new()
	text_row.add_theme_constant_override("separation", 8)
	text_row.custom_minimum_size.y = 24
	text_row.add_child(DccTheme.mono_label("Text", "text_dim", DccTheme.FS_SMALL))
	var text_edit := LineEdit.new()
	text_edit.text = String(lb.get("text", ""))
	text_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text_edit.text_changed.connect(func(v: String): _apply_label_field(idx, "text", v))
	text_edit.focus_exited.connect(_rebuild_label_panel)
	text_row.add_child(text_edit)
	_label_edit_body.add_child(text_row)

	## Step 1 of the Labels ruling, reachable: which typographic class this
	## hand-placed label belongs to. It sets the halo and tracking it draws with
	## (`map_overlay.gd::_draw_labels`) and its priority once the collision
	## culler lands; size, font and colour stay this form's own three fields
	## below, because those are the user's and the class has no claim on them.
	var cls := String(bridge.label_class_of(idx))
	if cls.is_empty():
		cls = "settlement"   ## `MapLabel::class`'s own default.
	## Guarded on the spec list rather than assumed: `_build_label_classes()`
	## fills it, and an `OptionButton` seeded with an out-of-range selection over
	## an empty option list is an error, not an empty row.
	if not _label_class_specs.is_empty():
		DccWidgets.choice(_label_edit_body, "Class",
			_label_class_specs.map(func(c: Dictionary) -> String: return String(c.get("label", ""))),
			_label_class_index(cls),
			func(i: int):
				bridge.label_set_class(idx, String((_label_class_specs[i] as Dictionary).get("key", "")))
				app.viewport.refresh_annotations(),
			"Sets the halo and tracking this label draws with, from the class table above. Not part of the Confirm/Cancel snapshot -- like a reposition, it commits immediately.")

	DccWidgets.slider(_label_edit_body, "Size", LABEL_SIZE_MIN, LABEL_SIZE_MAX, 1.0, float(lb.get("size", 16.0)), "px",
		func(v: float): _apply_label_field(idx, "size", v))
	DccWidgets.choice(_label_edit_body, "Size mode", ["Fixed", "Zoom with map"],
		0 if String(lb.get("size_mode", "fixed")) == "fixed" else 1,
		func(i: int): _apply_label_field(idx, "size_mode", "fixed" if i == 0 else "zoom"))
	DccWidgets.slider(_label_edit_body, "Arc", -1.0, 1.0, 0.01, float(lb.get("arc", 0.0)), "",
		func(v: float): _apply_label_field(idx, "arc", v))
	DccWidgets.slider(_label_edit_body, "Angle", -180.0, 180.0, 1.0, float(lb.get("angle", 0.0)), "°",
		func(v: float): _apply_label_field(idx, "angle", v))

	var font_row := HBoxContainer.new()
	font_row.add_theme_constant_override("separation", 8)
	font_row.custom_minimum_size.y = 24
	font_row.add_child(DccTheme.mono_label("Font", "text_dim", DccTheme.FS_SMALL))
	var font_edit := LineEdit.new()
	font_edit.text = String(lb.get("font", ""))
	font_edit.placeholder_text = "Georgia, serif"
	font_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	font_edit.text_submitted.connect(func(v: String): _apply_label_field(idx, "font", v))
	font_edit.focus_exited.connect(func(): _apply_label_field(idx, "font", font_edit.text))
	font_row.add_child(font_edit)
	_label_edit_body.add_child(font_row)

	var color_row := HBoxContainer.new()
	color_row.add_theme_constant_override("separation", 8)
	color_row.custom_minimum_size.y = 24
	color_row.add_child(DccTheme.mono_label("Color", "text_dim", DccTheme.FS_SMALL))
	var picker := ColorPickerButton.new()
	picker.color = Color(String(lb.get("color", "#f4e9c8")))
	picker.custom_minimum_size = Vector2(60, 20)
	picker.color_changed.connect(func(c: Color): _apply_label_field(idx, "color", "#%s" % c.to_html(false)))
	color_row.add_child(picker)
	_label_edit_body.add_child(color_row)

	DccWidgets.note(_label_edit_body,
		"The literal CSS font string the engine stores -- Godot has no web-font "
		+ "fallback chain, so only size/angle/arc/color render from this form "
		+ "(map_overlay.gd's own doc comment). Letter-spacing is not per label: "
		+ "it belongs to the class, in the section above, and this label takes "
		+ "its class's value. Anchor still has no backing field on MapLabel "
		+ "(label_bridge.rs's own \"Not modelled\" note) and is not exposed here.")

	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 6)
	DccWidgets.action(actions, "Confirm", _confirm_label_edit, true)
	DccWidgets.action(actions, "Cancel", _cancel_label_edit)
	DccWidgets.action(actions, "Delete", func():
		bridge.label_delete(idx)
		app.viewport.refresh_annotations()
		app.set_tool_options(_build_label_tool_options_row)
		_rebuild_label_panel())
	_label_edit_body.add_child(actions)

	_update_label_handles_overlay()


## Draws the selected label's resize/rotate/arc handles (`tool_overlay.gd`'s
## `set_handles`, already built) -- filters out any `{}` slot (a fixed-size
## label has no resize handle) per that method's own doc comment.
##
## Drawn only for a selection of exactly one, for the reason
## `_update_icon_handles_overlay` gives one section up -- more so here, since
## these three handles write `size`/`angle`/`arc` into one label and there is
## no defined meaning for rotating a set about nothing in particular.
func _update_label_handles_overlay() -> void:
	var idx := bridge.label_get_selected()
	if idx < 0 or bridge.label_get_selection().size() != 1:
		app.viewport.tool_overlay.set_handles([])
		return
	var h := bridge.label_handles(idx, app.viewport.zoom())
	if h.is_empty():
		app.viewport.tool_overlay.set_handles([])
		return
	var raw: Array = []
	for key in ["resize", "rotate", "arc"]:
		var hd: Dictionary = h.get(key, {})
		if not hd.is_empty():
			raw.append(hd)
	app.viewport.tool_overlay.set_handles(raw)
