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
## Generation still runs on a background Thread (unchanged). WorldGen.
## generate()/generate_world_structure() are pure Rust computation over
## plain WorldState, safe off-thread; build_color_texture() and every
## scene-tree write happen back on the main thread via call_deferred.

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
	_build_menus()
	_select_tab("WORLD")


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
	var tip := "Per-stage parameter dialog + staleness reporting not implemented yet -- the whole pipeline still runs as one step via File > New World's Generate button."
	for stage in ["Tectonics", "Volcanism", "Erosion", "Glacial & coastal", "Hydrology",
			"Climate", "Ecology", "Settlements", "Infrastructure", "Politics"]:
		_add_inert_item(popup, stage, tip)


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
	var resolution := RESOLUTION_PRESETS[resolution_input.selected]
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

	_gen_thread = Thread.new()
	_gen_thread.start(_generate_worker.bind(seed_value, width_km, resolution, archetype))


## Runs off the main thread. Touches only world_gen (plain Rust state),
## never a node. generate() and generate_world_structure() are both full,
## equally expensive generate_terrain() calls that mutate the same
## world_gen state -- this must be the ONE call site.
func _generate_worker(seed_value: int, width_km: float, resolution: int, archetype: String) -> void:
	var ok := true
	if archetype.is_empty():
		world_gen.generate(seed_value, width_km, resolution)
	else:
		ok = world_gen.generate_world_structure(seed_value, width_km, resolution, archetype)
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

		_last_width_km = width_km
		_update_scale_bar()

		var shape_label := world_shape_input.get_item_text(world_shape_input.selected)
		var civ_note := ", %d settlements" % settlements.size() if not settlements.is_empty() else ""
		var summary := "%dx%d, seed %d, %.0f km, %s%s" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km, shape_label, civ_note
		]
		status_label.text = summary
		shell_status_label.text = summary
		readout_label.text = "seed %d · %dx%d · %.0f km" % [seed_value, world_gen.get_width(), world_gen.get_height(), width_km]
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
	scale_bar_label.text = "%.0f km across viewport" % _last_width_km


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
