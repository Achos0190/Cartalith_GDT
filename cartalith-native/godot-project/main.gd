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

## Node lookups go through unique names (`%Name`, `unique_name_in_owner` in
## main.tscn) rather than deep `$Path/To/Node` chains -- the redesigned scene
## nests every input inside cards/scroll containers for visual grouping, and
## %-refs keep this script stable if that nesting changes again without
## touching any of these lines.
@onready var seed_input: SpinBox = %SeedInput
@onready var resolution_input: SpinBox = %ResolutionInput
@onready var width_input: SpinBox = %WidthInput
@onready var world_shape_input: OptionButton = %WorldShapeInput
@onready var dynamic_lithology_check: CheckBox = %DynamicLithologyCheck
@onready var volc_provinces_check: CheckBox = %VolcProvincesCheck
@onready var wind_deflection_check: CheckBox = %WindDeflectionCheck
@onready var ocean_currents_check: CheckBox = %OceanCurrentsCheck
@onready var generate_button: Button = %GenerateButton
@onready var load_save_button: Button = %LoadSaveButton
@onready var status_label: Label = %StatusLabel
@onready var map_view: TextureRect = %MapView
@onready var load_save_dialog: FileDialog = %LoadSaveDialog

## Responsive layout (see `_update_responsive_layout`): `Stage` is a plain
## `BoxContainer`, not a fixed `H`/`VBoxContainer` -- toggling its `vertical`
## property switches it between a side-by-side row (controls panel beside
## the map, for desktop/tablet-landscape widths) and a stacked column
## (controls above the map, for phone/portrait widths) while still letting
## the container do every size/position computation. `ControlsPanel` is the
## only node whose sizing differs between the two arrangements (a fixed-width
## column vs. a full-width band), so it's the only other node this script
## touches for layout purposes.
@onready var stage: BoxContainer = %Stage
@onready var controls_panel: PanelContainer = %ControlsPanel

var world_gen: WorldGen = WorldGen.new()
var _gen_thread: Thread
var _generating := false

## Index into WorldShapeInput -> the archetype name WorldGen.
## generate_world_structure expects (reference HTML `ARCHETYPES`). Index 0
## ("Classic") isn't an archetype at all -- it's World-Structure disabled,
## the plain `generate()` path.
const WORLD_SHAPES: Array[String] = ["", "earth", "supercontinent", "archipelago", "volcanic", "rift"]

## Below this viewport width, the fixed-width controls panel (360px, see
## main.tscn) plus a usably-sized map no longer both fit comfortably with
## touch-sized controls, so the layout stacks instead of sitting side by
## side. Chosen to fall between phone-portrait widths (~360-430px, always
## stacked) and phone-landscape/tablet/desktop widths (~700px+, always
## side-by-side) -- not verified against a real device, same GPU/touch
## carve-out the godot-shell skill calls out for anything screen-visual.
const RESPONSIVE_BREAKPOINT_WIDTH := 700.0


func _ready() -> void:
	generate_button.pressed.connect(_on_generate_pressed)
	load_save_button.pressed.connect(_on_load_save_pressed)
	load_save_dialog.file_selected.connect(_on_save_file_selected)
	get_viewport().size_changed.connect(_update_responsive_layout)
	_update_responsive_layout()


## Desktop window resize and phone orientation change both fire
## `Viewport.size_changed` -- one hook covers both real targets. Only
## property toggles here, no pixel math: `Stage.vertical` picks the axis,
## `ControlsPanel`'s horizontal size flag + minimum width pick fixed-column
## vs. fill-width, and the containers do the rest.
func _update_responsive_layout() -> void:
	var viewport_width := get_viewport().get_visible_rect().size.x
	var narrow := viewport_width < RESPONSIVE_BREAKPOINT_WIDTH
	stage.vertical = narrow
	if narrow:
		controls_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		controls_panel.custom_minimum_size.x = 0
	else:
		controls_panel.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
		controls_panel.custom_minimum_size.x = 360


func _on_generate_pressed() -> void:
	if _generating:
		return
	_generating = true
	generate_button.disabled = true
	status_label.text = "generating..."

	var seed_value := int(seed_input.value)
	var resolution := int(resolution_input.value)
	var width_km := width_input.value
	var archetype := WORLD_SHAPES[world_shape_input.selected]

	## All four golden-verified against the real JS engine
	## (cartalith-native/docs/CHANGELOG.md). Still exposed as toggles --
	## default checked state matches each one's real JS default.
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
		_generating = false
		generate_button.disabled = false
		return

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
		var shape_label := world_shape_input.get_item_text(world_shape_input.selected)
		status_label.text = "%dx%d, seed %d, %.0f km, %s" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km, shape_label
		]
	else:
		status_label.text = "generate failed — see console"

	generate_button.disabled = false
	_generating = false


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
