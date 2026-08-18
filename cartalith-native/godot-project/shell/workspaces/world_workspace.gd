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

## GUI-only bookkeeping, not engine state: which stage's row the user last
## edited. The engine is one-shot (`generate_terrain` runs the whole pipeline
## or none of it) so there is no per-stage recompute for a "stale" mark to be
## relative to -- but "you changed stage n, so n and everything after it in
## dependency order no longer matches the last generate" is true regardless,
## and is exactly what §5.1's stale-propagation rule describes. This tracks
## that honestly instead of either faking real per-stage state or dropping
## the state column main.gd's own comment argued didn't exist.
var _last_edited_stage := -1

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

	bridge.generation_finished.connect(_on_generation_finished)
	bridge.world_loaded.connect(_on_world_loaded)
	_refresh_stage_states()

func _select_mode(mode: String) -> void:
	_pipeline_body.visible = mode == "pipeline"
	_sculpt_body.visible = mode == "sculpt"

func _on_generation_finished(_ok: bool) -> void:
	_last_edited_stage = -1
	_refresh_stage_states()

func _on_world_loaded() -> void:
	_last_edited_stage = -1
	_refresh_stage_states()

## Deliberately NOT `flat = true`: `DccWidgets.action()`'s own primary button
## (the "Generate world" bar below) proves solid custom styleboxes render
## correctly without it, and `flat` on a toggle_mode button suppressed the
## "pressed" stylebox entirely in practice -- the active half of the switch
## drew with no accent background at all. Explicit normal/pressed/hover
## overrides carry the same "flat until interacted" look on their own.
func _switch_button(text: String, active: bool, group: ButtonGroup) -> Button:
	var b := Button.new()
	b.text = text.to_upper()
	b.toggle_mode = true
	b.button_pressed = active
	b.button_group = group
	b.focus_mode = Control.FOCUS_NONE
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.custom_minimum_size.y = 28
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("font_pressed_color", DccTheme.c("bg"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("sunken")))
	b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent")))
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("raised")))
	return b

# -- §5.1 Generation Pipeline --------------------------------------------------

func _build_pipeline(parent: Control) -> void:
	DccWidgets.action(parent, "Generate world", _on_generate_pressed, true)
	parent.add_child(DccTheme.rule())

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

	DccWidgets.note(body, "needs — %s" % String(stage["needs"]))
	DccWidgets.note(body, "produces — %s" % String(stage["produces"]))
	if not String(stage["gap"]).is_empty():
		DccWidgets.note(body, String(stage["gap"]))

	var run_row := HBoxContainer.new()
	run_row.add_theme_constant_override("separation", 6)
	var run_pad := MarginContainer.new()
	run_pad.add_theme_constant_override("margin_left", 14)
	run_pad.add_theme_constant_override("margin_top", 4)
	run_pad.add_child(run_row)
	body.add_child(run_pad)

	var run_tip := "generate_terrain is one-shot: the engine has no per-stage re-execution. Use Generate world above to run the whole pipeline."
	var run_one := DccWidgets.action(run_row, "Run stage %s" % number, func(): pass)
	run_one.disabled = true
	run_one.tooltip_text = run_tip
	if index < STAGES.size() - 1:
		var run_chain := DccWidgets.action(run_row, "Run %s → 10" % number, func(): pass)
		run_chain.disabled = true
		run_chain.tooltip_text = run_tip

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
		DccWidgets.toggle(parent, label, bool(bridge.param_get(key)),
			_on_bool_row_changed.bind(key, stage_index), hint)
		return

	var is_int := kind == "int"
	DccWidgets.slider(parent, label, float(info.get("min", 0.0)), float(info.get("max", 1.0)),
		float(info.get("step", 0.01)), float(bridge.param_get(key)), unit,
		_on_float_row_changed.bind(key, stage_index, is_int), hint)

