extends Control
## DCC shell (DCC_SHELL_SCOPE.md milestone 1): structural replacement of the
## prior panel-browser shell (GUI_SHELL_SCOPE.md, commits 5d44c6b..2dee8fc).
## Six regions per UI_SHELL_DESIGN.md's own governing table: top menu bar
## (program actions only), workspace tabs, tool options bar, left tool rail,
## viewport, right dock (Layers/Properties/Sample), status bar. Desktop
## 1920x1080 dark theme only this pass -- light theme, responsive
## breakpoints, and any actual tool functionality (pass-buffer/commit/
## discard/staleness) are explicitly deferred (DCC_SHELL_SCOPE.md milestone
## 3+, UNIFIED_TOOL_PLAN.md).
##
## Every real, working control from the prior shell is re-parented here --
## generation params moved into a "New World" dialog (File menu), reached
## the same way a DCC's own New Document dialog is: world setup is a
## program-level action, not a persistent dock panel, so it belongs behind
## a menu, not in the right dock (UI_SHELL_DESIGN.md's own governing rule:
## "the top bar is about the program, the map is about the world").
##
## The left tool rail's 16 tools + tool-preferences icon are built and
## visible, honestly inert: selecting one only changes which tool name the
## Tool Options Bar shows (a real, harmless presentation affordance) -- no
## pass-buffer/commit/discard/map-editing exists yet. Same "shell now, wire
## later" discipline GUI_SHELL_SCOPE.md milestone 1 already established.
##
## Category-1 parity-audit items folded into this rebuild while these
## controls were already being touched (GUI_FEATURE_PARITY_SCOPE.md):
## #1 (asset-pack import, now real: File > Import asset pack), #9 (layer
## granularity: Settlements/Roads/Sea routes are now three independent
## toggles instead of one that hid all of map_overlay together), #10
## (click-to-pin selection: the Properties dock now holds a settlement's
## causal "why here?" chain after a click, independent of the transient
## Sample dock's live hover data). Left for later: #2-5 (settlements
## table/economy panel/province list/culture-fit -- each needs its own real
## table UI, not a drive-by wiring), #6 (planet params setter), #7 (GPU
## toggle/readout -- GPU_LAYER_INTEGRATION_SCOPE.md's own current milestone
## is still the noise redesign, wiring a toggle ahead of that would surface
## an incomplete path), #8 (World Structure raw sliders).
##
## DCC_SHELL_SCOPE.md milestone 3 turns that New World dialog into a real
## world-setup gate: map width in km, working resolution, extent mode and
## frame aspect, with the derived grid/extent/cell-size shown live, and
## generation dispatched through WorldGen.generate_sized()/
## generate_world_structure_sized() so maps are no longer forced square. See
## the "World setup" section below for the three engine rules that shape it.
##
## Generation still runs on a background Thread (unchanged). WorldGen.
## generate_sized()/generate_world_structure_sized() are pure Rust
## computation over plain WorldState, safe off-thread; build_color_texture()
## and every scene-tree write happen back on the main thread via
## call_deferred.

@onready var seed_input: SpinBox = %SeedInput
@onready var resolution_input: OptionButton = %ResolutionInput
@onready var width_input: SpinBox = %WidthInput
@onready var sea_level_input: SpinBox = %SeaLevelInput
@onready var world_shape_input: OptionButton = %WorldShapeInput
@onready var dynamic_lithology_check: CheckBox = %DynamicLithologyCheck
@onready var volc_provinces_check: CheckBox = %VolcProvincesCheck
@onready var wind_deflection_check: CheckBox = %WindDeflectionCheck
@onready var ocean_currents_check: CheckBox = %OceanCurrentsCheck
@onready var villages_check: CheckBox = %VillagesCheck
@onready var generate_button: Button = %GenerateButton
@onready var status_label: Label = %StatusLabel ## New World dialog's own detailed status line.
@onready var map_view: TextureRect = %MapView
@onready var territory_view: TextureRect = %TerritoryView
@onready var province_boundary_view: TextureRect = %ProvinceBoundaryView
@onready var map_overlay: Control = %MapOverlay
@onready var load_save_dialog: FileDialog = %LoadSaveDialog
@onready var asset_pack_dialog: FileDialog = %AssetPackDialog
@onready var credits_dialog: AcceptDialog = %CreditsDialog
@onready var new_world_dialog: AcceptDialog = %NewWorldDialog
@onready var new_world_list: VBoxContainer = %NewWorldList ## The New-world dialog's scrolling section list; _build_world_setup() prepends to it.

## Shell chrome, new this milestone.
@onready var readout_label: Label = %ReadoutLabel
@onready var file_menu: MenuButton = %FileMenu
@onready var edit_menu: MenuButton = %EditMenu
@onready var generate_menu: MenuButton = %GenerateMenu
@onready var simulate_menu: MenuButton = %SimulateMenu
@onready var render_menu: MenuButton = %RenderMenu
@onready var assets_menu: MenuButton = %AssetsMenu
@onready var view_menu: MenuButton = %ViewMenu
@onready var help_menu: MenuButton = %HelpMenu

@onready var tabs_row: HBoxContainer = %TabsRow
@onready var workspace_subtitle_label: Label = %WorkspaceSubtitleLabel
@onready var active_tool_label: Label = %ActiveToolLabel
@onready var tool_rail_vbox: VBoxContainer = %ToolRailVBox

@onready var show_settlements_check: CheckBox = %ShowSettlementsCheck
@onready var show_roads_check: CheckBox = %ShowRoadsCheck
@onready var show_sea_routes_check: CheckBox = %ShowSeaRoutesCheck
@onready var territory_layer_check: CheckBox = %TerritoryLayerCheck
@onready var province_layer_check: CheckBox = %ProvinceLayerCheck

@onready var properties_header: Label = %PropertiesHeader
@onready var properties_body: RichTextLabel = %PropertiesBody
@onready var sample_body: RichTextLabel = %SampleBody

@onready var scale_bar_label: Label = %ScaleBarLabel
@onready var coordinates_label: Label = %CoordinatesLabel

@onready var shell_status_label: Label = %ShellStatusLabel
@onready var status_hint_label: Label = %StatusHintLabel ## Status bar's own "active tool's modifier hints" slot (UI_SHELL_DESIGN.md).

var world_gen: WorldGen = WorldGen.new()
var _gen_thread: Thread
var _generating := false
var _last_width_km := 0.0

## Index into WorldShapeInput -> the archetype name WorldGen.
## generate_world_structure expects (reference HTML ARCHETYPES). Index 0
## ("Classic") isn't an archetype at all -- it's World-Structure disabled,
## the plain generate() path.
const WORLD_SHAPES: Array[String] = ["", "earth", "supercontinent", "archipelago", "volcanic", "rift"]
const WORLD_SHAPE_LABELS: Array[String] = ["Classic", "Earth-like", "Supercontinent", "Archipelago", "Volcanic", "Rift"]

const RESOLUTION_PRESETS: Array[int] = [512, 1024, 2048, 4096, 8192]
const RESOLUTION_LABELS: Array[String] = ["512", "1K", "2K", "4K", "8K"]
const RESOLUTION_DEFAULT_INDEX := 2 ## 2K, matching the reference's own default.
const RESOLUTION_CUSTOM_INDEX := 5 ## Appended after the five presets -- free grid-width entry.

## ── World setup: map size, resolution, dimensions ───────────────────────
## DCC_SHELL_SCOPE.md milestone 3. Owner's own request: "a proper base setup
## menu where we can pick map size, resolution, dimensions - basically
## expanded from the current html version."
##
## Built at runtime, and every physical constant it needs comes from the
## engine rather than from a second copy here: WorldGen.
## reference_grid_height() owns both of the reference app's own gridH
## factors (0.5 world / 0.64 region), get_map_width_km()/get_map_height_km()
## report what actually generated, and set_params({"world": ...}) is the one
## place the extent mode is stored. GDScript holds only labels and layout
## (ARCHITECTURE.md: "Godot computes nothing beyond layout").
##
## Three engine rules this dialog is built around (GENERATION_PARAMETERS.md
## "Map dimensions and aspect ratio"):
##
## 1. CELLS ARE SQUARE IN KM. Every km<->cell conversion in the workspace
##    derives from the single quotient map_width_km / gw and applies it to
##    both axes, so the map's height in km is width_km * gh / gw -- derived,
##    never independently settable. There is deliberately no map-height-km
##    control below; the height in km is a readout.
## 2. WORLD MODE IS PHYSICALLY 2:1. X wraps 360 deg of longitude over gw and
##    Y spans 180 deg of latitude over gh, so any other ratio silently
##    stretches the graticule. In Whole-world extent the aspect control is
##    pinned to 2:1 and the grid height comes from reference_grid_height(gw,
##    true) -- with the reason stated on screen, not silently disabled.
## 3. GRID HEIGHT IS A CALL ARGUMENT, NOT A PARAMETER. It reallocates every
##    field in the pipeline, so it cannot honour the parameter table's "set
##    once, generate many" contract. It sits beside seed, resolution and
##    map width as an argument to generate_sized().

## Map-width presets, in km. Real cartographic scales rather than round
## numbers: the reference ships one free "Map width (km)" number input with
## no guidance at all, and 800 km (its default) is genuinely hard to place
## without a scale to compare it against. The free SpinBox beside them is
## still the authority -- picking a preset only writes it a value, and
## typing any other value flips the button back to "Custom".
const SIZE_PRESETS: Array = [
	{"label": "Local · 200 km", "km": 200.0},
	{"label": "Province · 800 km", "km": 800.0},
	{"label": "Region · 2 000 km", "km": 2000.0},
	{"label": "Subcontinent · 5 000 km", "km": 5000.0},
	{"label": "Continent · 12 000 km", "km": 12000.0},
	{"label": "Planet · 40 075 km (Earth's equator)", "km": 40075.0},
]
const SIZE_CUSTOM_INDEX := 6

