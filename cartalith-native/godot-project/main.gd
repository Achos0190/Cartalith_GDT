extends Control
## Phase 1 MVP UI (MVP_SCOPE.md points 10-11): seed / resolution / map
## width inputs, a Generate button, and a TextureRect showing the result.
## Godot computes nothing here beyond reading input values and handing them
## to WorldGen.generate() (ARCHITECTURE.md: "Godot computes nothing beyond
## layout. Anything you could get numerically wrong belongs in Rust.").

@onready var seed_input: SpinBox = $VBox/SeedRow/SeedInput
@onready var resolution_input: SpinBox = $VBox/ResolutionRow/ResolutionInput
@onready var width_input: SpinBox = $VBox/WidthRow/WidthInput
@onready var generate_button: Button = $VBox/GenerateButton
@onready var load_save_button: Button = $VBox/LoadSaveButton
@onready var status_label: Label = $VBox/StatusLabel
@onready var map_view: TextureRect = $MapView
@onready var load_save_dialog: FileDialog = $LoadSaveDialog

var world_gen: WorldGen = WorldGen.new()


func _ready() -> void:
	generate_button.pressed.connect(_on_generate_pressed)
	load_save_button.pressed.connect(_on_load_save_pressed)
	load_save_dialog.file_selected.connect(_on_save_file_selected)


func _on_generate_pressed() -> void:
	status_label.text = "generating..."
	var seed_value := int(seed_input.value)
	var resolution := int(resolution_input.value)
	var width_km := width_input.value

	world_gen.generate(seed_value, width_km, resolution)
	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		status_label.text = "%dx%d, seed %d, %.0f km" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km
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
