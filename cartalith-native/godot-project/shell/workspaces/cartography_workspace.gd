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
## §7 specifies three panes and a ten-row layer stack. The renderer can honour
## five of those rows today; the rest (hand-drawn hillshade, colour relief as
## separate layers, opacity, blend mode, the ramp editor) need `render.rs`'s
## `TerrainAppearance` bound to Godot first.
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
## "Layer properties" is gone as a category: it was one honest note about a
## capability that does not exist (CA-04), and v3 folds per-layer opacity/
## blend/zoom into LAYERS' own rows. The note moved there rather than being
## dropped. Nothing was rewritten -- every builder below is the one that was
## already here, called with a different parent.

## The layers the shell can actually toggle, in §7's own draw order:
## topmost first, matching how the layer list reads.
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

## What an in-progress canvas drag on the Label tool is doing -- set on
## `map_clicked` (a handle grab or a hit on an existing label's box), read on
## every subsequent `map_dragged` sample, cleared on `map_released`. `RESIZE`/
## `ROTATE`/`ARC` mirror the three handles `label_handles()` returns.
enum DragMode { NONE, MOVE, RESIZE, ROTATE, ARC }

## The Icon tool's own drag state -- `IconDragMode`'s one-handle mirror of
## `DragMode` above (`GUI_GAP_REGISTER.md` CA-05: `icon_handles()` returns
## exactly one circle, `"resize"`, since a manually-placed icon has no
## rotate/arc field at all -- `icon_bridge.rs`'s own doc comment). `MOVE`
## has no counterpart here either: unlike the Label tool, a box hit outside
## the handle isn't wired to a move-drag yet (`icon_hit_test` is exposed but
## unused by this file still -- a separate gap from CA-05, not folded in
## here).
enum IconDragMode { NONE, RESIZE }

# -- Icon tool state (UI-side only -- the engine holds the authoritative armed
# selection via `icon_arm`; this is just what the tool options row shows and
# re-arms from on every change). --------------------------------------------
var _icon_family_idx := 0
var _icon_variant_idx := 0
var _icon_scale := 1.0
var _icon_rotation := 0.0
var _icon_jitter := 0.0
var _icon_list_body: VBoxContainer

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
		DccWidgets.toggle(body, layer.label, layer.on,
			func(on: bool): app.viewport.set_layer_visible(layer.id, on))
	DccWidgets.note(body,
		"Terrain, hillshade and colour relief are one baked raster today, so "
		+ "they toggle together with the map itself rather than as separate rows.")
	_build_settlement_class_filter(cat)
	_build_layer_gaps(cat)

	## 5 -- ROADS & ROUTES
	_build_way_style(DccWidgets.category(self, "Roads & routes", categories))

	## 6 -- LABELS
	_build_label_panel(DccWidgets.category(self, "Labels", categories))

	## 7 -- ASSETS & LANDMARKS
	_build_icon_panel(DccWidgets.category(self, "Assets & landmarks", categories))

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
	DccWidgets.note(sec,
		"Per-layer opacity, draw order and blend mode (GUI_GAP_REGISTER.md CA-04): "
		+ "terrain, hillshade and colour relief are composited into one raster by "
		+ "render.rs before it crosses the boundary, so there are no separable "
		+ "outputs to order or blend. Opacity alone is cheap once they separate. "
		+ "v3's sixteen-row draggable stack, its per-layer zoom range and its "
		+ "picking/clip switches all rest on that same separation.")
	DccWidgets.note(sec,
		"Show rivers in biome view (#showRivers) and Rivers as ways: both are "
		+ "reference RENDER filters over a river network that never crosses the "
		+ "GDExtension boundary -- cartalith-hydrology computes it internally and "
		+ "only the finished raster comes out (there is no get_rivers()). Same "
		+ "entity gap the Rivers subject and the right dock's River context both "
		+ "already report.")
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
		DccWidgets.toggle(sec, layer.label, layer.on,
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

	var gaps := DccWidgets.section(parent, "Not built")
	DccWidgets.note(gaps,
		"Border line width and style, claim hatching, and the influence gradient "
		+ "with its legend (GUI_GAP_REGISTER.md CA-17). All three rest on data "
		+ "that does not exist rather than on a missing control: CivData::"
		+ "territory is one plurality owner per cell, with no contested-claim "
		+ "value and no influence field for a gradient to ramp (CV-23). Fill "
		+ "opacity and identity colour, above, are the two that had a real "
		+ "quantity behind them.")


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
		+ "zoom range needs the separable layer stack CA-04 is blocked on.")
	DccWidgets.note(gaps,
		"The declutter budget is not built: label and icon collision is not "
		+ "resolved anywhere -- overlapping annotation simply overlaps.")
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
	match id:
		"icon":
			_arm_icon_from_ui()
			app.set_tool_options(_build_icon_tool_options_row)
			_rebuild_icon_panel()
		"label":
			app.set_tool_options(_build_label_tool_options_row)
			_rebuild_label_panel()
		_:
			_label_drag_mode = DragMode.NONE
			_label_drag_index = -1
			_icon_drag_mode = IconDragMode.NONE
			_icon_drag_index = -1
			app.viewport.tool_overlay.set_handles([])
			if app.active_domain() == "cartography":
				_show_style_tool_options()


