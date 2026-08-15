extends Control
## Phase 1 MVP UI (MVP_SCOPE.md points 10-11): seed / resolution / map
## width inputs, a Generate button, and a TextureRect showing the result.
## Godot computes nothing here beyond reading input values and handing them
## to WorldGen.generate() (ARCHITECTURE.md: "Godot computes nothing beyond
## layout. Anything you could get numerically wrong belongs in Rust.").
##
## Generation runs on a background Thread (godot-shell skill: "generation
## runs off the main thread... a frozen window during it is the difference
## between a tool and a toy"). `WorldGen.generate()` is pure Rust
## computation over plain WorldState -- no scene-tree or Godot-resource
## touch (ARCHITECTURE.md: "Rust never touches the scene tree") -- so it's
## safe off-thread. `build_color_texture()` (which builds an Image/
## ImageTexture) and every scene-tree write happen back on the main thread
## via `call_deferred`.

@onready var seed_input: SpinBox = $VBox/SeedRow/SeedInput
@onready var resolution_input: SpinBox = $VBox/ResolutionRow/ResolutionInput
@onready var width_input: SpinBox = $VBox/WidthRow/WidthInput
@onready var generate_button: Button = $VBox/GenerateButton
@onready var status_label: Label = $VBox/StatusLabel
@onready var map_view: TextureRect = $MapView

var world_gen: WorldGen = WorldGen.new()
var _gen_thread: Thread
var _generating := false


func _ready() -> void:
	generate_button.pressed.connect(_on_generate_pressed)


func _on_generate_pressed() -> void:
	if _generating:
		return
	_generating = true
	generate_button.disabled = true
	status_label.text = "generating..."

	var seed_value := int(seed_input.value)
	var resolution := int(resolution_input.value)
	var width_km := width_input.value

	_gen_thread = Thread.new()
	_gen_thread.start(_generate_worker.bind(seed_value, width_km, resolution))


## Runs off the main thread. Touches only `world_gen` (plain Rust state),
## never a node -- see the class doc comment above.
func _generate_worker(seed_value: int, width_km: float, resolution: int) -> void:
	world_gen.generate(seed_value, width_km, resolution)
	_on_generate_done.call_deferred(seed_value, width_km)


## Deferred back to the main thread: joins the worker, then does the one
## Rust call that builds a Godot resource (`build_color_texture`) and every
## scene-tree write.
func _on_generate_done(seed_value: int, width_km: float) -> void:
	_gen_thread.wait_to_finish()
	_gen_thread = null

	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		status_label.text = "%dx%d, seed %d, %.0f km" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km
		]
	else:
		status_label.text = "generate failed — see console"

	generate_button.disabled = false
	_generating = false
