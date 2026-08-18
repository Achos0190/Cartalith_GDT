extends Workspace
class_name WorldWorkspace

## WORLD domain (`DCC_SHELL_SPEC.md` §5): a two-button switch between the
## ten-stage Generation Pipeline (§5.1) and the Sculpt panel (§5.2).
##
## Every stage row reads and writes through `bridge.param_keys()` /
## `param_info()` / `param_get()` / `param_set()` -- the same live table
## `main.gd`'s old Generate menu built its per-stage dialogs from
## (`cartalith-godot/src/params.rs`, 58 parameters). No range, step, label or
## default is copied into this file; only which stage a group/key belongs to,
## which rows are L5 Advanced, and the prose -- exactly the division main.gd's
## own GEN_STAGES comment already argued for.
##
## The STAGES table below re-derives that stage/group mapping for the spec's
## own ten stage names and dependency order, which differ from main.gd's old
## ten-entry Generate menu: that menu mixed in civ stages (Settlements,
## Infrastructure, Politics) the spec's Generation Pipeline does not cover at
## all -- those live in the CIVIL/INFRA domains, not WORLD, under this spec.
##
## The Sculpt half cannot be wired at all until `cartalith-godot` binds
## `SculptStamp` -- see `STRANDED_TOOLS.md` rows 4-8.

## L5 -- the same list main.gd's ADVANCED_KEYS used, for the same reason: a
## parameter is Advanced if the reference itself buried it (its own
## `<details class="adv">` fold), or if this port surfaces it as a superset
## the reference never exposed at all.
const ADVANCED_KEYS: Array[String] = [
	"tect.flexure", "tect.hetero", "tect.resist", "tect.dynamic_lithology", "tect.lloyd",
	"climate.current_k", "climate.terrain_wind_deflection", "climate.ocean_hum", "climate.bulk_evap",
]

## Section headings for a stage's `groups`, matching the reference HTML's own
## panel headings (main.gd's GROUP_TITLES).
const GROUP_TITLES := {
	"planet": "Planet",
	"world_structure": "World structure",
	"tectonics": "Plates & uplift",
	"volcanism": "Volcanism & impacts",
	"erosion": "Stream-power carve",
	"climate": "Climate & temperature",
	"weather": "Weather · rainfall sim",
}

## A stage whose real content is a handful of loose `keys` (pulled out of the
## params.rs "world" group, which spans three different stages here) gets a
## nicer section title than its own stage name repeated.
const KEYS_SECTION_TITLES := {
	"Extent & scale": "Scale & calibration",
	"Hydrology": "River network",
}