## Aspect presets, width:height. The reference offers no aspect control at
## all -- its gridH() hardcodes 2:1 in world mode and 1.5625:1 otherwise --
## so both of those appear here by name, with the frame ratios a
## cartographic tool is actually asked for around them (DECISIONS.md 7d:
## improving past the reference is permitted where behaviour is preserved,
## and both reference ratios remain reachable by name).
const ASPECT_PRESETS: Array = [
	{"label": "2:1 · equirectangular", "ratio": 2.0},
	{"label": "16:9 · widescreen", "ratio": 16.0 / 9.0},
	{"label": "1.5625:1 · reference region frame", "ratio": 1.5625},
	{"label": "4:3 · classic landscape", "ratio": 4.0 / 3.0},
	{"label": "1:1 · square", "ratio": 1.0},
	{"label": "3:4 · portrait", "ratio": 0.75},
	{"label": "9:16 · tall portrait", "ratio": 9.0 / 16.0},
]
const ASPECT_WORLD_INDEX := 0 ## 2:1 -- the shape world mode is pinned to.
const ASPECT_REGION_INDEX := 2 ## The reference's own region frame; gh comes from the engine, not from the ratio above.
const ASPECT_CUSTOM_INDEX := 7
const ASPECT_DEFAULT_INDEX := ASPECT_REGION_INDEX

const GRID_MIN := 4 ## generate_sized() clamps each dimension to >= 4; match it rather than let the engine clamp behind the dialog's back.
const GRID_MAX := 8192

## Aspect ratios past this are non-crashing but degenerate: the coarse
## weather grid loses almost all resolution on the short axis, and the plate
## frame (a uniform margin in cells keyed to gw) eats a large fraction of
## the sheet. Found by the Rust non-square pass, surfaced here rather than
## left to be discovered after a five-minute generate.
const DEGENERATE_ASPECT := 16.0

const EXTENT_NOTE_REGION := "Region — a framed area of a world. The map's north and south edge latitudes are set in Generate ▸ Climate; X does not wrap. Any aspect ratio is physically fine here."
const EXTENT_NOTE_WORLD := "Whole world — a seamless equirectangular sheet: X wraps a full 360° of longitude and Y spans 180° of latitude, pole to pole. That fixes the shape at 2:1; any other ratio would stretch the graticule against the terrain, so the aspect is pinned and the grid height comes from the engine's own reference_grid_height(gw, true)."

var extent_input: OptionButton
var size_preset_input: OptionButton
var aspect_input: OptionButton
var grid_w_input: SpinBox
var grid_h_input: SpinBox
var extent_note_label: Label
var dimension_warning_label: Label
var _derived_labels: Dictionary = {} ## row key -> the value Label
var _dim_syncing := false
## Whether this build of the GDExtension carries the non-square API
## (generate_sized / reference_grid_height / get_map_*_km). False falls back
## to the square generate() path with the aspect controls disabled, rather
## than erroring on a stale binary -- same honesty rule _params_available
## already follows for get_param_info().
var _sized_api := false

## ── Workspace tabs ──────────────────────────────────────────────────────
## UI_SHELL_DESIGN.md's own workspace row: "what the old navigator's groups
## became". A tab swaps tool-rail emphasis and dock context; it never
## swaps the viewport (UI_SHELL_DESIGN.md's own rule).
const TAB_NAMES: Array[String] = ["WORLD", "CIVILIZATION", "INFRASTRUCTURE", "CARTOGRAPHY", "RENDER"]
const TAB_SUBTITLES := {
	"WORLD": "Terrain · Water · Climate · Ecology · Resources",
	"CIVILIZATION": "Settlements · Factions · Economy · Statistics",
	"INFRASTRUCTURE": "Roads · Ports · Trade · Logistics",
	"CARTOGRAPHY": "Map style · Labels · Icons · Paint",
	"RENDER": "Terrain appearance · Lighting · NPR · Bake",
}
## Which tool-rail group index (0-based, matching TOOL_GROUPS below) a tab
## puts visual emphasis on. Presentation-only -- no tool becomes functional
## by switching tabs, this only brightens/dims rail icon groups.
const TAB_TO_GROUP_INDEX := {
	"WORLD": 1, "CIVILIZATION": 3, "INFRASTRUCTURE": 3, "CARTOGRAPHY": 4, "RENDER": 4,
}
var _tab_buttons: Dictionary = {} ## tab name -> Button
var _active_tab := "WORLD"

## ── Left tool rail ──────────────────────────────────────────────────────
## UI_SHELL_DESIGN.md's own 5 groups, 16 tools total (+ tool preferences,
## pinned separately at the bottom). Selecting one only changes the Tool
## Options Bar's displayed name -- no pass-buffer/commit/discard exists
## (DCC_SHELL_SCOPE.md Track 2, not this milestone).
const TOOL_GROUPS: Array = [
	[{"name": "Select / inspect", "glyph": "➤", "key": "V"},
	 {"name": "Pan", "glyph": "✥", "key": "H"},
	 {"name": "Point sample", "glyph": "◎", "key": "I"}],
	[{"name": "Raise / lower", "glyph": "▲", "key": "B"},
	 {"name": "Smooth", "glyph": "≈", "key": "S"},
	 {"name": "Flatten / terrace", "glyph": "▭", "key": "F"},
	 {"name": "Stamp (landform library)", "glyph": "◆", "key": ""}],
	[{"name": "River / water", "glyph": "∿", "key": "R"},
	 {"name": "Biome paint", "glyph": "❋", "key": "P"}],
	[{"name": "Place settlement", "glyph": "⌂", "key": ""},
	 {"name": "Draw route / way", "glyph": "↝", "key": ""},
	 {"name": "Territory / faction", "glyph": "▩", "key": ""}],
	[{"name": "Label", "glyph": "T", "key": "T"},
	 {"name": "Icon stamp", "glyph": "✦", "key": ""},
	 {"name": "Measure", "glyph": "⟋", "key": "M"},
	 {"name": "Region select / export", "glyph": "⬚", "key": ""}],
]
var _tool_group_buttons: Array = [] ## Array[Array[Button]], mirrors TOOL_GROUPS.

## ── Right dock: Properties (click-to-pin) / Sample (live hover) ────────
var _selected_settlement: Variant = null
var _selected_index := -1
var _cursor_valid := false
var _cursor_gx := 0.0
var _cursor_gy := 0.0
var _hover_settlement: Variant = null

## Noun phrases for each suitability term key returned by
## WorldGen.explain_settlement(). Wording lives here, not in Rust: the
## engine supplies facts, the UI phrases them (ARCHITECTURE.md).
const SUIT_TERM_LABELS := {
	"carrying_capacity": "fertile land",
	"water_access": "fresh water",
	"gentle_slope": "gentle terrain",
	"terrain_form": "terrain form",
	"coastal_access": "coastal access",
	"river": "river access",
	"lake": "lakeside",
	"minerals": "mineral deposits",
	"route_corridor": "natural route corridor",
	"farmland": "farmland",
	"buildable_ground": "buildable ground",
	"flood_risk": "flood risk",
	"islet_penalty": "isolation",
	"water_bonus": "water",
}


func _ready() -> void:
	for label in WORLD_SHAPE_LABELS:
		world_shape_input.add_item(label)
	world_shape_input.selected = 0

	for label in RESOLUTION_LABELS:
		resolution_input.add_item(label)
	resolution_input.add_item("Custom")
	resolution_input.selected = RESOLUTION_DEFAULT_INDEX

	generate_button.pressed.connect(_on_generate_pressed)
	load_save_dialog.file_selected.connect(_on_save_file_selected)
	asset_pack_dialog.file_selected.connect(_on_asset_pack_file_selected)

	show_settlements_check.toggled.connect(func(pressed: bool): map_overlay.set_show_settlements(pressed))
	show_roads_check.toggled.connect(func(pressed: bool): map_overlay.set_show_roads(pressed))
	show_sea_routes_check.toggled.connect(func(pressed: bool): map_overlay.set_show_sea_routes(pressed))
	territory_layer_check.toggled.connect(func(pressed: bool): territory_view.visible = pressed)
	province_layer_check.toggled.connect(func(pressed: bool): province_boundary_view.visible = pressed)
	map_overlay.settlement_hovered.connect(_on_settlement_hovered)
	map_overlay.settlement_selected.connect(_on_settlement_selected)
	map_overlay.cursor_sampled.connect(_on_cursor_sampled)

	_build_workspace_tabs()
	_build_tool_rail()
	## Must precede _init_generation_params(): the Climate stage's `world`
	## row is a proxy onto this section's own Extent control (PROXY_KEYS), so
	## that control has to exist before any stage dialog can read it.
	_build_world_setup()
	## Must precede _build_menus(): the Generate menu asks each stage whether
	## any of its parameters actually resolved to a real WorldGen setter
	## before deciding whether that stage opens a dialog or stays inert.
	_init_generation_params()
	_build_menus()
	_select_tab("WORLD")


