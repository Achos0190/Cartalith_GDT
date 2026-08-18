extends AcceptDialog
class_name NewWorldDialog

## File ▸ New world… (`DCC_SHELL_SPEC.md` §2.1): name, seed, extent
## (region/world) and working resolution.
##
## The dimension logic here is ported from `main.gd`'s `_build_world_setup`
## rather than rewritten -- it is hard-won: cells are square in km, grid
## height is a derived readout except when hand-typed, and world mode pins
## the aspect to 2:1 because X wraps 360° of longitude over the grid width
## (`GENERATION_PARAMETERS.md` "Map dimensions and aspect ratio").
##
## Sea level, the extent flag and the four experimental toggles below are all
## real `params.rs` entries (`bridge.param_get`/`param_set`), not local state
## -- so this dialog and World ▸ Generation Pipeline's own stage rows (02
## Extent & scale, 04 Tectonics, 05 Volcanism, 08 Climate) are two views of
## the same engine state rather than two copies that can disagree. Only what
## has no engine-side storage (seed, grid dimensions, archetype name, village
## seeding) is this dialog's own.

var bridge: EngineBridge

# -- Ported constants (main.gd, verbatim) --------------------------------------

const SIZE_PRESETS: Array = [
	{"label": "Local · 200 km", "km": 200.0},
	{"label": "Province · 800 km", "km": 800.0},
	{"label": "Region · 2 000 km", "km": 2000.0},
	{"label": "Subcontinent · 5 000 km", "km": 5000.0},
	{"label": "Continent · 12 000 km", "km": 12000.0},
	{"label": "Planet · 40 075 km (Earth's equator)", "km": 40075.0},
]
const SIZE_CUSTOM_INDEX := 6 ## SIZE_PRESETS.size() -- one past the last preset.

const ASPECT_PRESETS: Array = [
	{"label": "2:1 · equirectangular", "ratio": 2.0},
	{"label": "16:9 · widescreen", "ratio": 16.0 / 9.0},
	{"label": "1.5625:1 · reference region frame", "ratio": 1.5625},
	{"label": "4:3 · classic landscape", "ratio": 4.0 / 3.0},
	{"label": "1:1 · square", "ratio": 1.0},
	{"label": "3:4 · portrait", "ratio": 0.75},
	{"label": "9:16 · tall portrait", "ratio": 9.0 / 16.0},
]
const ASPECT_WORLD_INDEX := 0   ## 2:1 -- the shape world mode is pinned to.
const ASPECT_REGION_INDEX := 2  ## The reference's own region frame; gh comes from the engine.
const ASPECT_CUSTOM_INDEX := 7
const ASPECT_DEFAULT_INDEX := ASPECT_REGION_INDEX

const RESOLUTION_PRESETS: Array[int] = [512, 1024, 2048, 4096, 8192]
const RESOLUTION_LABELS: Array[String] = ["512", "1K", "2K", "4K", "8K"]
const RESOLUTION_DEFAULT_INDEX := 2 ## 2K, the reference's own default.
const RESOLUTION_CUSTOM_INDEX := 5

const GRID_MIN := 4 ## generate_sized() clamps each dimension to >= 4; match it rather than let the engine clamp behind the dialog's back.
const GRID_MAX := 8192
const DEGENERATE_ASPECT := 16.0 ## Past this, the coarse weather grid loses almost all resolution on the short axis.

const EXTENT_NOTE_REGION := "Region — a framed area of a world. The map's north and south edge latitudes are set in the Climate stage. X does not wrap. Any aspect ratio is physically fine here."
const EXTENT_NOTE_WORLD := "Whole world — a seamless equirectangular sheet: X wraps a full 360° of longitude and Y spans 180° of latitude, pole to pole. That fixes the shape at 2:1; any other ratio would stretch the graticule against the terrain, so the aspect is pinned and the grid height comes from the engine's own reference_grid_height(gw, true)."

## `earth`/`supercontinent`/… (`bridge.archetypes()`) with the reference's own
## friendlier names -- a display concern, so it lives here rather than in Rust.
const ARCHETYPE_LABELS := {
	"earth": "Earth-like", "supercontinent": "Supercontinent",
	"archipelago": "Archipelago", "volcanic": "Volcanic", "rift": "Rift",
}

# -- Controls -------------------------------------------------------------------