## §5.1's own ten-row table, in its own dependency order. `groups` pulls every
## params.rs row whose `group` field matches; `keys` pulls single keys out of
## the "world" group. `use_gpu` (also in "world") appears in neither list --
## it is the NOT-A-GENERATION-STAGE block's GPU row, per Preferences ▸
## Performance, not a pipeline parameter.
const STAGES: Array = [
	{"name": "Planet", "needs": "—",
	 "produces": "gravity, rotation, tilt, geoid, tides → 02 Extent & scale, 08 Climate",
	 "groups": ["planet"], "keys": [],
	 "gap": "Geoid sea level and tides (moon mass, distance, k₂) are default-off reference sub-systems with no cartalith-engine equivalent yet."},
	{"name": "Extent & scale", "needs": "01 Planet",
	 "produces": "land/sea split, all distances → every later stage",
	 "groups": [], "keys": ["world", "sea_level", "peak_m"],
	 "gap": "Working resolution and the extent's grid-height effect are creation-time call arguments, not stored parameters (\"GRID HEIGHT IS A CALL ARGUMENT, NOT A PARAMETER\" -- main.gd's own rule): set in File ▸ New world, not here."},
	{"name": "World structure", "needs": "01 Planet",
	 "produces": "continentality field → 04 Tectonics",
	 "groups": ["world_structure"], "keys": [],
	 "gap": "The archetype NAME (Earth-like, Supercontinent, Archipelago, Volcanic, Rift) picks which generation call runs and is a creation-time argument chosen in File ▸ New world. The five dials below are what that choice sets, and stay editable here afterward."},
	{"name": "Tectonics", "needs": "01 Planet, 03 World structure",
	 "produces": "elevation, plate_id, boundary_type, resistance → 05 Volcanism, 06 Erosion, 10 Resources & soils",
	 "groups": ["tectonics"], "keys": [], "gap": ""},
	{"name": "Volcanism & impacts", "needs": "04 Tectonics",
	 "produces": "cones, provinces, craters → 06 Erosion",
	 "groups": ["volcanism"], "keys": [], "gap": ""},
	{"name": "Erosion", "needs": "04 Tectonics, 08 Climate",
	 "produces": "final surface → 07 Hydrology, 10 Resources & soils",
	 "groups": ["erosion"], "keys": [],
	 "gap": "Only the stream-power carve is ported. Droplet hydraulic, Hillslope diffuse, Velocity (momentum), Glacial and Coastal are each a separate manual pass in the reference with no cartalith-engine equivalent -- the groups below for those five are honest placeholders, not missing controls."},
	{"name": "Hydrology", "needs": "06 Erosion",
	 "produces": "rivers, lakes, drainage, flow accumulation → 08 Climate, 09 Ecology & biomes",
	 "groups": [], "keys": ["carve_rivers", "river_density"],
	 "gap": "Min stream order and lakes-as-water are reference render filters, not generation parameters -- Cartography's map-mode work, not this stage."},
	{"name": "Climate", "needs": "01 Planet, 02 Extent & scale, 06 Erosion",
	 "produces": "temperature, rainfall, wind, currents → 09 Ecology & biomes, 10 Resources & soils",
	 "groups": ["climate", "weather"], "keys": [],
	 "gap": "Seasons and Köppen-Geiger classification are not ported."},
	{"name": "Ecology & biomes", "needs": "07 Hydrology, 08 Climate",
	 "produces": "biome classification, ecotones → 10 Resources & soils",
	 "groups": [], "keys": [],
	 "gap": "Not parameterised. Biome classification runs off the finished elevation/temperature/rainfall fields with no dials of its own in cartalith-engine."},
	{"name": "Resources & soils", "needs": "04 Tectonics, 08 Climate, 09 Ecology & biomes",
	 "produces": "soil depth, ore, fertility → nothing downstream in this pipeline",
	 "groups": [], "keys": [],
	 "gap": "Not parameterised. No dials exist in cartalith-engine for soil, ore or fertility generation."},
]

const EROSION_STAGE_INDEX := 5 ## Zero-based -- STAGES[5] is "Erosion".

var _pipeline_body: VBoxContainer
var _sculpt_body: VBoxContainer
var _stage_state_labels: Array = []  ## stage index -> the trailing state Label.

## There is no staleness state, verified live against the reference
## (Playwright, 2026-08-19, on direct owner instruction) rather than assumed
## from the DCC mockup's prose: `tparam()` wires every generation slider so
## `input` (dragging) only updates its own label, and `change` (release)
## applies the value AND calls `generate()` immediately --
## `el.addEventListener('change',()=>{ apply(+el.value); withBusy
## ('generating…',generate); })`, verbatim. A DOM sweep for anything matching
## `/run stage|run \d+.*→/i` found zero buttons anywhere in the reference.
## §5.1's "stale from 04 Tectonics — 6 downstream stages will re-run" and its
## "Run stage N / Run N → 10" controls describe a partial-recompute capability
## that exists in neither the reference app nor this engine (`generate_terrain`
## is one-shot, confirmed by reading `generate()`'s own body: it runs all ten
## stages unconditionally, every call). Building disabled buttons for that
## capability was clutter implying it will one day exist; it will not, absent
## a real engine redesign, so every row below regenerates the whole world on
## release instead -- the same one call site `_on_generate_pressed` already
## used, now fired automatically rather than waiting for a button.