## ── World setup section (File ▸ New world) ──────────────────────────────
## Prepended to the New-world dialog's own section list, ahead of seed/sea
## level, world structure and the advanced fold, which stay exactly where
## they were. The dialog's two existing dimension controls (%ResolutionInput,
## %WidthInput) are re-parented into the new three-column rows rather than
## duplicated -- one node per value, so no two controls can disagree.
func _build_world_setup() -> void:
	_sized_api = world_gen.has_method("generate_sized") and world_gen.has_method("reference_grid_height")
	if not _sized_api:
		push_warning("cartalith: WorldGen has no generate_sized()/reference_grid_height() — World setup falls back to square maps.")

	var section := VBoxContainer.new()
	section.add_theme_constant_override("separation", 8)

	var header := Label.new()
	header.theme_type_variation = &"SectionHeader"
	header.text = "MAP SIZE, RESOLUTION & DIMENSIONS"
	section.add_child(header)
	section.add_child(_hint_label(
		"Map width in kilometres, working resolution in cells, and the frame's shape. Cells are square in kilometres, so the map's height in km is derived — width × rows ÷ columns — and is a readout below, never a separate control: a height that disagreed with it would silently contradict every distance, grade, river threshold and settlement spacing the world is generated from.",
		NEW_WORLD_HINT_WIDTH))

	## Extent — Region / Whole world. Same two-way choice as the reference's
	## own extentSeg, but this is the engine's `world` parameter, so it is
	## written through set_params() and proxied by the Climate stage dialog.
	extent_input = OptionButton.new()
	extent_input.add_item("Region")
	extent_input.add_item("Whole world")
	extent_input.selected = 1 if bool(world_gen.get_params().get("world", false)) else 0
	extent_input.tooltip_text = "Reference control #extentSeg. Region = a framed area with user-set latitudes; Whole world = a seamless equirectangular sheet with toroidal X wrap."
	extent_input.item_selected.connect(_on_extent_selected)
	section.add_child(_dim_row("Extent", extent_input, null))

	extent_note_label = _hint_label("", NEW_WORLD_HINT_WIDTH)
	section.add_child(extent_note_label)

	## Map width — preset scales beside the free km entry the reference has.
	size_preset_input = OptionButton.new()
	for preset: Dictionary in SIZE_PRESETS:
		size_preset_input.add_item(String(preset["label"]))
	size_preset_input.add_item("Custom")
	size_preset_input.tooltip_text = "Real-world width of the map. Creation-time only: the reference refuses to make this editable mid-project because changing it silently rescales every derived distance, grade, route length and settlement spacing."
	size_preset_input.item_selected.connect(_on_size_preset_selected)
	_reparent(width_input)
	width_input.value_changed.connect(func(_v: float): _refresh_dimensions())
	section.add_child(_dim_row("Map width (km)", size_preset_input, width_input))

	## Working resolution — the reference's own 512/1K/2K/4K/8K segment,
	## which sets the grid WIDTH only, plus free entry beside it.
	_reparent(resolution_input)
	resolution_input.item_selected.connect(_on_resolution_selected)
	grid_w_input = _grid_spin(RESOLUTION_PRESETS[RESOLUTION_DEFAULT_INDEX])
	grid_w_input.tooltip_text = "Grid columns. The reference's Working resolution segment sets this and nothing else; its grid height follows from gridH()."
	grid_w_input.value_changed.connect(func(_v: float): _refresh_dimensions())
	section.add_child(_dim_row("Resolution (columns)", resolution_input, grid_w_input))

	## Aspect — the shape of the frame, as a ratio preset or a free row count.
	aspect_input = OptionButton.new()
	for preset: Dictionary in ASPECT_PRESETS:
		aspect_input.add_item(String(preset["label"]))
	aspect_input.add_item("Custom")
	aspect_input.selected = ASPECT_DEFAULT_INDEX
	aspect_input.tooltip_text = "The frame's width:height. The reference has no aspect control — it hardcodes 2:1 in world mode and 1.5625:1 otherwise; both are here by name."
	aspect_input.item_selected.connect(func(_i: int): _refresh_dimensions())
	grid_h_input = _grid_spin(1311)
	grid_h_input.tooltip_text = "Grid rows. A call argument to generate_sized(), not a stored parameter: changing it reallocates every field in the pipeline."
	grid_h_input.value_changed.connect(_on_grid_h_changed)
	section.add_child(_dim_row("Aspect (rows)", aspect_input, grid_h_input))

	section.add_child(_build_derived_panel())

	dimension_warning_label = _hint_label("", NEW_WORLD_HINT_WIDTH)
	dimension_warning_label.add_theme_color_override("font_color", Color(0.878431, 0.639216, 0.290196))
	section.add_child(dimension_warning_label)

	new_world_list.add_child(section)
	new_world_list.move_child(section, 0)
	var divider := HSeparator.new()
	new_world_list.add_child(divider)
	new_world_list.move_child(divider, 1)

	## The two rows whose inputs moved up here are now empty shells, and the
	## static resolution hint has been replaced by the live warning label.
	_free_leftover("WorldParamsSection/ResolutionRow")
	_free_leftover("WorldParamsSection/ResolutionHint")
	_free_leftover("WorldParamsSection/WidthRow")
	var params_header := new_world_list.get_node_or_null("WorldParamsSection/WorldParamsHeader") as Label
	if params_header:
		params_header.text = "SEED & SEA LEVEL"

	if not _sized_api:
		aspect_input.disabled = true
		grid_h_input.editable = false
	_update_extent_state()


## A three-column setup row: label, preset chooser, free numeric entry. The
## second column is always the guided choice and the third always the exact
## one, so a reader learns the pattern once and it holds for all three rows.
func _dim_row(label_text: String, preset: Control, exact: Control) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.custom_minimum_size = Vector2(0, 40)
	row.add_theme_constant_override("separation", 10)

	var label := Label.new()
	label.custom_minimum_size = Vector2(140, 0)
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.text = label_text
	row.add_child(label)

	preset.custom_minimum_size = Vector2(0, 40)
	preset.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(preset)

	if exact != null:
		exact.custom_minimum_size = Vector2(130, 40)
		## Fixed width, so the guided column keeps the slack -- %WidthInput
		## arrives here carrying the scene's own EXPAND flag.
		exact.size_flags_horizontal = Control.SIZE_FILL
		row.add_child(exact)
	return row


func _grid_spin(initial: int) -> SpinBox:
	var spin := SpinBox.new()
	spin.min_value = GRID_MIN
	spin.max_value = GRID_MAX
	spin.step = 1
	spin.set_value_no_signal(initial)
	return spin


## Live derived readout — the part that makes the km/cell relationship
## legible instead of something a user infers after generating. Every row
## here is computed from exactly the same quotient the engine uses.
func _build_derived_panel() -> Control:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.0627451, 0.0666667, 0.0705882, 1)
	style.set_content_margin_all(10)
	panel.add_theme_stylebox_override("panel", style)

	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 16)
	grid.add_theme_constant_override("v_separation", 4)
	panel.add_child(grid)

	for key in ["Grid", "Extent", "Cell size", "Aspect"]:
		var k := Label.new()
		k.custom_minimum_size = Vector2(90, 0)
		k.add_theme_font_size_override("font_size", 11)
		k.add_theme_color_override("font_color", Color(0.552941, 0.576471, 0.588235))
		k.text = key
		grid.add_child(k)

		var v := Label.new()
		v.add_theme_font_size_override("font_size", 12)
		v.add_theme_color_override("font_color", Color(0.878431, 0.639216, 0.290196))
		v.text = "—"
		grid.add_child(v)
		_derived_labels[key] = v
	return panel


## Moves a scene-authored control into the runtime-built layout, keeping the
## one node (and so its `%` unique name, its @onready reference, and every
## existing connection) rather than making a second control for one value.
func _reparent(node: Control) -> void:
	var parent := node.get_parent()
	if parent:
		parent.remove_child(node)


func _free_leftover(path: String) -> void:
	var node := new_world_list.get_node_or_null(path)
	if node:
		node.queue_free()


## Extent is the engine's `world` parameter, so it is stored the same way
## every other parameter is. The Climate stage dialog's own `world` row is a
## PROXY_KEYS entry onto this control, so the two can never disagree.
func _on_extent_selected(index: int) -> void:
	world_gen.set_params({"world": index == 1})
	_refresh_param_control("world")
	_mark_params_dirty()
	_update_extent_state()


## World mode pins the aspect to 2:1 and says why on screen. The control is
## disabled rather than removed, and the note above it carries the physical
## reason -- a silently greyed control reads as a bug.
func _update_extent_state() -> void:
	var world := extent_input.selected == 1
	if world:
		aspect_input.selected = ASPECT_WORLD_INDEX
	if _sized_api:
		aspect_input.disabled = world
		grid_h_input.editable = not world
	extent_note_label.text = EXTENT_NOTE_WORLD if world else EXTENT_NOTE_REGION
	_refresh_dimensions()


func _on_size_preset_selected(index: int) -> void:
	if index < SIZE_PRESETS.size():
		width_input.set_value_no_signal(float(SIZE_PRESETS[index]["km"]))
	_refresh_dimensions()


func _on_resolution_selected(index: int) -> void:
	if index < RESOLUTION_PRESETS.size():
		grid_w_input.set_value_no_signal(RESOLUTION_PRESETS[index])
	_refresh_dimensions()


func _on_grid_h_changed(_value: float) -> void:
	if _dim_syncing:
		return
	## A hand-typed row count is by definition no longer one of the presets.
	aspect_input.selected = ASPECT_CUSTOM_INDEX
	_refresh_dimensions()


## The grid height the current extent + aspect selection implies. Both
## reference ratios are asked of the engine (WorldGen.reference_grid_height,
## the reference app's own gridH) rather than recomputed from the ratio in
## ASPECT_PRESETS, so the two constants live in exactly one place.
func _derived_grid_h(gw: int) -> int:
	if not _sized_api:
		return gw
	if extent_input.selected == 1:
		return world_gen.reference_grid_height(gw, true)
	if aspect_input.selected == ASPECT_REGION_INDEX:
		return world_gen.reference_grid_height(gw, false)
	var ratio := float(ASPECT_PRESETS[aspect_input.selected]["ratio"])
	return maxi(GRID_MIN, int(round(gw / ratio)))


## Single re-entrant-safe sync: preset buttons follow their free entry,
## the row count follows the aspect selection, and the readout follows both.
func _refresh_dimensions() -> void:
	if _dim_syncing:
		return
	_dim_syncing = true

	var gw := int(grid_w_input.value)
	resolution_input.selected = RESOLUTION_PRESETS.find(gw) if RESOLUTION_PRESETS.has(gw) else RESOLUTION_CUSTOM_INDEX

	var km := width_input.value
	var size_index := SIZE_CUSTOM_INDEX
	for i in SIZE_PRESETS.size():
		if is_equal_approx(float(SIZE_PRESETS[i]["km"]), km):
			size_index = i
	size_preset_input.selected = size_index

	var gh := int(grid_h_input.value)
	if aspect_input.selected != ASPECT_CUSTOM_INDEX:
		gh = _derived_grid_h(gw)
		grid_h_input.set_value_no_signal(gh)

	_dim_syncing = false
	_update_derived_readout(gw, gh, km)