## Named rather than inline: a row's callback both writes the engine and
## touches this dock's own stage-state bookkeeping, and a multi-line lambda
## that still has to close a call with a trailing argument after it (`hint)`)
## is exactly the shape that has bitten this project before -- a named method
## bound with `.bind()` sidesteps the question entirely. `.bind()` appends its
## arguments AFTER the caller's own, so the changed value comes first here.
func _on_bool_row_changed(v: bool, key: String, stage_index: int) -> void:
	bridge.param_set(key, v)
	_touch_stage(stage_index)

func _on_float_row_changed(v: float, key: String, stage_index: int, is_int: bool) -> void:
	bridge.param_set(key, (int(round(v)) if is_int else v))
	_touch_stage(stage_index)

func _touch_stage(index: int) -> void:
	_last_edited_stage = index
	_refresh_stage_states()

## §5.1's state column, honestly computed rather than faked: "resolved" means
## the last full generate covered this stage and nothing since has touched it
## or anything upstream of it; "editing" is the one row actually just
## changed; "stale" is every row at or after it in dependency order, per the
## spec's own propagation rule. There is deliberately no per-stage recompute
## behind any of this -- it is dock bookkeeping, not an engine capability.
func _refresh_stage_states() -> void:
	for i in _stage_state_labels.size():
		var lbl: Label = _stage_state_labels[i]
		if not bridge.has_world:
			lbl.text = "no world"
			lbl.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		elif _last_edited_stage == i:
			lbl.text = "%s editing" % DccIcons.SYMBOLS["on"]
			lbl.add_theme_color_override("font_color", DccTheme.c("accent"))
		elif (_last_edited_stage >= 0 and i > _last_edited_stage) or bridge.params_dirty:
			lbl.text = "%s stale" % DccIcons.SYMBOLS["off"]
			lbl.add_theme_color_override("font_color", DccTheme.c("stale"))
		else:
			lbl.text = "%s resolved" % DccIcons.SYMBOLS["tick"]
			lbl.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	_push_dock_readout()

## §3's rail-foot stage counter ("04 / 10"), repurposed as the collapsed left
## dock's own primary readout (§6: a collapsed dock keeps its one essential
## number, never blanks). Same honest basis as _refresh_stage_states() above --
## a count of GUI-tracked stage state, not a real per-stage completion signal.
func _push_dock_readout() -> void:
	if app == null:
		return
	if not bridge.has_world:
		app.set_dock_readout("left", "no world")
	elif _last_edited_stage >= 0:
		app.set_dock_readout("left", "%02d / 10 stale" % (_last_edited_stage + 1))
	else:
		app.set_dock_readout("left", "10 / 10 resolved")

## The dock's own primary action. `generate_sized`/`generate_world_structure_sized`
## are the ONE call site (`EngineBridge.generate`) -- this just supplies the
## request dictionary the New World dialog already keeps current, so pressing
## this after tuning stage rows above regenerates with today's live parameter
## table plus whatever creation-time shape (seed, extent, resolution, archetype)
## the dialog last held.
func _on_generate_pressed() -> void:
	if app == null or app.new_world_dialog == null:
		return
	bridge.generate(app.new_world_dialog.request())

# -- §5.2 Sculpt ----------------------------------------------------------------

func _build_sculpt(parent: Control) -> void:
	var body := DccWidgets.section(parent, "Sculpt")
	DccWidgets.note(body,
		"cartalith-terrain/src/sculpt.rs implements the full registry §5.2 specifies -- all thirteen geological features (Mountains, Hills, Ridge, Plateau, Cliff/Escarpment, Canyon, Valley, River, Lake, Basin, Coastline, Volcano, Freehand), eight presets, eight brush-shape falloffs and eight Freehand sub-modes -- but cartalith-godot exports no sculpt, stamp or commit method. Nothing here can be wired until that binding lands (STRANDED_TOOLS.md rows 4-8, UNIFIED_TOOL_PLAN.md milestone F).")
	DccWidgets.note(body,
		"One spec-vs-engine detail worth recording before that binding lands: §5.2's commit prose says it \"re-runs erosion, hydrology and climate once\", but commit_sculpt_pass deliberately marks tiles stale instead -- the eager form measured about 7 s per stroke at 2048².")