func _build() -> void:
	var switch_row := HBoxContainer.new()
	switch_row.add_theme_constant_override("separation", 0)
	var switch_pad := MarginContainer.new()
	switch_pad.add_theme_constant_override("margin_left", 12)
	switch_pad.add_theme_constant_override("margin_right", 12)
	switch_pad.add_theme_constant_override("margin_top", 8)
	switch_pad.add_theme_constant_override("margin_bottom", 4)
	switch_pad.add_child(switch_row)
	add_child(switch_pad)

	var mode_group := ButtonGroup.new()
	var pipeline_btn := _switch_button("Generation pipeline", true, mode_group)
	var sculpt_btn := _switch_button("Sculpt", false, mode_group)
	switch_row.add_child(pipeline_btn)
	switch_row.add_child(sculpt_btn)

	_pipeline_body = VBoxContainer.new()
	_pipeline_body.add_theme_constant_override("separation", 0)
	_pipeline_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_pipeline_body)
	_build_pipeline(_pipeline_body)

	_sculpt_body = VBoxContainer.new()
	_sculpt_body.add_theme_constant_override("separation", 0)
	_sculpt_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sculpt_body.visible = false
	add_child(_sculpt_body)
	_build_sculpt(_sculpt_body)

	pipeline_btn.pressed.connect(_select_mode.bind("pipeline"))
	sculpt_btn.pressed.connect(_select_mode.bind("sculpt"))

	bridge.generation_started.connect(_refresh_stage_states)
	bridge.generation_finished.connect(_on_generation_finished)
	bridge.world_loaded.connect(_on_world_loaded)
	_refresh_stage_states()

func _select_mode(mode: String) -> void:
	_pipeline_body.visible = mode == "pipeline"
	_sculpt_body.visible = mode == "sculpt"

func _on_generation_finished(_ok: bool) -> void:
	_refresh_stage_states()

func _on_world_loaded() -> void:
	_refresh_stage_states()

## The mockup draws the switch as **tabs**, not as a segmented control: mono
## caps, the active half carrying an accent top rule and a slightly lifted
## ground, both halves sitting on the dock's own hairline. A filled amber pill
## was the first attempt and read as a call-to-action button, which is the one
## thing this control is not -- it is a view selector.
##
## Deliberately not `flat = true`: on a `toggle_mode` button that suppresses the
## "pressed" stylebox entirely, so the active tab drew with no accent at all.
func _switch_button(text: String, active: bool, group: ButtonGroup) -> Button:
	var b := Button.new()
	b.text = text.to_upper()
	b.toggle_mode = true
	b.button_pressed = active
	b.button_group = group
	b.focus_mode = Control.FOCUS_NONE
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.custom_minimum_size.y = 30
	b.add_theme_font_size_override("font_size", DccTheme.FS_HEADER)
	b.add_theme_font_override("font", DccTheme.mono(2, true))
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))

	var rest := StyleBoxFlat.new()
	rest.bg_color = DccTheme.c("panel_alt")
	rest.border_width_bottom = 1
	rest.border_color = DccTheme.c("line")
	var on := StyleBoxFlat.new()
	on.bg_color = DccTheme.c("accent_wash")
	on.border_width_top = 1
	on.border_color = DccTheme.c("accent")
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("pressed", on)
	b.add_theme_stylebox_override("hover", rest)
	return b

# -- §5.1 Generation Pipeline --------------------------------------------------

func _build_pipeline(parent: Control) -> void:
	for i in STAGES.size():
		_build_stage(parent, i)

	var not_stage := DccWidgets.section(parent, "Not a generation stage")
	DccWidgets.note(not_stage,
		"GPU acceleration and multi-GPU → Preferences ▸ Performance. Render quality, lighting, 3D viewport → Preferences ▸ Graphics. Tiled LOD, atlas cache, chunk debug → Preferences ▸ Tiles & LOD. Terrain appearance, style presets, ramps → Cartography. Settlements, routes, politics → Civilization.")

	var foot := DccWidgets.section(parent, "Finalize")
	var bake := DccWidgets.action(foot, "Finalize · LOD 0–3 · bake & freeze", func(): pass)
	bake.disabled = true
	bake.tooltip_text = "No bake pipeline exists: nothing writes a frozen LOD tile atlas yet (cartalith-spatial exists standalone, unintegrated -- LOD_TILING_BASE_SCOPE.md). Finalizing would lock stages 01-10 and Sculpt; there is nothing here to lock against yet."

