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
@onready var status_label: Label = $VBox/StatusLabel
@onready var map_view: TextureRect = $MapView

var world_gen: WorldGen = WorldGen.new()


func _ready() -> void:
	generate_button.pressed.connect(_on_generate_pressed)


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