func _update_derived_readout(gw: int, gh: int, km_w: float) -> void:
	## The one quotient the whole engine derives distances from.
	var cell_km := km_w / float(gw)
	var km_h := cell_km * gh
	var ratio := float(gw) / float(gh)

	_derived_labels["Grid"].text = "%d × %d cells   (%s)" % [gw, gh, _format_count(gw * gh)]
	_derived_labels["Extent"].text = "%s km × %s km" % [_format_km(km_w), _format_km(km_h)]
	_derived_labels["Cell size"].text = "%s km per cell, square" % _format_km(cell_km)
	var shape := "landscape" if ratio > 1.005 else ("portrait" if ratio < 0.995 else "square")
	_derived_labels["Aspect"].text = ("%.3f : 1 · %s" % [ratio, shape]) if ratio >= 1.0 \
		else ("1 : %.3f · %s" % [1.0 / ratio, shape])

	var warnings: Array[String] = []
	if gw >= 4096 or gh >= 4096:
		warnings.append("4K/8K grids are memory- and time-heavy on this port's CPU-only pipeline — a single generate at this size runs for minutes and allocates several GB.")
	if maxf(ratio, 1.0 / ratio) > DEGENERATE_ASPECT:
		warnings.append("Aspect ratios past about %d:1 are degenerate: the coarse weather grid has almost no resolution across the short axis and the plate frame (a uniform margin in cells) swallows a large fraction of the sheet. It generates without crashing, but the result is not a useful map." % int(DEGENERATE_ASPECT))
	dimension_warning_label.text = "\n".join(warnings)
	dimension_warning_label.visible = not warnings.is_empty()


func _format_count(n: int) -> String:
	if n >= 1000000:
		return "%.2f M cells" % (n / 1000000.0)
	return "%.1f k cells" % (n / 1000.0)


func _format_km(v: float) -> String:
	if v >= 100.0:
		return "%.0f" % v
	if v >= 1.0:
		return "%.1f" % v
	return "%.3f" % v


## ── Workspace tabs ──────────────────────────────────────────────────────
func _build_workspace_tabs() -> void:
	for tab_name in TAB_NAMES:
		var btn := Button.new()
		btn.text = tab_name
		btn.flat = true
		btn.custom_minimum_size = Vector2(0, 30)
		btn.add_theme_font_size_override("font_size", 10)
		btn.add_theme_color_override("font_color", Color(0.552941, 0.576471, 0.588235))
		btn.pressed.connect(_select_tab.bind(tab_name))
		_tab_buttons[tab_name] = btn
		## Insert before the spacer so the subtitle label stays right-aligned.
		tabs_row.add_child(btn)
		tabs_row.move_child(btn, tabs_row.get_child_count() - 3)


## Per UI_SHELL_DESIGN.md: "A tab swaps which tools and dock panels are
## shown around the same viewport -- it never swaps the application, and
## never changes the map." This only restyles the tab row, the workspace
## subtitle, and brightens the tool-rail group most relevant to the tab --
## no engine call, no viewport change.
func _select_tab(tab_name: String) -> void:
	_active_tab = tab_name
	for k in _tab_buttons:
		var btn: Button = _tab_buttons[k]
		btn.add_theme_color_override("font_color",
			Color(0.878431, 0.639216, 0.290196) if k == tab_name else Color(0.552941, 0.576471, 0.588235))
	workspace_subtitle_label.text = TAB_SUBTITLES.get(tab_name, "")

	var emphasis_group: int = TAB_TO_GROUP_INDEX.get(tab_name, -1)
	for gi in _tool_group_buttons.size():
		var dim := Color(0.372549, 0.392157, 0.407843)
		var normal := Color(0.552941, 0.576471, 0.588235)
		var col := normal if gi == emphasis_group else dim
		for btn: Button in _tool_group_buttons[gi]:
			if not btn.button_pressed:
				btn.add_theme_color_override("font_color", col)


## ── Left tool rail ──────────────────────────────────────────────────────
func _build_tool_rail() -> void:
	var group_button = ButtonGroup.new()
	var first_group := true
	for group in TOOL_GROUPS:
		if not first_group:
			var sep := Control.new()
			sep.custom_minimum_size = Vector2(22, 1)
			var sep_rect := ColorRect.new()
			sep_rect.color = Color(1, 1, 1, 0.10)
			sep_rect.custom_minimum_size = Vector2(22, 1)
			var sep_wrap := CenterContainer.new()
			sep_wrap.custom_minimum_size = Vector2(0, 9)
			sep_wrap.add_child(sep_rect)
			tool_rail_vbox.add_child(sep_wrap)
		var group_buttons: Array = []
		for tool: Dictionary in group:
			var btn := Button.new()
			btn.custom_minimum_size = Vector2(32, 32)
			btn.toggle_mode = true
			btn.button_group = group_button
			btn.flat = true
			btn.text = String(tool["glyph"])
			btn.add_theme_font_size_override("font_size", 15)
			btn.add_theme_color_override("font_color", Color(0.552941, 0.576471, 0.588235))
			var key_hint := "  (%s)" % tool["key"] if String(tool["key"]) != "" else ""
			btn.tooltip_text = "%s%s" % [tool["name"], key_hint]
			btn.pressed.connect(_on_tool_selected.bind(tool["name"], btn))
			group_buttons.append(btn)
			tool_rail_vbox.add_child(btn)
		_tool_group_buttons.append(group_buttons)
		first_group = false

	var rail_spacer := Control.new()
	rail_spacer.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tool_rail_vbox.add_child(rail_spacer)

	var prefs_btn := Button.new()
	prefs_btn.custom_minimum_size = Vector2(32, 32)
	prefs_btn.flat = true
	prefs_btn.disabled = true
	prefs_btn.text = "⚙"
	prefs_btn.add_theme_font_size_override("font_size", 15)
	prefs_btn.add_theme_color_override("font_disabled_color", Color(0.372549, 0.392157, 0.407843))
	prefs_btn.tooltip_text = "Tool preferences -- not implemented (no tool system exists yet)"
	tool_rail_vbox.add_child(prefs_btn)

	## Default active tool: Select / inspect, the rail's own first button.
	var first_btn: Button = _tool_group_buttons[0][0]
	first_btn.button_pressed = true
	_on_tool_selected(TOOL_GROUPS[0][0]["name"], first_btn)


func _on_tool_selected(tool_name: String, btn: Button) -> void:
	active_tool_label.text = String(tool_name).to_upper()
	## Status bar's own modifier-hints slot (UI_SHELL_DESIGN.md): honestly
	## reflects which tool is active without implying it does anything --
	## no pass-buffer/commit/discard exists yet (DCC_SHELL_SCOPE.md Track 2).
	status_hint_label.text = "%s selected -- no pass-buffer/commit/discard yet" % String(tool_name).to_upper()
	for group_buttons in _tool_group_buttons:
		for b: Button in group_buttons:
			b.add_theme_color_override("font_color",
				Color(0.878431, 0.639216, 0.290196) if b == btn else Color(0.552941, 0.576471, 0.588235))


## ── Top menu bar ────────────────────────────────────────────────────────
const ID_FILE_NEW_WORLD := 1
const ID_FILE_OPEN_PROJECT := 2
const ID_FILE_IMPORT_ASSET_PACK := 3
const ID_HELP_CREDITS := 1

func _build_menus() -> void:
	_build_file_menu()
	_build_edit_menu()
	_build_generate_menu()
	_build_simulate_menu()
	_build_render_menu()
	_build_assets_menu()
	_build_view_menu()
	_build_help_menu()


## Adds a disabled, honestly-inert item -- present per the shell's own
## "visibly present, not hidden" rule (GUI_SHELL_SCOPE.md), never a silent
## no-op: every disabled item keeps a tooltip naming why.
func _add_inert_item(popup: PopupMenu, text: String, tooltip: String = "not implemented yet") -> void:
	popup.add_item(text)
	var idx := popup.item_count - 1
	popup.set_item_disabled(idx, true)
	popup.set_item_tooltip(idx, tooltip)


func _build_file_menu() -> void:
	var popup := file_menu.get_popup()
	popup.add_item("New world", ID_FILE_NEW_WORLD)
	popup.add_item("Open project (.zip)...", ID_FILE_OPEN_PROJECT)
	popup.add_separator()
	_add_inert_item(popup, "Save", "No export writer exists yet -- Export .zip below is the closest real equivalent, also not yet implemented.")
	_add_inert_item(popup, "Save as")
	_add_inert_item(popup, "Recent")
	popup.add_separator()
	_add_inert_item(popup, "Import heightmap...", "No cartalith-io reader for this exists yet (GUI_FEATURE_PARITY_SCOPE.md Category 2).")
	popup.add_item("Import asset pack...", ID_FILE_IMPORT_ASSET_PACK)
	popup.add_separator()
	_add_inert_item(popup, "Export image/tiles")
	_add_inert_item(popup, "Export GeoJSON", "No Rust writer exists yet -- the underlying road/settlement/territory data is already real.")
	_add_inert_item(popup, "Export region")
	popup.add_separator()
	_add_inert_item(popup, "Project settings...")
	popup.id_pressed.connect(_on_file_menu_id)


func _on_file_menu_id(id: int) -> void:
	match id:
		ID_FILE_NEW_WORLD: new_world_dialog.popup_centered()
		ID_FILE_OPEN_PROJECT: load_save_dialog.popup_centered()
		ID_FILE_IMPORT_ASSET_PACK: asset_pack_dialog.popup_centered()