var seed_input: SpinBox
var extent_input: OptionButton
var extent_note_label: Label
var size_preset_input: OptionButton
var width_input: SpinBox
var resolution_input: OptionButton
var grid_w_input: SpinBox
var aspect_input: OptionButton
var grid_h_input: SpinBox
var archetype_input: OptionButton
var villages_check: CheckBox
var dimension_warning_label: Label
var _derived_labels: Dictionary = {} ## "Grid"/"Extent"/"Cell size"/"Aspect" -> value Label

var _archetype_names: PackedStringArray = []
var _archetype := "" ## Empty = Classic (World Structure disabled).
var _villages := false
var _dim_syncing := false
var _auto_generate := true ## Whether Create also calls bridge.generate() -- see set_auto_generate().

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "New world"
	size = Vector2i(620, 780)
	wrap_controls = false ## See _build_stage_dialog's own comment in main.gd: an autowrap dialog grows to its full content height and can run off a 1080p screen.
	get_ok_button().text = "Create"
	confirmed.connect(_on_create)

	var margin := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 10)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 4)
	margin.add_child(root)

	DccWidgets.note(root,
		"Creation-time only. Extent, resolution and archetype reallocate every field in the pipeline, so changing them later means a fresh Create, not a live edit — the reference itself refuses to make width mid-project editable for the same reason.")

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	_archetype_names = bridge.archetypes()
	_build(body)
	_update_extent_state()

func _build(body: VBoxContainer) -> void:
	var seed_sec := DccWidgets.section(body, "Seed")
	seed_input = DccWidgets.number(seed_sec, "Seed", 0, 2147483647, 1, randi() % 1000000,
		func(_v: float): pass,
		"Integer seed. The same seed and settings reproduce the same world.")

	var extent_sec := DccWidgets.section(body, "Extent")
	extent_input = DccWidgets.choice(extent_sec, "Extent", ["Region", "Whole world"],
		1 if bool(bridge.param_get("world")) else 0, _on_extent_selected,
		"Reference control #extentSeg. Region = a framed area with user-set latitudes; Whole world = a seamless equirectangular sheet with toroidal X wrap.")
	extent_note_label = DccWidgets.note(extent_sec, "")

	var size_sec := DccWidgets.section(body, "Map width & resolution")
	var size_labels: Array = []
	for preset: Dictionary in SIZE_PRESETS:
		size_labels.append(String(preset["label"]))
	size_labels.append("Custom")
	size_preset_input = DccWidgets.choice(size_sec, "Map width", size_labels, 1,
		_on_size_preset_selected,
		"Real-world width of the map. Creation-time only, same as the reference: changing it silently rescales every derived distance, grade, route length and settlement spacing.")
	width_input = DccWidgets.number(size_sec, "Width (km)", 1.0, 100000.0, 1.0, 800.0,
		func(_v: float): _refresh_dimensions())

	var res_labels: Array = []
	for l in RESOLUTION_LABELS:
		res_labels.append(l)
	res_labels.append("Custom")
	resolution_input = DccWidgets.choice(size_sec, "Resolution", res_labels, RESOLUTION_DEFAULT_INDEX,
		_on_resolution_selected,
		"The reference's own 512/1K/2K/4K/8K segment. Sets the grid WIDTH only; grid height follows below.")
	grid_w_input = DccWidgets.number(size_sec, "Grid columns", GRID_MIN, GRID_MAX, 1,
		RESOLUTION_PRESETS[RESOLUTION_DEFAULT_INDEX], func(_v: float): _refresh_dimensions())

	var aspect_labels: Array = []
	for preset: Dictionary in ASPECT_PRESETS:
		aspect_labels.append(String(preset["label"]))
	aspect_labels.append("Custom")
	aspect_input = DccWidgets.choice(size_sec, "Aspect", aspect_labels, ASPECT_DEFAULT_INDEX,
		func(_i: int): _refresh_dimensions(),
		"The frame's width:height. The reference has no aspect control -- it hardcodes 2:1 in world mode and 1.5625:1 otherwise; both are here by name.")
	grid_h_input = DccWidgets.number(size_sec, "Grid rows", GRID_MIN, GRID_MAX, 1, 1311,
		_on_grid_h_changed,
		"A call argument to generate_sized(), not a stored parameter: changing it reallocates every field in the pipeline.")

	_build_derived_panel(size_sec)
	dimension_warning_label = DccWidgets.note(size_sec, "")
	dimension_warning_label.add_theme_color_override("font_color", DccTheme.c("stale"))

	var struct_sec := DccWidgets.section(body, "World structure")
	var archetype_labels: Array = ["Classic"]
	for name in _archetype_names:
		archetype_labels.append(String(ARCHETYPE_LABELS.get(String(name), String(name).capitalize())))
	archetype_input = DccWidgets.choice(struct_sec, "Archetype", archetype_labels, 0, _on_archetype_selected,
		"Reference ARCHETYPES. Choosing one applies its continentality/fragmentation/tectonic-energy/ocean-depth/hotspot-density preset immediately and routes Create through generate_world_structure_sized instead of the plain path.")
	DccWidgets.note(struct_sec,
		"The five dials that preset sets -- continentality, fragmentation, tectonic energy, ocean depth, hotspot density -- live in World ▸ Generation Pipeline ▸ 03 World structure, editable there after Create.")

	var gen_sec := DccWidgets.section(body, "Generation")
	villages_check = DccWidgets.toggle(gen_sec, "Village seeding (additive hamlets)", false,
		func(v: bool): _villages = v,
		"Reference civVillagesChk, default off. Seeds an extra tier of hamlets after the main settlement pass.")
	DccWidgets.note(gen_sec,
		"Sea level, dynamic lithology, volcanic provinces, terrain wind deflection and ocean currents are live engine parameters, not dialog state -- edit them in World ▸ Generation Pipeline (stages 02, 04, 05, 08) before or after Create; Create reads whatever they currently hold.")