func _build_stage(parent: Control, index: int) -> void:
	var stage: Dictionary = STAGES[index]
	var number := "%02d" % (index + 1)
	var head := DccWidgets.stage_category(parent, number, String(stage["name"]), categories, index == 0)
	var body: VBoxContainer = head["body"]
	_stage_state_labels.append(head["state_label"])

	## The mockup indents a stage's `needs`/`produces` under its title rather
	## than running them to the dock's own edge, which is what `note()` on a
	## bare body does.
	var meta := VBoxContainer.new()
	meta.add_theme_constant_override("separation", 1)
	var meta_pad := MarginContainer.new()
	meta_pad.add_theme_constant_override("margin_left", 14)
	meta_pad.add_theme_constant_override("margin_right", 12)
	meta_pad.add_theme_constant_override("margin_bottom", 2)
	meta_pad.add_child(meta)
	body.add_child(meta_pad)

	DccWidgets.note(meta, "needs — %s" % String(stage["needs"]))
	DccWidgets.note(meta, "produces — %s" % String(stage["produces"]))
	if not String(stage["gap"]).is_empty():
		DccWidgets.note(meta, String(stage["gap"]))

	if index == EROSION_STAGE_INDEX:
		_build_erosion_passes(body, index)
		return

	for group_name: String in (stage["groups"] as Array):
		_build_group_section(body, group_name, index)

	if not (stage["keys"] as Array).is_empty():
		var title: String = KEYS_SECTION_TITLES.get(String(stage["name"]), String(stage["name"]))
		var sec := DccWidgets.section(body, title)
		var advanced_keys: Array = []
		for key: String in (stage["keys"] as Array):
			if ADVANCED_KEYS.has(key):
				advanced_keys.append(key)
			else:
				_build_param_row(sec, key, index)
		if not advanced_keys.is_empty():
			var adv := DccWidgets.advanced(sec)
			for key in advanced_keys:
				_build_param_row(adv, key, index)

## One params.rs `group`, in the reference's own within-panel order (the
## engine builds PARAMS in that order, and Dictionary iteration in GDScript
## preserves insertion order, so no extra sort is needed here).
func _build_group_section(parent: Control, group_name: String, stage_index: int) -> void:
	var sec := DccWidgets.section(parent, String(GROUP_TITLES.get(group_name, group_name.capitalize())))
	var advanced_keys: Array = []
	for key in bridge.param_keys():
		var info := bridge.param_info(key)
		if String(info.get("group", "")) != group_name:
			continue
		if ADVANCED_KEYS.has(key):
			advanced_keys.append(key)
		else:
			_build_param_row(sec, key, stage_index)
	if not advanced_keys.is_empty():
		var adv := DccWidgets.advanced(sec)
		for key in advanced_keys:
			_build_param_row(adv, key, stage_index)

## Stage 06's own table: "droplet, hillslope diffuse, stream-power, velocity,
## glacial, coastal -- each its own group with its own run button". Only
## stream-power is real; the other five are L4 groups too, honestly empty.
func _build_erosion_passes(body: VBoxContainer, stage_index: int) -> void:
	var real := DccWidgets.group(body, "Stream-power carve", true)
	for key in bridge.param_keys():
		var info := bridge.param_info(key)
		if String(info.get("group", "")) == "erosion":
			_build_param_row(real, key, stage_index)

	for pass_name in ["Droplet hydraulic", "Hillslope diffuse", "Velocity (momentum)", "Glacial", "Coastal"]:
		var grp := DccWidgets.group(body, pass_name, false)
		DccWidgets.note(grp, "Not ported -- a separate manual pass in the reference with no cartalith-engine equivalent.")
		var btn := DccWidgets.action(grp, "Run %s" % pass_name, func(): pass)
		btn.disabled = true
		btn.tooltip_text = "No cartalith-engine implementation exists for this pass."

## One row for one parameter. `bridge.param_info(key)`'s `type` field decides
## the control (`toggle` for bool, `slider` for int/float); nothing about the
## range, step, label or unit is guessed here.
func _build_param_row(parent: Control, key: String, stage_index: int) -> void:
	var info := bridge.param_info(key)
	if info.is_empty():
		return
	var label := String(info.get("label", key))
	var unit := String(info.get("unit", ""))
	var ref_ctrl := String(info.get("reference_control", ""))
	var hint := ("Reference control #%s." % ref_ctrl) if not ref_ctrl.is_empty() else \
		"Not exposed by the reference app — surfaced here as a superset, at the engine's own default."
	var kind := String(info.get("type", "float"))

	if kind == "bool":
		## A checkbox toggle is atomic -- there is no "dragging" phase to defer
		## past -- so it regenerates immediately, matching the reference's own
		## `<input type=checkbox>` `change` handlers (fired on click, not on a
		## release distinct from a press).
		DccWidgets.toggle(parent, label, bool(bridge.param_get(key)),
			_on_bool_row_changed.bind(key), hint)
		return

	var is_int := kind == "int"
	## `tparam()`'s exact split: `input` (every drag tick) only updates the
	## value; `change` (release) applies it and regenerates. `DccWidgets.slider`
	## already gives the continuous half via `on_change` -- `on_release` is new,
	## wired to `HSlider.drag_ended`, which is Godot's one-shot release signal.
	DccWidgets.slider(parent, label, float(info.get("min", 0.0)), float(info.get("max", 1.0)),
		float(info.get("step", 0.01)), float(bridge.param_get(key)), unit,
		_on_float_row_input.bind(key, is_int), hint,
		_on_float_row_released.bind(key, is_int))