func _build_edit_menu() -> void:
	var popup := edit_menu.get_popup()
	_add_inert_item(popup, "Undo", "No undo system exists -- there is nothing to undo yet (no tool system, DCC_SHELL_SCOPE.md).")
	_add_inert_item(popup, "Redo")
	_add_inert_item(popup, "Undo history")
	popup.add_separator()
	_add_inert_item(popup, "Preferences")
	_add_inert_item(popup, "Theme", "Light/dark toggle -- the light-theme milestone is deferred (GUI_SHELL_SCOPE.md).")


func _build_generate_menu() -> void:
	var popup := generate_menu.get_popup()
	for i in GEN_STAGES.size():
		var stage: Dictionary = GEN_STAGES[i]
		if _stage_has_live_params(i):
			popup.add_item(String(stage["name"]) + "...", i)
			popup.set_item_tooltip(popup.item_count - 1, String(stage["note"]))
		else:
			## A stage with no engine-side parameters stays visibly present
			## and disabled, its tooltip naming the real reason -- the shell's
			## own "visibly present, not hidden" rule (GUI_SHELL_SCOPE.md).
			_add_inert_item(popup, String(stage["name"]),
				String(stage["note"]) if _params_available else PARAMS_MISSING_TIP)
	popup.add_separator()
	if _params_available:
		popup.add_item("Reset all generation parameters", ID_GEN_RESET_ALL)
	else:
		_add_inert_item(popup, "Reset all generation parameters", PARAMS_MISSING_TIP)
	popup.id_pressed.connect(_on_generate_menu_id)


func _build_simulate_menu() -> void:
	var popup := simulate_menu.get_popup()
	var tip := "The engine is a one-shot static generator, not a continuous simulation (HARDWARE_ACCELERATION.md's own scope correction)."
	for item in ["Time controls", "Collapse / recovery", "Economy", "Statistics", "Logistics"]:
		_add_inert_item(popup, item, tip)


func _build_render_menu() -> void:
	var popup := render_menu.get_popup()
	_add_inert_item(popup, "Map mode")
	_add_inert_item(popup, "Style preset")
	_add_inert_item(popup, "Terrain appearance...", "TERRAIN_APPEARANCE_SCOPE.md milestones 1-4 built real CPU-only rendering; no GUI exists yet.")
	_add_inert_item(popup, "Painter styles (NPR)")
	_add_inert_item(popup, "Lighting & shadows")
	popup.add_separator()
	_add_inert_item(popup, "3D viewport")
	_add_inert_item(popup, "Tiled LOD & atlas cache", "cartalith-spatial exists standalone, unintegrated (LOD_TILING_BASE_SCOPE.md).")
	_add_inert_item(popup, "Render quality")
	popup.add_separator()
	_add_inert_item(popup, "Bake image / tiles...")


func _build_assets_menu() -> void:
	var popup := assets_menu.get_popup()
	_add_inert_item(popup, "Asset library")
	_add_inert_item(popup, "Sprite sheet slicer")
	_add_inert_item(popup, "Asset pack (validate / export)", "Import is real -- see File > Import asset pack. Validate/export have no Rust backing yet.")
	_add_inert_item(popup, "Assets by domain")


func _build_view_menu() -> void:
	var popup := view_menu.get_popup()
	_add_inert_item(popup, "Panel visibility")
	_add_inert_item(popup, "Workspace tabs")
	_add_inert_item(popup, "Analysis field overlay")
	_add_inert_item(popup, "Performance readout", "No live CPU/GPU/memory #[func] exists yet (GUI_FEATURE_PARITY_SCOPE.md).")


func _build_help_menu() -> void:
	var popup := help_menu.get_popup()
	popup.add_item("Credits & academic principles", ID_HELP_CREDITS)
	_add_inert_item(popup, "References")
	_add_inert_item(popup, "Keyboard map")
	popup.id_pressed.connect(_on_help_menu_id)


func _on_help_menu_id(id: int) -> void:
	if id == ID_HELP_CREDITS:
		credits_dialog.popup_centered()


## ── Generate menu: per-stage parameter dialogs ──────────────────────────
## DCC_SHELL_SCOPE.md milestone 2 (GUI half). UI_SHELL_DESIGN.md's Generate
## menu spec: "The pipeline stages in order [...] each opens its parameter
## dialog". Dialogs, never persistent panels -- that document's governing
## rule for the whole menu bar.
##
## The five-level disclosure grammar (design/Cartalith Menu Structure v2):
## menu bar (1) -> Generate menu (2) -> a stage's dialog (3) -> a section
## inside it (4) -> that section's collapsed ADVANCED fold (5), holding
## "only dials whose defaults are already correct".
##
## ── Where the numbers come from ─────────────────────────────────────────
## Nowhere in this file. Every range, step, label, unit and default is read
## at runtime from WorldGen.get_param_info() / get_params(), which the Rust
## side builds from `cartalith-godot/src/params.rs`'s PARAMS table -- itself
## derived from the reference HTML's own controls put through their own
## mapping functions, and from cartalith_engine::WorldParams::defaults. That
## table's doc comment states the reason directly: a GDScript copy of 59
## ranges is 59 chances for a slider to silently drift from the range the
## reference actually shipped. So this file carries only what the Rust table
## has no opinion about -- which Generate-menu STAGE a parameter group
## belongs to, which rows are level-5 Advanced, and the prose.
##
## ── Staleness: deliberate decision, recorded rather than faked ──────────
## UI_SHELL_DESIGN.md says each stage "reports staleness". No staleness
## system exists (UNIFIED_TOOL_PLAN.md milestone A, unbuilt), and more
## fundamentally the engine is a ONE-SHOT generator: generate_terrain() runs
## the whole pipeline or none of it, so there is no per-stage incremental
## recompute for a stage to be stale *relative to*. A per-stage "stale" pip
## would advertise exactly the incremental pipeline that does not exist.
## So: no per-stage staleness indicators. Instead every dialog carries an
## honest regenerate-to-apply affordance -- a footer line that says plainly
## that the whole world is regenerated, a status-bar note when a parameter
## has changed since the last generate, and a real Generate now button that
## runs the same single full pass File > New World's Generate runs.

const ID_GEN_RESET_ALL := 1000

## Stage -> which params.rs groups (and which individual keys) it owns.
## `groups` are pulled whole; `keys` pull single parameters out of a group
## that is otherwise split across stages (the Rust table's "world" group is
## world setup + hydrology + a GPU switch, three different stages' worth).
const GEN_STAGES: Array = [
	{
		"name": "Tectonics",
		"groups": ["tectonics", "world_structure"],
		"keys": [],
		"note": "Plate layout, boundary stress and the base height field. Reference: the Tectonics and World Structure panels.",
		"gaps": "Not exposed: the reference's graph-driven orogeny switch (state.tect.tectonicGraph, omitted by cartalith-engine) and its three dials — Fold intensity, Trench depth, Fault blocks. generate_terrain hardcodes those three to the exact values the reference's own defaults produce, so behaviour matches; surfacing them needs three new fields threaded through OrogenyParams (GENERATION_PARAMETERS.md).",
	},
	{
		"name": "Volcanism",
		"groups": ["volcanism"],
		"keys": [],
		"note": "Volcanic cones, provinces and impact craters stamped onto the height field. Reference: the Volcanism & impacts panel.",
		"gaps": "",
	},
	{
		"name": "Erosion",
		"groups": ["erosion"],
		"keys": [],
		"note": "The stream-power incision pass that runs inside generation. Reference: Erosion > Stream-power carve.",
		"gaps": "Not ported: Droplet hydraulic, Hillslope diffuse, Velocity (momentum) and Evolve & sediment. Each is a separate manual erosion op in the HTML app with no cartalith-engine equivalent, so their dials are absent rather than inert.",
	},
	{
		"name": "Glacial & coastal",
		"groups": [],
		"keys": [],
		"note": "Not ported. The reference's glacial-erosion pass (snowline, U-width, cirques, fjords) and its coastal pass (sea cliffs, estuaries, tidal marsh) have no cartalith-engine equivalent, so there is nothing to parameterise yet.",
		"gaps": "",
	},
	{
		"name": "Hydrology",
		"groups": [],
		"keys": ["carve_rivers", "river_density"],
		"note": "River-network extraction and valley carving. Reference: the carve-on-generation switch plus the Rivers panel's density control.",
		"gaps": "Min stream order is a reference render filter, not a generation parameter -- it belongs with the Render menu's map-mode work, not here.",
	},
	{
		"name": "Climate",
		"groups": ["planet", "climate", "weather"],
		"keys": ["world", "peak_m"],
		"note": "Planet setup, the temperature field, and the moisture/weather simulation. Reference: the Planet, Climate & biomes and Weather panels.",
		"gaps": "Not ported: geoid and tides (both default-off sub-systems in the reference), Seasons, and Koppen-Geiger classification.",
	},
	{
		"name": "Ecology",
		"groups": [],
		"keys": [],
		"note": "Not parameterised. Biome classification runs off the finished temperature/rainfall/elevation fields with no dials of its own in cartalith-engine; the reference's Ecology panel is likewise all render-side.",
		"gaps": "",
	},
	{
		"name": "Settlements",
		"groups": [],
		"keys": [],
		"note": "Suitability scoring and settlement placement (compute_civilisation). Reference: the Civilization panel's generation half.",
		"gaps": "The reference exposed no numeric dials for civ generation beyond village seeding -- the suitability weights are constants in both engines.",
	},
	{
		"name": "Infrastructure",
		"groups": [],
		"keys": [],
		"note": "Not parameterised. Roads and sea routes are derived from the settlement set and the travel-cost field with no dials in cartalith-civ.",
		"gaps": "",
	},
	{
		"name": "Politics",
		"groups": [],
		"keys": [],
		"note": "Not parameterised. Territory and province assignment are derived from settlements and travel cost with no dials in cartalith-civ.",
		"gaps": "",
	},
]