func _build_derived_panel(parent: Control) -> void:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("sunken"), 2))
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 16)
	grid.add_theme_constant_override("v_separation", 4)
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 10)
	pad.add_child(grid)
	panel.add_child(pad)
	for key in ["Grid", "Extent", "Cell size", "Aspect"]:
		grid.add_child(DccTheme.label(key, "text_faint", DccTheme.FS_TINY))
		var v := DccTheme.label("—", "accent", DccTheme.FS_SMALL)
		grid.add_child(v)
		_derived_labels[key] = v
	parent.add_child(panel)

# -- Extent / dimension logic (main.gd, ported) --------------------------------

func _on_extent_selected(index: int) -> void:
	bridge.param_set("world", index == 1)
	_update_extent_state()

## World mode pins the aspect to 2:1 and says why on screen -- the control is
## disabled rather than removed, and the note above it carries the physical
## reason, so a greyed control never reads as a bug.
func _update_extent_state() -> void:
	var world := extent_input.selected == 1
	if world:
		aspect_input.selected = ASPECT_WORLD_INDEX
	if bridge.sized_api:
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
## reference ratios are asked of the engine (`bridge.reference_grid_height`,
## the reference app's own gridH) rather than recomputed from ASPECT_PRESETS,
## so the two constants live in exactly one place.
func _derived_grid_h(gw: int) -> int:
	if not bridge.sized_api:
		return gw
	if extent_input.selected == 1:
		return bridge.reference_grid_height(gw, true)
	if aspect_input.selected == ASPECT_REGION_INDEX:
		return bridge.reference_grid_height(gw, false)
	var ratio := float(ASPECT_PRESETS[aspect_input.selected]["ratio"])
	return maxi(GRID_MIN, int(round(gw / ratio)))

## Single re-entrant-safe sync: preset buttons follow their free entry, the
## row count follows the aspect selection, and the readout follows both.
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

# -- World structure --------------------------------------------------------

func _on_archetype_selected(index: int) -> void:
	if index <= 0:
		_archetype = ""
		return
	var name := String(_archetype_names[index - 1])
	_archetype = name
	bridge.apply_archetype(name)

# -- Create ---------------------------------------------------------------------

## Whether pressing Create also starts a generate (default true, matching
## File ▸ New world's own reference behaviour). A future caller wanting to
## only stage values without generating yet can flip this before opening.
func set_auto_generate(v: bool) -> void:
	_auto_generate = v

func _on_create() -> void:
	if _auto_generate:
		bridge.generate(request())

## The keys `EngineBridge.generate()` reads. Sea level and the four
## experimental flags are read live off `bridge` rather than cached locally --
## see this file's own header comment on why that keeps this dialog and the
## Generation Pipeline's stage rows from ever disagreeing.
func request() -> Dictionary:
	return {
		"seed": int(seed_input.value),
		"width_km": width_input.value,
		"grid_w": int(grid_w_input.value),
		"grid_h": int(grid_h_input.value),
		"archetype": _archetype,
		"villages": _villages,
		"sea_level": bridge.param_get("sea_level"),
		"dynamic_lithology": bridge.param_get("tect.dynamic_lithology"),
		"volcanic_provinces": bridge.param_get("volc.provinces"),
		"wind_deflection": bridge.param_get("climate.terrain_wind_deflection"),
		"ocean_currents": bridge.param_get("climate.currents"),
	}