func _on_bool_row_changed(v: bool, key: String) -> void:
	bridge.param_set(key, v)
	_regenerate_live()

## Writes the value continuously (cheap: `param_set` is an in-memory Rust
## write, no recompute) but does not regenerate -- matches `tparam()`'s
## `input` handler updating only the label.
func _on_float_row_input(v: float, key: String, is_int: bool) -> void:
	bridge.param_set(key, (int(round(v)) if is_int else v))

func _on_float_row_released(key: String, is_int: bool) -> void:
	_regenerate_live()

## The one thing every generation control now triggers on release, exactly
## like the reference's own `withBusy('generating…', generate)`: the whole
## world, from stage 01, with whatever the dock's sliders currently say. No
## staleness to track -- by the time this returns, the map matches the dials
## again, same as it always did in the app being ported.
func _regenerate_live() -> void:
	if app == null or app.new_world_dialog == null or bridge.generating:
		return
	bridge.generate(app.new_world_dialog.request())

## §5.1's state column, honestly reduced to what is actually true now that
## there is no partial-recompute concept: every stage is either not-yet-built,
## mid-regenerate, or resolved together, because one `generate()` call resolves
## all ten at once. Nothing here claims per-stage granularity that isn't real.
func _refresh_stage_states() -> void:
	for i in _stage_state_labels.size():
		var lbl: Label = _stage_state_labels[i]
		if not bridge.has_world:
			lbl.text = "no world"
			lbl.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		elif bridge.generating:
			lbl.text = "%s generating" % DccIcons.SYMBOLS["on"]
			lbl.add_theme_color_override("font_color", DccTheme.c("accent"))
		else:
			lbl.text = "%s resolved" % DccIcons.SYMBOLS["tick"]
			lbl.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	_push_dock_readout()

## §3's rail-foot stage counter ("04 / 10"), repurposed as the collapsed left
## dock's own primary readout (§6: a collapsed dock keeps its one essential
## number, never blanks).
func _push_dock_readout() -> void:
	if app == null:
		return
	if not bridge.has_world:
		app.set_dock_readout("left", "no world")
	elif bridge.generating:
		app.set_dock_readout("left", "generating…")
	else:
		app.set_dock_readout("left", "10 / 10 resolved")

## The dock's own primary action, mirroring the tool options bar's "Generate
## world" (`app.gd`'s `_run_pipeline`, `#genBtn` in the reference) -- the same
## one call site (`EngineBridge.generate`) every live-edit row above also uses.
func _on_generate_pressed() -> void:
	_regenerate_live()

# -- §5.2 Sculpt ----------------------------------------------------------------

func _build_sculpt(parent: Control) -> void:
	var body := DccWidgets.section(parent, "Sculpt")
	DccWidgets.note(body,
		"cartalith-terrain/src/sculpt.rs implements the full registry §5.2 specifies -- all thirteen geological features (Mountains, Hills, Ridge, Plateau, Cliff/Escarpment, Canyon, Valley, River, Lake, Basin, Coastline, Volcano, Freehand), eight presets, eight brush-shape falloffs and eight Freehand sub-modes -- but cartalith-godot exports no sculpt, stamp or commit method. Nothing here can be wired until that binding lands (STRANDED_TOOLS.md rows 4-8, UNIFIED_TOOL_PLAN.md milestone F).")
	DccWidgets.note(body,
		"One spec-vs-engine detail worth recording before that binding lands: §5.2's commit prose says it \"re-runs erosion, hydrology and climate once\", but commit_sculpt_pass deliberately marks tiles stale instead -- the eager form measured about 7 s per stroke at 2048².")