## Section headings, matching the reference HTML's own panel headings.
const GROUP_TITLES := {
	"tectonics": "PLATES & UPLIFT",
	"world_structure": "WORLD STRUCTURE",
	"volcanism": "VOLCANISM & IMPACTS",
	"erosion": "STREAM-POWER CARVE",
	"planet": "PLANET",
	"climate": "CLIMATE & TEMPERATURE",
	"weather": "WEATHER · RAINFALL SIM",
	"world": "WORLD & SCALE",
}

## Disclosure level 5 -- "only dials whose defaults are already correct".
## The rule, rather than taste: a parameter is Advanced if the reference
## itself buried it (its Physical coupling fields <details class="adv">
## block, and dynamic lithology inside the Evolve & sediment accordion), or
## if the reference never exposed it at all and this port surfaces it as a
## superset (DECISIONS.md 7d). Everything else stays visible.
const ADVANCED_KEYS: Array[String] = [
	"tect.flexure", "tect.hetero", "tect.resist",
	"tect.dynamic_lithology", "tect.lloyd",
	"climate.current_k", "climate.terrain_wind_deflection",
	"climate.ocean_hum", "climate.bulk_evap",
]

## Parameters deliberately NOT given a Generate-menu row, each with its
## reason. Left out is a decision; silently dropping one would not be.
const EXCLUDED_KEYS := {
	"sea_level": "Owned by File > New World, which already drives it through set_sea_level(). Duplicating it here would create two controls for one value.",
	"use_gpu": "GPU_LAYER_INTEGRATION_SCOPE.md's current milestone is still the GPU-safe noise redesign; per DECISIONS.md 7c the GPU path produces a different world for the same seed. Surfacing the switch before that lands would expose an incomplete path (GUI_FEATURE_PARITY_SCOPE.md Category-1 item #7, deferred again here).",
}

## Parameters whose single source of truth is an existing, already-wired
## scene control rather than this dialog's own state. The four experimental
## flags are pushed to the engine by _on_generate_pressed's
## set_experimental_flags() call and villages by set_villages_enabled();
## these rows drive those CheckBoxes directly so the two surfaces can never
## disagree about one value.
## `world` joined them when File ▸ New world grew its own Extent control:
## extent is a creation-time shape decision, so it belongs in the setup
## dialog, but it is also a real generation parameter the Climate stage
## legitimately shows. Proxying keeps one node behind both surfaces.
const PROXY_KEYS := {
	"tect.dynamic_lithology": "dynamic_lithology_check",
	"volc.provinces": "volc_provinces_check",
	"climate.terrain_wind_deflection": "wind_deflection_check",
	"climate.currents": "ocean_currents_check",
	"world": "extent_input",
}

## Rows with no params.rs entry at all -- a real engine capability reached
## through its own older #[func], given a home in the stage it belongs to.
const EXTRA_ROWS := {
	"Settlements": [{
		"key": "_villages", "group": "settlements", "type": "bool", "default": false,
		"label": "Village seeding (additive hamlets)", "unit": "",
		"proxy": "villages_check", "reference_control": "civVillagesChk",
		"hint": "Reference _civVillages, default off. Seeds an extra tier of hamlets after the main settlement pass.",
	}],
}

var _param_info: Dictionary = {} ## key -> the info Dictionary from Rust (plus GUI-only extras)
var _param_defaults: Dictionary = {} ## key -> the engine's own default
var _stage_rows: Dictionary = {} ## stage index -> Array of info Dictionaries, in display order
var _param_controls: Dictionary = {} ## key -> [control, value Label or null]
var _stage_dialogs: Dictionary = {} ## stage index -> AcceptDialog
var _stage_footers: Dictionary = {} ## stage index -> Label
var _params_available := false
var _params_dirty := false

const STALE_TEXT_CLEAN := "Cartalith is a one-shot generator: parameters take effect on the next full generate, not incrementally."
const STALE_TEXT_DIRTY := "Parameters changed since the last generate — press Generate now (or File > New World > Generate) to apply them. The whole world is regenerated; there is no per-stage recompute."
const PARAMS_MISSING_TIP := "WorldGen exposes no get_param_info() yet, so no live parameter table can be read from the engine. Rebuild the GDExtension once the generation-parameter API has landed."


## Reads the engine's own parameter table and sorts it into stages. Runs
## before _build_menus(), which asks each stage whether it has any real row
## before deciding to offer a dialog or stay inert.
func _init_generation_params() -> void:
	_params_available = world_gen.has_method("get_param_info") and world_gen.has_method("set_params")
	if not _params_available:
		push_warning("cartalith: WorldGen has no get_param_info()/set_params() — Generate-menu stages stay inert.")
		return

	## get_param_info() is key -> {group, type, default, min, max, step,
	## label, unit, reference_control}. The key lives outside the row, so it
	## is folded in here; every row then carries everything a control needs.
	var engine_info: Dictionary = world_gen.get_param_info()
	for key: String in engine_info:
		if EXCLUDED_KEYS.has(key):
			continue
		var info: Dictionary = (engine_info[key] as Dictionary).duplicate()
		info["key"] = key
		if PROXY_KEYS.has(key):
			info["proxy"] = PROXY_KEYS[key]
		_param_info[key] = info

	_param_defaults = world_gen.get_param_defaults()

	for i in GEN_STAGES.size():
		var stage: Dictionary = GEN_STAGES[i]
		var rows: Array = []
		## Dictionary iteration follows insertion order, and the engine builds
		## its table in the order the GUI should show it -- so this preserves
		## the reference's own within-panel ordering for free.
		for group_name: String in stage["groups"]:
			for key: String in _param_info:
				if String((_param_info[key] as Dictionary)["group"]) == group_name:
					rows.append(_param_info[key])
		for key: String in stage["keys"]:
			if _param_info.has(key):
				rows.append(_param_info[key])
		for extra: Dictionary in EXTRA_ROWS.get(String(stage["name"]), []):
			_param_info[String(extra["key"])] = extra
			rows.append(extra)
		_stage_rows[i] = rows

	var placed := 0
	for i: int in _stage_rows:
		placed += (_stage_rows[i] as Array).size()
	print("cartalith: generation parameters — %d exposed by the engine, %d deliberately excluded, %d rows across the Generate menu" % [
		engine_info.size(), EXCLUDED_KEYS.size(), placed])


func _stage_has_live_params(index: int) -> bool:
	return not (_stage_rows.get(index, []) as Array).is_empty()


func _on_generate_menu_id(id: int) -> void:
	if id == ID_GEN_RESET_ALL:
		_reset_params([])
		return
	if id >= 0 and id < GEN_STAGES.size():
		_open_stage_dialog(id)


## ── Reading and writing one parameter ───────────────────────────────────
func _param_get(key: String):
	var info: Dictionary = _param_info[key]
	if info.has("proxy"):
		var node: Control = get(String(info["proxy"]))
		if node is CheckBox:
			return node.button_pressed
		## The Extent OptionButton's two items are Region / Whole world, in
		## the same order as the `world` parameter's false / true.
		if node is OptionButton:
			return node.selected == 1
		return node.value
	return world_gen.get_params().get(key, _param_defaults.get(key, 0.0))


## Engine-side type tag: "bool" | "int" | "float" (params.rs Kind::as_str).
func _param_is_bool(info: Dictionary) -> bool:
	return String(info.get("type", "float")) == "bool"


func _param_set(key: String, value) -> void:
	var info: Dictionary = _param_info[key]
	if info.has("proxy"):
		## One source of truth: drive the existing scene control, whose own
		## already-wired handler is what reaches the engine.
		var node: Control = get(String(info["proxy"]))
		if node is CheckBox:
			node.button_pressed = bool(value)
		elif node is OptionButton:
			## Assigning `selected` does not emit item_selected, so the
			## handler that actually reaches the engine is called directly.
			node.selected = 1 if bool(value) else 0
			_on_extent_selected(node.selected)
			return
		else:
			node.value = value
		_mark_params_dirty()
		return
	var clamped: Dictionary = world_gen.set_params({key: value})
	## set_params reports what it actually stored. If it clamped or rejected
	## the value, echo the engine's own value back into the widget rather
	## than leaving the GUI claiming something the engine did not accept.
	if clamped.get("clamped", []).has(key) or clamped.get("rejected", []).has(key):
		_refresh_param_control(key)
	_mark_params_dirty()


## The honest half of UI_SHELL_DESIGN.md's "reports staleness" — see this
## section's own header comment for why there is nothing per-stage to report.
func _mark_params_dirty() -> void:
	_params_dirty = true
	shell_status_label.text = "generation parameters changed — regenerate to apply"
	for i: int in _stage_footers:
		(_stage_footers[i] as Label).text = STALE_TEXT_DIRTY


func _clear_params_dirty() -> void:
	_params_dirty = false
	for i: int in _stage_footers:
		(_stage_footers[i] as Label).text = STALE_TEXT_CLEAN


## Restores cartalith_engine::WorldParams::defaults — the reference app's own
## `state` literal — for the given keys, or for every exposed parameter when
## `keys` is empty.
func _reset_params(keys: Array) -> void:
	var targets: Array = keys if not keys.is_empty() else _param_info.keys()
	if keys.is_empty():
		## The engine has its own whole-table reset; use it rather than
		## replaying 58 values back at it.
		world_gen.reset_params()
	var batch := {}
	for key: String in targets:
		var info: Dictionary = _param_info[key]
		var fallback = _param_defaults.get(key, info.get("default", _param_get(key)))
		if info.has("proxy"):
			_param_set(key, fallback)
		elif not keys.is_empty():
			batch[key] = fallback
	if not batch.is_empty():
		world_gen.set_params(batch)
	_refresh_all_param_controls()
	_mark_params_dirty()
	shell_status_label.text = "generation parameters reset to defaults — regenerate to apply"


func _refresh_all_param_controls() -> void:
	for key: String in _param_controls:
		_refresh_param_control(key)


func _refresh_param_control(key: String) -> void:
	if not _param_controls.has(key):
		return
	var pair: Array = _param_controls[key]
	var value = _param_get(key)
	var control: Control = pair[0]
	if control is CheckBox:
		control.set_pressed_no_signal(bool(value))
	else:
		control.set_value_no_signal(float(value))
		(pair[1] as Label).text = _format_param(_param_info[key], value)