func _on_world_changed() -> void:
	app.viewport.tool_overlay.set_handles([])
	_label_drag_mode = DragMode.NONE
	_label_drag_index = -1
	_icon_drag_mode = IconDragMode.NONE
	_icon_drag_index = -1
	if app.armed_tool == "icon":
		_arm_icon_from_ui()
	_rebuild_icon_panel()
	_rebuild_label_panel()


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
	var slot_labels: Array = slots.map(func(s): return String(s).capitalize())
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

	var armed := bridge.icon_armed()
	if not armed.is_empty():
		row.add_child(DccTheme.mono_label(
			"→ %s/%s ×%.2f" % [armed.get("family", ""), armed.get("slot", ""), float(armed.get("scale", 1.0))],
			"text_dim", DccTheme.FS_MICRO))
	row.add_child(DccTheme.spacer())


## Pointerdown-on-the-handle-starts-a-resize, a-miss-falls-through-to-place
## precedence (reference lines 9664-9671: `_carIconHitTest` checked before
## the click handler's own place/select branch) -- `IconEditor::handles`
## doesn't require `sel` to already be selected, but the reference's own
## `_iconHandle` is only ever set for `_iconSelected`, so checking it here
## against whatever `icon_get_selected()` currently names reproduces that.
func _on_icon_click(gx: float, gy: float) -> void:
	var sel := bridge.icon_get_selected()
	if sel >= 0:
		var h: Dictionary = bridge.icon_handles(sel, app.viewport.zoom()).get("resize", {})
		if not h.is_empty() and Vector2(gx, gy).distance_to(Vector2(h["x"], h["y"])) <= float(h["r"]):
			_begin_icon_handle_drag(sel, gx, gy)
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
	if _icon_drag_mode != IconDragMode.RESIZE or _icon_drag_index < 0:
		return
	bridge.icon_resize(_icon_drag_index, _icon_drag_cx, _icon_drag_cy, gx, gy, _icon_drag_start_dist)
	app.viewport.refresh_annotations()
	_update_icon_handles_overlay()


func _on_icon_release(_gx: float, _gy: float, _valid: bool) -> void:
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
	DccWidgets.action(sec, "Clear all icons", func():
		bridge.icon_clear_all()
		app.viewport.refresh_annotations()
		_rebuild_icon_panel())
	DccWidgets.note(sec,
		"Arm the Icon tool above, then click the map to stamp it. Family, "
		+ "variant, scale, rotation and jitter live in the tool options bar "
		+ "while Icon is armed. Placing an icon selects it and shows its own "
		+ "on-canvas resize handle -- drag it to rescale in place; delete and "
		+ "re-place to change family/slot.")


func _rebuild_icon_panel() -> void:
	if _icon_list_body == null:
		return
	for child in _icon_list_body.get_children():
		_icon_list_body.remove_child(child)
		child.queue_free()
	var list: Array = bridge.icon_list()
	if list.is_empty():
		_icon_list_body.add_child(DccTheme.label("none placed", "text_ghost", DccTheme.FS_MICRO))
	for entry in list:
		var d: Dictionary = entry
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var text := "%s / %s  ·  ×%.2f" % [String(d.get("family", "")), String(d.get("slot", "")), float(d.get("scale", 1.0))]
		var l := DccTheme.mono_label(text, "text_dim", DccTheme.FS_SMALL)
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.clip_text = true
		if int(d.get("index", -1)) == bridge.icon_get_selected():
			l.add_theme_color_override("font_color", DccTheme.c("accent"))
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


## Draws the selected icon's resize handle (`icon_handles`,
## `GUI_GAP_REGISTER.md` CA-05) through the same `tool_overlay.gd` primitive
## the Label tool's own handles already use -- `_update_label_handles_
## overlay`'s one-handle mirror.
func _update_icon_handles_overlay() -> void:
	var idx := bridge.icon_get_selected()
	if idx < 0:
		app.viewport.tool_overlay.set_handles([])
		return
	var h: Dictionary = bridge.icon_handles(idx, app.viewport.zoom()).get("resize", {})
	app.viewport.tool_overlay.set_handles([h] if not h.is_empty() else [])


# ===========================================================================
# Label tool (§4.5.5)
# ===========================================================================

func _on_label_click(gx: float, gy: float) -> void:
	var sel := bridge.label_get_selected()
	if sel >= 0:
		var mode := _handle_hit(sel, gx, gy)
		if mode != DragMode.NONE:
			_begin_label_handle_drag(sel, mode, gx, gy)
			return

	var hit := bridge.label_hit_test(gx, gy)
	if hit >= 0:
		_label_drag_mode = DragMode.MOVE
		_label_drag_index = hit
		app.set_tool_options(_build_label_tool_options_row)
		_rebuild_label_panel()
	else:
		_label_drag_mode = DragMode.NONE
		_label_drag_index = -1
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
	DccWidgets.action(sec, "Clear all labels", func():
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
		if bool(d.get("selected", false)):
			l.add_theme_color_override("font_color", DccTheme.c("accent"))
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
		+ "fallback chain, so only size/angle/arc/color actually render "
		+ "(map_overlay.gd's own doc comment). Letter-spacing and anchor from the "
		+ "spec's tool-options row have no backing field on MapLabel "
		+ "(label_bridge.rs's own \"Not modelled\" note) and are not exposed here.")

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
func _update_label_handles_overlay() -> void:
	var idx := bridge.label_get_selected()
	if idx < 0:
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
