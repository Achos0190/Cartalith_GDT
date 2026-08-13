extends Control
## Phase 1 MVP UI (MVP_SCOPE.md points 10-11): seed / resolution / map
## width inputs, a Generate button, and a TextureRect showing the result.
## Godot computes nothing here beyond reading input values and handing them
## to WorldGen.generate() (ARCHITECTURE.md: "Godot computes nothing beyond
## layout. Anything you could get numerically wrong belongs in Rust.").

@onready var seed_input: SpinBox = $VBox/SeedRow/SeedInput
@onready var resolution_input: SpinBox = $VBox/ResolutionRow/ResolutionInput
@onready var width_input: SpinBox = $VBox/WidthRow/WidthInput
@onready var world_shape_input: OptionButton = $VBox/WorldShapeRow/WorldShapeInput
@onready var dynamic_lithology_check: CheckBox = $VBox/DynamicLithologyCheck
@onready var volc_provinces_check: CheckBox = $VBox/VolcProvincesCheck
@onready var wind_deflection_check: CheckBox = $VBox/WindDeflectionCheck
@onready var ocean_currents_check: CheckBox = $VBox/OceanCurrentsCheck
@onready var generate_button: Button = $VBox/GenerateButton
@onready var load_save_button: Button = $VBox/LoadSaveButton
@onready var status_label: Label = $VBox/StatusLabel
@onready var map_view: TextureRect = $MapView
@onready var load_save_dialog: FileDialog = $LoadSaveDialog

var world_gen: WorldGen = WorldGen.new()

## Index into WorldShapeInput -> the archetype name WorldGen.
## generate_world_structure expects (reference HTML `ARCHETYPES`). Index 0
## ("Classic") isn't an archetype at all -- it's World-Structure disabled,
## the plain `generate()` path.
const WORLD_SHAPES := ["", "earth", "supercontinent", "archipelago", "volcanic", "rift"]


func _ready() -> void:
	generate_button.pressed.connect(_on_generate_pressed)
	load_save_button.pressed.connect(_on_load_save_pressed)
	load_save_dialog.file_selected.connect(_on_save_file_selected)


func _on_generate_pressed() -> void:
	status_label.text = "generating..."
	var seed_value := int(seed_input.value)
	var resolution := int(resolution_input.value)
	var width_km := width_input.value
	var archetype := WORLD_SHAPES[world_shape_input.selected]

	## Ported but unverified against the real JS engine in this dev
	## environment (no JS runtime here to extract golden fixtures) --
	## opt-in only. Comparing this build's output against the actual HTML
	## app with these on is exactly how that gap gets closed.
	world_gen.set_experimental_flags(
		dynamic_lithology_check.button_pressed,
		volc_provinces_check.button_pressed,
		wind_deflection_check.button_pressed,
		ocean_currents_check.button_pressed,
	)

	var ok := true
	if archetype.is_empty():
		world_gen.generate(seed_value, width_km, resolution)
	else:
		ok = world_gen.generate_world_structure(seed_value, width_km, resolution, archetype)

	if not ok:
		status_label.text = "generate failed — see console"
		return

	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		var shape_label := world_shape_input.get_item_text(world_shape_input.selected)
		status_label.text = "%dx%d, seed %d, %.0f km, %s" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km, shape_label
		]
	else:
		status_label.text = "generate failed — see console"


func _on_load_save_pressed() -> void:
	load_save_dialog.popup_centered()


## `MVP_SCOPE.md` criterion 7: opens a real HTML-app `.zip` and renders that
## save's terrain. `WorldGen.load_save` (`cartalith-io`) reads the save's
## own stored fields directly -- no `generate()` call involved, so whatever
## grid size/seed/map width the save was exported at is what's shown.
func _on_save_file_selected(path: String) -> void:
	status_label.text = "loading %s..." % path.get_file()
	if not world_gen.load_save(path):
		status_label.text = "load failed — see console"
		return
	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		status_label.text = "loaded %s (%dx%d)" % [
			path.get_file(), world_gen.get_width(), world_gen.get_height()
		]
	else:
		status_label.text = "load succeeded but render failed — see console"