## Decimal places follow the parameter's own step, so a 0.0003-step dial
## reads 0.012 and a 1.0-step dial reads 70 — no per-parameter format string,
## and nothing here to drift from the Rust table.
func _format_param(info: Dictionary, value) -> String:
	var step := float(info.get("step", 1.0))
	var digits := 0 if step >= 1.0 else (1 if step >= 0.1 else (2 if step >= 0.01 else 3))
	## "%.*f" rather than String.num(), which trims trailing zeros -- the
	## reference's own readouts are toFixed(2), so 0.60 must read "0.60".
	var text: String = ("%." + str(digits) + "f") % float(value)
	var unit := String(info.get("unit", ""))
	if unit.is_empty():
		return text
	## The reference writes its multiplier unit in front (×1.00), everything
	## else after (23.4°, 4000 m).
	return unit + text if unit == "×" else text + (" " if unit.length() > 1 else "") + unit


## ── Building a stage dialog ─────────────────────────────────────────────
func _open_stage_dialog(index: int) -> void:
	if not _stage_dialogs.has(index):
		_stage_dialogs[index] = _build_stage_dialog(index)
	_refresh_all_param_controls()
	(_stage_dialogs[index] as AcceptDialog).popup_centered(DIALOG_SIZE)


func _build_stage_dialog(index: int) -> AcceptDialog:
	var stage: Dictionary = GEN_STAGES[index]
	var dialog := AcceptDialog.new()
	dialog.title = "Generate — %s" % stage["name"]
	dialog.theme = theme
	## AcceptDialog defaults wrap_controls on, which grows the window to its
	## content's full minimum height -- a 20-row stage would run off a 1080p
	## screen and take its own footer buttons with it. Off, so the fixed size
	## below holds and the ScrollContainer does the scrolling it is there for.
	dialog.wrap_controls = false
	add_child(dialog)

	var margin := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 8)
	dialog.add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 10)
	margin.add_child(root)

	root.add_child(_hint_label(String(stage["note"])))
	if not String(stage["gaps"]).is_empty():
		root.add_child(_hint_label(String(stage["gaps"])))

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	var list := VBoxContainer.new()
	list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	list.add_theme_constant_override("separation", 14)
	scroll.add_child(list)

	## Disclosure level 4 = a section per params.rs group, in the order the
	## Rust table lists them. Level 5 = each section's collapsed ADVANCED
	## fold, present only when that section actually has advanced rows.
	var sections: Array[String] = []
	var by_group: Dictionary = {}
	for info: Dictionary in _stage_rows[index]:
		var g := String(info["group"])
		if not by_group.has(g):
			by_group[g] = []
			sections.append(g)
		(by_group[g] as Array).append(info)

	for g: String in sections:
		var header := Label.new()
		header.theme_type_variation = &"SectionHeader"
		header.text = String(GROUP_TITLES.get(g, g.to_upper()))
		list.add_child(header)

		var body := VBoxContainer.new()
		body.add_theme_constant_override("separation", 8)
		list.add_child(body)

		var advanced: Array = []
		for info: Dictionary in by_group[g]:
			if ADVANCED_KEYS.has(String(info["key"])):
				advanced.append(info)
			else:
				body.add_child(_build_param_row(info))

		if not advanced.is_empty():
			var fold := FoldableContainer.new()
			fold.title = "ADVANCED"
			fold.folded = true
			fold.tooltip_text = "Dials whose defaults are already correct — buried in the reference too, or never exposed by it at all."
			body.add_child(fold)
			var adv_body := VBoxContainer.new()
			adv_body.add_theme_constant_override("separation", 8)
			fold.add_child(adv_body)
			for info: Dictionary in advanced:
				adv_body.add_child(_build_param_row(info))

	root.add_child(HSeparator.new())

	var footer := _hint_label(STALE_TEXT_DIRTY if _params_dirty else STALE_TEXT_CLEAN)
	root.add_child(footer)
	_stage_footers[index] = footer

	var buttons := HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 8)
	root.add_child(buttons)

	var reset := Button.new()
	reset.text = "Reset this stage"
	reset.custom_minimum_size = Vector2(0, 40)
	reset.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	reset.tooltip_text = "Restores cartalith_engine::WorldParams::defaults for this stage's parameters — the reference app's own state literal."
	var stage_keys: Array = []
	for info: Dictionary in _stage_rows[index]:
		stage_keys.append(String(info["key"]))
	reset.pressed.connect(_reset_params.bind(stage_keys))
	buttons.add_child(reset)

	var gen := Button.new()
	gen.text = "Generate now"
	gen.custom_minimum_size = Vector2(0, 40)
	gen.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	gen.theme_type_variation = &"PrimaryButton"
	gen.tooltip_text = "Runs the same single full generation pass File > New World's Generate runs — the whole pipeline, not just this stage."
	gen.pressed.connect(_on_generate_pressed)
	buttons.add_child(gen)

	return dialog


const DIALOG_SIZE := Vector2i(620, 840)
## The New-world dialog is wider (660 in main.tscn) and its own margins and
## scrollbar eat a little more, so its hints wrap at their own width.
const NEW_WORLD_HINT_WIDTH := 590

## An autowrap Label with no width constraint reports a minimum height for
## wrapping at its longest-word width -- hundreds of lines for a paragraph,
## which drags the whole dialog past the bottom of a 1080p screen and takes
## its own footer buttons with it. Pinning the wrap width fixes the min
## height at the height it will actually render.
func _hint_label(text: String, wrap_width: int = DIALOG_SIZE.x - 60) -> Label:
	var label := Label.new()
	label.theme_type_variation = &"HintLabel"
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size = Vector2(wrap_width, 0)
	label.text = text
	return label


func _build_param_row(info: Dictionary) -> Control:
	var key := String(info["key"])
	var hint := String(info.get("hint", ""))
	var ref := String(info.get("reference_control", ""))
	if hint.is_empty():
		hint = ("Reference control #%s." % ref) if not ref.is_empty() else \
			"Not exposed by the reference app — surfaced here as a superset, at the engine's own default."

	if _param_is_bool(info):
		var check := CheckBox.new()
		check.text = String(info["label"])
		## Width pinned for the same reason _hint_label pins its own.
		check.custom_minimum_size = Vector2(DIALOG_SIZE.x - 90, 36)
		check.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		check.tooltip_text = hint
		check.set_pressed_no_signal(bool(_param_get(key)))
		check.toggled.connect(func(pressed: bool): _param_set(key, pressed))
		_param_controls[key] = [check, null]
		return check

	var row := HBoxContainer.new()
	row.custom_minimum_size = Vector2(0, 36)
	row.add_theme_constant_override("separation", 10)

	var label := Label.new()
	label.custom_minimum_size = Vector2(140, 0)
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.text = String(info["label"])
	label.tooltip_text = hint
	row.add_child(label)

	var slider := HSlider.new()
	slider.min_value = float(info["min"])
	slider.max_value = float(info["max"])
	slider.step = float(info["step"])
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	slider.tooltip_text = hint
	slider.set_value_no_signal(float(_param_get(key)))
	row.add_child(slider)

	var value_label := Label.new()
	value_label.custom_minimum_size = Vector2(78, 0)
	value_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	value_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	## Same numeric-readout treatment the shell's ReadoutLabel/
	## CoordinatesLabel already use — dark_theme.tres has no monospace
	## variation to reference, so this follows the existing convention rather
	## than inventing a theme entry (UI_SHELL_DESIGN.md's monospace readouts
	## are a later theme pass, not this milestone).
	value_label.add_theme_color_override("font_color", Color(0.784314, 0.796078, 0.803922))
	value_label.add_theme_font_size_override("font_size", 11)
	value_label.text = _format_param(info, _param_get(key))
	row.add_child(value_label)

	var is_int := String(info.get("type", "float")) == "int"
	slider.value_changed.connect(func(v: float):
		value_label.text = _format_param(info, v)
		_param_set(key, int(round(v)) if is_int else v))

	_param_controls[key] = [slider, value_label]
	return row


## ── Generation ───────────────────────────────────────────────────────────
func _term_strength(value: float) -> String:
	if value >= 0.75:
		return "strong"
	if value >= 0.45:
		return "moderate"
	if value > 0.05:
		return "weak"
	return "negligible"


func _describe_term(t: Dictionary) -> String:
	var key := String(t["key"])
	var label: String = SUIT_TERM_LABELS.get(key, key.replace("_", " "))
	return "%s %s (%.2f)" % [_term_strength(float(t["value"])), label, float(t["value"])]


## Builds the causal "why here?" chain for the Properties dock (pinned
## selection). Same underlying data/logic the prior shell's Inspector used
## on hover; now used on click instead (Category-1 item #10).
func _build_causal_chain_text(s: Dictionary, index: int) -> String:
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := [
		"[b]%s[/b] (%s)" % [s["name"], kind_label],
		"Population: %s" % s["population"],
		"Faction: %d" % s["faction"],
		"Coastal: %s" % ("yes" if s["coastal"] else "no"),
		"Capital: %s" % ("yes" if s["capital"] else "no"),
	]

	var why: Dictionary = world_gen.explain_settlement(index)
	if not why.is_empty():
		lines.append("")
		lines.append("[b]WHY HERE?[/b]")
		if why.has("excluded"):
			lines.append("Cell excluded from suitability (%s)." % why["excluded"])
		else:
			var terms: Array = why["terms"]
			var positives: Array[String] = []
			var negatives: Array[String] = []
			for t: Dictionary in terms:
				var c := float(t["contribution"])
				if c > 0.005 and positives.size() < 3:
					positives.append(_describe_term(t))
				elif c < -0.005 and negatives.size() < 2:
					negatives.append(_describe_term(t))
			if positives.is_empty():
				lines.append("No single factor stands out -- placed on broadly average ground.")
			else:
				lines.append(" → ".join(positives))
			if not negatives.is_empty():
				lines.append("Despite: %s" % ", ".join(negatives))
			lines.append("Suitability %.2f" % float(why["score"]))

		lines.append("")
		var ord_i := int(why["river_order"])
		var river_txt := ("Strahler %d" % ord_i) if ord_i > 0 else "none"
		var coast_cells := float(why["coast_dist_cells"])
		lines.append("River: %s · flow %.0f" % [river_txt, float(why["flow"])])
		lines.append("Distance to water: %.1f cells" % coast_cells)
		lines.append("Elevation: %.3f (normalised)" % float(why["elevation"]))
		lines.append("Travel cost: %.2f" % float(why["travel_cost"]))

	return "\n".join(lines)


## Properties dock: pinned selection (click-to-pin, Category-1 item #10).
## Independent of the transient hover state Sample owns below.
func _on_settlement_selected(data: Variant, index: int) -> void:
	_selected_settlement = data
	_selected_index = index
	if data == null:
		properties_header.text = "PROPERTIES"
		properties_body.text = "No tool active and no selection. Click a settlement on the map to inspect it. A full per-cell property inspector (elevation, slope, aspect, etc. under an active tool) needs the tool system this milestone doesn't add -- see DCC_SHELL_SCOPE.md."
		return
	properties_header.text = "PROPERTIES · SETTLEMENT"
	properties_body.text = _build_causal_chain_text(data, index)


## Sample dock: live, transient hover data -- lighter than Properties'
## pinned causal chain, and never invents fields (elevation/slope/biome/
## drainage under the cursor) the engine doesn't expose per-cell yet.
func _on_settlement_hovered(data: Variant, index: int) -> void:
	_hover_settlement = data
	_refresh_sample_panel()


func _on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	_cursor_valid = valid
	_cursor_gx = gx
	_cursor_gy = gy
	coordinates_label.text = ("%.0f E · %.0f N (cell)" % [gx, gy]) if valid else ""
	_refresh_sample_panel()


func _refresh_sample_panel() -> void:
	var lines: Array[String] = []
	if _hover_settlement != null:
		var s: Dictionary = _hover_settlement
		var kind_label: String = String(s["kind"]).capitalize()
		lines.append("[b]%s[/b] (%s)" % [s["name"], kind_label])
		lines.append("Population %s" % s["population"])
		lines.append("Faction %d" % s["faction"])
		lines.append("Coastal: %s" % ("yes" if s["coastal"] else "no"))
		lines.append("Capital: %s" % ("yes" if s["capital"] else "no"))
	elif _cursor_valid:
		lines.append("X %.0f    Y %.0f (grid cell)" % [_cursor_gx, _cursor_gy])
		lines.append("")
		lines.append("Per-cell fields (elevation, slope, biome, etc.) need a new engine query this milestone doesn't add -- only cursor position and settlement hover data are real today.")
	else:
		lines.append("Hover the map to sample.")
	sample_body.text = "\n".join(lines)


func _on_generate_pressed() -> void:
	if _generating:
		return
	_generating = true
	generate_button.disabled = true
	status_label.text = "generating..."
	readout_label.text = "generating..."
	shell_status_label.text = "generating..."

	var seed_value := int(seed_input.value)
	## Grid dimensions come from the two SpinBoxes, not from the preset
	## OptionButtons: the buttons are a way of writing those SpinBoxes, and
	## "Custom" has no preset value to read at all.
	var grid_w := int(grid_w_input.value)
	var grid_h := int(grid_h_input.value)
	var width_km := width_input.value
	var archetype := WORLD_SHAPES[world_shape_input.selected]

	world_gen.set_experimental_flags(
		dynamic_lithology_check.button_pressed,
		volc_provinces_check.button_pressed,
		wind_deflection_check.button_pressed,
		ocean_currents_check.button_pressed,
	)
	world_gen.set_villages_enabled(villages_check.button_pressed)
	world_gen.set_sea_level(sea_level_input.value / 100.0)
	## Every Generate-menu parameter is already stored on the engine's own
	## WorldParams by set_params() at the moment its dial moved -- there is
	## nothing to re-push here. The five lines above stay because their five
	## values live on File > New World's controls, not in the parameter table
	## (PROXY_KEYS/EXCLUDED_KEYS record which and why).

	_gen_thread = Thread.new()
	_gen_thread.start(_generate_worker.bind(seed_value, width_km, grid_w, grid_h, archetype))


## Runs off the main thread. Touches only world_gen (plain Rust state),
## never a node. generate_sized() and generate_world_structure_sized() are
## both full, equally expensive generate_terrain() calls that mutate the same
## world_gen state -- this must be the ONE call site.
##
## The archetype branch is load-bearing and was silently broken once before
## (fixed in a265b2b): a non-empty WORLD_SHAPES entry must reach
## generate_world_structure_sized, or the World-shape choice never affects
## generation at all. Its bool return is the archetype-name check, surfaced
## as a real failure below rather than swallowed.
func _generate_worker(seed_value: int, width_km: float, grid_w: int, grid_h: int, archetype: String) -> void:
	var ok := true
	if archetype.is_empty():
		if _sized_api:
			world_gen.generate_sized(seed_value, width_km, grid_w, grid_h)
		else:
			world_gen.generate(seed_value, width_km, grid_w)
	elif _sized_api:
		ok = world_gen.generate_world_structure_sized(seed_value, width_km, grid_w, grid_h, archetype)
	else:
		ok = world_gen.generate_world_structure(seed_value, width_km, grid_w, archetype)
	_on_generate_done.call_deferred(seed_value, width_km, ok)


func _on_generate_done(seed_value: int, width_km: float, ok: bool) -> void:
	_gen_thread.wait_to_finish()
	_gen_thread = null

	if not ok:
		status_label.text = "generate failed — see console"
		readout_label.text = "generate failed"
		shell_status_label.text = "generate failed — see console"
		generate_button.disabled = false
		_generating = false
		return

	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		var settlements := world_gen.get_settlements()
		var roads := world_gen.get_roads()
		var sea_routes := world_gen.get_sea_routes()
		map_overlay.set_civ_data(settlements, roads, sea_routes, world_gen.get_width(), world_gen.get_height(), world_gen.get_border_inset_frac())
		territory_view.texture = world_gen.build_territory_texture()
		province_boundary_view.texture = world_gen.build_province_boundary_texture()

		## Read the real extent back from the engine rather than echoing what
		## the dialog asked for: get_map_height_km() is derived from the world
		## actually built (map_width_km * gh / gw), so a mismatch between this
		## line and the setup dialog's own readout is a real bug, visible.
		var real_w: float = world_gen.get_map_width_km() if _sized_api else width_km
		var real_h: float = world_gen.get_map_height_km() if _sized_api else width_km
		_last_width_km = real_w
		_update_scale_bar()

		var shape_label := world_shape_input.get_item_text(world_shape_input.selected)
		var extent_label := extent_input.get_item_text(extent_input.selected)
		var civ_note := ", %d settlements" % settlements.size() if not settlements.is_empty() else ""
		var summary := "%dx%d cells, %.0f x %.0f km, seed %d, %s, %s%s" % [
			world_gen.get_width(), world_gen.get_height(), real_w, real_h,
			seed_value, extent_label, shape_label, civ_note
		]
		status_label.text = summary
		shell_status_label.text = summary
		readout_label.text = "seed %d · %dx%d · %.0f x %.0f km" % [
			seed_value, world_gen.get_width(), world_gen.get_height(), real_w, real_h]
		## Everything the dialogs hold is now reflected in the world on screen.
		_clear_params_dirty()
	else:
		status_label.text = "generate failed — see console"
		readout_label.text = "generate failed"
		shell_status_label.text = "generate failed — see console"

	generate_button.disabled = false
	_generating = false


func _update_scale_bar() -> void:
	if _last_width_km <= 0.0 or map_view.size.x <= 0.0:
		scale_bar_label.text = ""
		return
	var gw := world_gen.get_width()
	if gw <= 0:
		scale_bar_label.text = "%.0f km across the map" % _last_width_km
		return
	## Cells are square in km, so one quotient describes both axes.
	scale_bar_label.text = "%.0f km across the map · %s km per cell" % [
		_last_width_km, _format_km(_last_width_km / float(gw))]


## MVP_SCOPE.md criterion 7: opens a real HTML-app .zip and renders that
## save's terrain. WorldGen.load_save (cartalith-io) reads the save's own
## stored fields directly -- no generate() call involved.
func _on_save_file_selected(path: String) -> void:
	shell_status_label.text = "loading %s..." % path.get_file()
	if not world_gen.load_save(path):
		shell_status_label.text = "load failed — see console"
		return
	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		## No civ data in a loaded save (WorldGen.load_save's own doc
		## comment) -- clear any leftover overlay from a previous generate().
		map_overlay.set_civ_data([], [], [], world_gen.get_width(), world_gen.get_height(), world_gen.get_border_inset_frac())
		territory_view.texture = null
		province_boundary_view.texture = null
		var summary := "loaded %s (%dx%d)" % [path.get_file(), world_gen.get_width(), world_gen.get_height()]
		shell_status_label.text = summary
		status_label.text = summary
		readout_label.text = "loaded %s" % path.get_file()
	else:
		shell_status_label.text = "load succeeded but render failed — see console"


## Category-1 item #1: File > Import asset pack. WorldGen.load_asset_pack/
## has_asset_pack are both real Rust functions (cartalith-assets), used
## previously only by a hardcoded debug call -- this is the first real GUI
## surface for them.
func _on_asset_pack_file_selected(path: String) -> void:
	shell_status_label.text = "importing asset pack %s..." % path.get_file()
	if world_gen.load_asset_pack(path):
		shell_status_label.text = "asset pack loaded: %s (has_asset_pack=%s)" % [path.get_file(), world_gen.has_asset_pack()]
	else:
		shell_status_label.text = "asset pack import failed — see console"
