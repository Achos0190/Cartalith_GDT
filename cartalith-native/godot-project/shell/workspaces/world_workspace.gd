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
## The Sculpt half (§5.2) and the Biome-paint tool (§4.5.2's "Biome paint" row)
## are both wired now, against `cartalith-godot`'s milestone F bindings
## (`sculpt_bridge.rs`, `paint_bridge.rs`, exposed through `EngineBridge`'s own
## "Milestone F tool bindings" section). `_build_sculpt`/`_build_paint` below
## read every feature/preset/global/palette table live off the engine rather
## than hardcoding `DCC_SHELL_SPEC.md` §5.2's own table -- `get_sculpt_features`
## already returns each feature's controls (key/label/min/max/step/default), so
## there is nothing here that could drift from the engine's own registry.

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
	 "groups": ["tectonics"], "keys": [],
	 "gap": "Structured-orogeny tuning (fold intensity, trench depth, fault blocks -- the reference's foldI/trenchD/faultB) is not exposed: generate_terrain hardcodes the exact values the reference's own defaults produce (0.16, 1.0, 0), so behaviour matches, but the three dials would each need threading through OrogenyParams' call site (GENERATION_PARAMETERS.md, \"Parameters the reference exposed that this port does not\")."},
	{"name": "Volcanism & impacts", "needs": "04 Tectonics",
	 "produces": "cones, provinces, craters → 06 Erosion",
	 "groups": ["volcanism"], "keys": [], "gap": ""},
	{"name": "Erosion", "needs": "04 Tectonics, 08 Climate",
	 "produces": "final surface → 07 Hydrology, 10 Resources & soils",
	 "groups": ["erosion"], "keys": [],
	 "gap": "Only the stream-power carve is ported. Droplet hydraulic, Hillslope diffuse, Velocity (momentum), Glacial and Coastal are each a separate manual pass in the reference with no cartalith-engine equivalent -- the groups below for those five are honest placeholders, not missing controls. Two more reference passes have no group at all because they are not passes over this stage's own inputs: Evolve climate <-> terrain (evoCyc / state.stream.cycles, read only by evolveCoupled()) re-runs erosion and climate against each other for n cycles, and Sediment fill deposits into basins afterward -- neither has a cartalith-engine equivalent."},
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
var _paint_body: VBoxContainer
var _stage_state_labels: Array = []  ## stage index -> the trailing state Label.

## The in-progress Sculpt stroke's captured points (grid-cell coords), tracked
## here in parallel with the engine the same way `GlobalTools._measure_points`
## tracks Measure's -- `sculpt_add_point` has no readback of its own, so the
## drawn path preview needs a local copy of where the clicks actually landed.
var _sculpt_stroke_points: PackedVector2Array = PackedVector2Array()

## Biome paint's live brush state, mirrored here because `paint_set_brush`'s
## own contract is "apply and echo back what was stored", not "read the
## current brush" -- there is no `paint_get_brush`. Defaults match
## `Brush::default()` in `paint_bridge.rs` exactly, so an untouched panel and
## an untouched engine agree before the first dab.
var _paint_layer := "biome"
var _paint_brush := {
	"value": 1, "radius": 6.0, "hardness": 1.0, "softness": 0.0,
	"erase": false, "land_only": true,
}

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
	## §4.5: every left dock opens with the TOOLS block, the four global tools
	## then the domain's own. WORLD's own row per §4.5.2's table is really
	## three: "Sculpt features (13)", "Freehand" and "Biome paint" -- but the
	## first two arm through the Sculpt panel's own feature picker instead of
	## a TOOLS-block button (`_build_feature_picker` below): each of its 13
	## icon buttons both selects that feature AND arms the same shared
	## "sculpt" tool id, since Freehand is simply the 13th entry in
	## `FEATURE_KEYS`/`get_sculpt_features()` -- a different `FeatureParams`
	## variant plus an extra sub-mode row, not a structurally separate record
	## the way Settlement/POI are in CIVIL. So the only button that belongs
	## here is Biome paint.
	DccWidgets.tools_block(self, app, app.tool_group, [
		{"id": "paint", "glyph": "tool_paint", "label": "Biome paint (B)"},
	])

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

	## Biome paint has no position of its own in §5's two-button switch
	## (Generation pipeline | Sculpt) -- per §4.5.2 its real estate is the
	## tool options bar plus a right-dock legend, and this file's own task
	## boundary keeps the tool options bar (app.gd) out of scope. Hosted here
	## instead as its own panel, shown whenever the Biome-paint tool is armed
	## regardless of which of the two switch positions is selected -- the
	## same "arming a tool never changes the workspace" independence §4.5
	## already establishes for every other domain.
	_paint_body = VBoxContainer.new()
	_paint_body.add_theme_constant_override("separation", 0)
	_paint_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_paint_body.visible = false
	add_child(_paint_body)
	_build_paint(_paint_body)

	pipeline_btn.pressed.connect(_select_mode.bind("pipeline"))
	sculpt_btn.pressed.connect(_select_mode.bind("sculpt"))

	app.register_tool_click_handler("sculpt", _sculpt_click)
	app.register_tool_drag_handler("sculpt", _sculpt_drag)
	app.register_tool_release_handler("sculpt", _sculpt_release)
	app.register_tool_escape_handler("sculpt", _sculpt_escape)
	app.register_tool_click_handler("paint", _paint_click)
	app.register_tool_drag_handler("paint", _paint_drag)
	app.register_tool_release_handler("paint", _paint_release)
	app.tool_armed.connect(_on_tool_armed)

	bridge.generation_started.connect(_refresh_stage_states)
	bridge.generation_finished.connect(_on_generation_finished)
	bridge.world_loaded.connect(_on_world_loaded)
	_refresh_stage_states()

func _select_mode(mode: String) -> void:
	_pipeline_body.visible = mode == "pipeline"
	_sculpt_body.visible = mode == "sculpt"
	if mode == "sculpt" and app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

## A new/loaded world means a fresh (or absent) `SculptEditor`/`PaintEditor`
## on the Rust side -- both panels rebuild from scratch rather than trusting
## whatever they showed for the previous world.
func _on_generation_finished(_ok: bool) -> void:
	_refresh_stage_states()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)

func _on_world_loaded() -> void:
	_refresh_stage_states()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)

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
	bake.tooltip_text = "No bake pipeline exists: LOD tiles are synthesized on demand at deep zoom (lod_synthesize_tile, LOD_TILING_INTEGRATION_SCOPE.md) and never written anywhere, so there is no frozen atlas to bake into and no finalize-lock state to enter. Finalizing would lock stages 01-10 and Sculpt; there is nothing here to lock against yet."

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
#
# Every table this panel draws (features + their own controls, presets, the
# eight brush/noise globals) comes straight off `bridge.get_sculpt_features()`
# / `get_sculpt_presets()` / `get_sculpt_globals_info()` -- none of §5.2's own
# table is hand-copied here, so this panel cannot drift from the registry the
# way a hardcoded copy could. `parent` is always `_sculpt_body`; this function
# tears down and rebuilds its whole subtree on every call, the same
# wholesale-rebuild discipline `right_dock.gd`'s own `_rebuild()` already uses,
# because a feature switch, a preset, or a stroke ending each change which
# controls belong on screen, not just a value within them.

func _build_sculpt(parent: Control) -> void:
	for child in parent.get_children():
		parent.remove_child(child)
		child.queue_free()

	if not bridge.has_world:
		var sec := DccWidgets.section(parent, "Sculpt")
		DccWidgets.note(sec, "Generate a world first -- the Sculpt editor is created fresh per generated world (World ▸ Generation pipeline).")
		return
	var globals_now := bridge.sculpt_get_globals()
	if globals_now.is_empty():
		var sec2 := DccWidgets.section(parent, "Sculpt")
		DccWidgets.note(sec2, "No sculpt editor for this world -- a loaded save has no draft session, only a freshly generated world does (sculpt_bridge.rs's own field doc).")
		return

	_build_feature_picker(parent)
	_build_presets(parent)
	_build_feature_params(parent)
	if bridge.sculpt_get_feature() == "freehand":
		_build_freehand_modes(parent)
	_build_brush_globals(parent)
	_build_sculpt_unbuilt_note(parent)
	_build_sculpt_draft(parent)

## §5.2's `#sculptFeatureSeg` -- 13 icon buttons sharing `app.tool_group`, so
## exactly one can read "armed" at a time, same as every other tool in the
## app. Clicking one both selects that feature (`sculpt_set_feature`, which
## resets its parameters to the registry's own defaults -- the reference's
## own behaviour on a feature switch) and arms the shared "sculpt" tool.
func _build_feature_picker(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Geological feature")
	var current := bridge.sculpt_get_feature()
	var grid := GridContainer.new()
	grid.columns = 5
	grid.add_theme_constant_override("h_separation", 2)
	grid.add_theme_constant_override("v_separation", 2)
	sec.add_child(grid)
	var hint := ""
	for f in bridge.get_sculpt_features():
		var d: Dictionary = f
		var key := String(d.get("key", ""))
		var label_text := String(d.get("label", key))
		var feature_hint := String(d.get("hint", ""))
		if key == current:
			hint = feature_hint
		var btn := DccWidgets.tool_button(grid, key, "%s -- %s" % [label_text, feature_hint],
			app.tool_group, _on_feature_button_armed.bind(key))
		## `set_pressed_no_signal`, not `.button_pressed =`, so restoring the
		## visually-armed state on a rebuild never re-fires `toggled` -- that
		## would call `_on_feature_button_armed` again and reset this
		## feature's own live parameters back to their registry defaults,
		## discarding whatever the user had just tuned.
		if app.armed_tool == "sculpt" and current == key:
			btn.set_pressed_no_signal(true)
	if not hint.is_empty():
		DccWidgets.note(sec, hint)

func _on_feature_button_armed(key: String) -> void:
	bridge.sculpt_set_feature(key)
	app.arm_tool("sculpt")
	_build_sculpt(_sculpt_body)

## §5.2's `#sculptPresetSeg` -- eight one-click parameter seeds. "A preset
## sets the feature and its parameters; it never paints" (§5.2 verbatim), so
## this arms the tool the same way a feature button does but draws no stroke.
func _build_presets(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Presets")
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 2)
	sec.add_child(grid)
	var presets := bridge.get_sculpt_presets()
	for i in presets.size():
		var d: Dictionary = presets[i]
		DccWidgets.action(grid, String(d.get("name", "Preset %d" % i)), _on_preset_pressed.bind(i))
	DccWidgets.note(sec, "A preset seeds the feature and its own parameters -- it never paints; draw the stroke yourself afterward.")

func _on_preset_pressed(index: int) -> void:
	bridge.sculpt_apply_preset(index)
	app.arm_tool("sculpt")
	_build_sculpt(_sculpt_body)

## The currently-selected feature's own registry entry (`get_sculpt_features`'
## own shape: key/label/hint/radial/modes/controls), or `{}` before any
## `generate()` call.
func _current_feature_meta() -> Dictionary:
	var current := bridge.sculpt_get_feature()
	for f in bridge.get_sculpt_features():
		var d: Dictionary = f
		if String(d.get("key", "")) == current:
			return d
	return {}

## §5.2's `#sculptFeatureControls` -- the selected feature's own controls,
## titled with the feature's name, live values from `sculpt_get_feature_params`.
func _build_feature_params(parent: Control) -> void:
	var meta := _current_feature_meta()
	if meta.is_empty():
		return
	var sec := DccWidgets.section(parent, String(meta.get("label", "Feature")) + " parameters")
	var live := bridge.sculpt_get_feature_params()
	var controls: Array = meta.get("controls", [])
	for c in controls:
		var cd: Dictionary = c
		var key := String(cd.get("key", ""))
		var clabel := String(cd.get("label", key))
		var cmin := float(cd.get("min", 0.0))
		var cmax := float(cd.get("max", 1.0))
		var cstep := float(cd.get("step", 0.01))
		var cval := float(live.get(key, cd.get("default", 0.0)))
		DccWidgets.slider(sec, clabel, cmin, cmax, cstep, cval, "", _on_feature_param_changed.bind(key))

func _on_feature_param_changed(v: float, key: String) -> void:
	bridge.sculpt_set_feature_params({key: v})

## §5.2's `#sculptModeSeg`, shown only for Freehand -- "Raise/Lower/Smooth
## follow the drag; Cliff/Ridge/Canyon follow its direction; Mesa/Volcano
## stamp once at a tap."
func _build_freehand_modes(parent: Control) -> void:
	var modes := bridge.get_sculpt_freehand_modes()
	if modes.is_empty():
		return
	var sec := DccWidgets.section(parent, "Freehand · direct drag")
	var current := bridge.sculpt_get_freehand_mode()
	var options: Array = []
	var selected_index := 0
	for i in modes.size():
		options.append(String(modes[i]).capitalize())
		if String(modes[i]) == current:
			selected_index = i
	DccWidgets.choice(sec, "Sub-mode", options, selected_index, _on_freehand_mode_changed.bind(modes))
	DccWidgets.note(sec, "Raise/Lower/Smooth follow the drag; Cliff/Ridge/Canyon follow its direction; Mesa/Volcano stamp once at a tap.")

func _on_freehand_mode_changed(i: int, modes: PackedStringArray) -> void:
	bridge.sculpt_set_freehand_mode(String(modes[i]))

## §5.2's "Brush & noise · global" table -- applies to every feature. The
## eight controls (`#sBrush`…`#sSeed`) come from `get_sculpt_globals_info()`;
## the seed row is its own thing since a dice button has no `Control` entry.
func _build_brush_globals(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Brush & noise · global")
	var live := bridge.sculpt_get_globals()
	for c in bridge.get_sculpt_globals_info():
		var cd: Dictionary = c
		var key := String(cd.get("key", ""))
		var clabel := String(cd.get("label", key))
		var cmin := float(cd.get("min", 0.0))
		var cmax := float(cd.get("max", 1.0))
		var cstep := float(cd.get("step", 0.01))
		var cval := float(live.get(key, cd.get("default", 0.0)))
		var is_int := String(cd.get("type", "float")) == "int"
		var unit := " px" if key == "brush_size" else ""
		DccWidgets.slider(sec, clabel, cmin, cmax, cstep, cval, unit, _on_global_changed.bind(key, is_int))

	var seed_row := HBoxContainer.new()
	seed_row.add_theme_constant_override("separation", 8)
	seed_row.custom_minimum_size.y = 24
	sec.add_child(seed_row)
	var seed_label := DccTheme.mono_label("Seed", "text_dim", DccTheme.FS_SMALL)
	seed_label.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	seed_row.add_child(seed_label)
	var seed_readout := DccTheme.mono_label(str(bridge.sculpt_get_seed()), "text", DccTheme.FS_SMALL)
	seed_readout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	seed_row.add_child(seed_readout)
	var dice := Button.new()
	dice.icon = DccIcons.get_icon("dice", 12)
	dice.focus_mode = Control.FOCUS_NONE
	dice.custom_minimum_size = Vector2(22, 22)
	dice.tooltip_text = "Randomise the seed the next stroke will capture."
	dice.add_theme_stylebox_override("normal", DccTheme.empty())
	dice.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft"), 2))
	dice.pressed.connect(_on_sculpt_seed_dice)
	seed_row.add_child(dice)

func _on_global_changed(v: float, key: String, is_int: bool) -> void:
	bridge.sculpt_set_globals({key: (round(v) if is_int else v)})

func _on_sculpt_seed_dice() -> void:
	bridge.sculpt_set_seed(randi())
	_build_sculpt(_sculpt_body)

## Spec header correction #3: Brush shape, Stroke & grid and Actions have no
## engine behind them and are not in the reference HTML either -- honest
## prose instead of building fake controls for them.
func _build_sculpt_unbuilt_note(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	DccWidgets.note(sec,
		"Brush shape (8 falloff shapes, Import brush, custom Falloff), Stroke & grid " +
		"(Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align control-point " +
		"editing) and Actions (Flip X/Y, Rot Left/Right, Flatten) have no engine behind them " +
		"and are not in the reference HTML either -- DCC_SHELL_SPEC.md header correction #3. " +
		"New, unscoped design work, not a port gap.")

## The draft/stamp-stack summary and Commit/Discard -- §5.2 places these at
## the foot of the left dock (`#sculptCommitBtn`/`#sculptDiscardBtn`); the
## full stamp-by-stamp list with its own Undo/Redo lives in the right dock
## (§6, `right_dock.gd`'s `_build_sculpt`) since that is where §6 puts it.
func _build_sculpt_draft(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Draft")
	var count := bridge.sculpt_stamp_count()
	DccWidgets.note(sec, "%d stamp%s on the draft." % [count, "" if count == 1 else "s"])

	var actions := DccWidgets.group(sec, "Commit")
	var commit_btn := DccWidgets.action(actions, "%s Commit to map" % DccIcons.SYMBOLS["tick"], _on_sculpt_commit, true)
	commit_btn.disabled = count == 0
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_sculpt_discard)
	discard_btn.disabled = count == 0
	DccWidgets.note(sec,
		"Commit bakes the whole stamp stack into the heightfield in one pass and marks the " +
		"tiles it touched stale -- it deliberately does not re-run erosion, hydrology or " +
		"climate (measured ~7s/stroke at 2048² and rejected on that ground; " +
		"DCC_SHELL_SPEC.md header correction #1). No finalize/lock state exists to note here " +
		"either -- see the Finalize section above: no bake/LOD pipeline exists yet.")

func _on_sculpt_commit() -> void:
	bridge.sculpt_commit("sculpt")
	## `build_color_texture()` reads the live (now-baked) field fresh on every
	## call, so setting it directly is enough -- `ViewportHost.refresh()`
	## would also reset the camera to fit, an unwanted side effect of every
	## Commit that this avoids by writing the public `map_view` field instead.
	app.viewport.map_view.texture = bridge.color_texture()
	app.viewport.set_preview_texture(null)
	_build_sculpt(_sculpt_body)
	if app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

func _on_sculpt_discard() -> void:
	bridge.sculpt_discard()
	app.viewport.set_preview_texture(null)
	_build_sculpt(_sculpt_body)
	if app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

# -- §5.2 Sculpt: stroke capture (map_clicked/map_dragged/map_released) --------
#
# A drag is always `map_clicked` (the press) then zero or more `map_dragged`
# (each motion sample) then exactly one `map_released` (`viewport_host.gd`'s
# own signal doc). `sculpt_begin_stroke`/`sculpt_add_point`/`sculpt_end_stroke`
# map onto that 1:1; `_sculpt_drag`'s own "begin if empty" guard covers the one
# case where press and drag disagree -- a press that lands off the plate (no
## `map_clicked` at all, per `map_overlay.gd`) followed by a drag that moves
# onto it (`map_dragged` fires once valid).

func _sculpt_click(gx: float, gy: float) -> void:
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gx, gy)
	_sculpt_stroke_points = PackedVector2Array([Vector2(gx, gy)])
	app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)

func _sculpt_drag(gx: float, gy: float) -> void:
	if _sculpt_stroke_points.is_empty():
		bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gx, gy)
	_sculpt_stroke_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)

## `build_sculpt_preview_texture()` is only called here, on release -- calling
## it mid-drag would show nothing new, since the in-progress stroke isn't a
## stamp (and so isn't part of the draft the preview composites) until this
## point; `set_path_preview`'s teal polyline is what shows live progress
## during the drag itself.
func _sculpt_release(_gx: float, _gy: float, _valid: bool) -> void:
	if _sculpt_stroke_points.is_empty():
		return
	bridge.sculpt_end_stroke()
	_sculpt_stroke_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)
	app.viewport.set_preview_texture(bridge.build_sculpt_preview_texture())
	_build_sculpt(_sculpt_body)
	if app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

## Sculpt isn't one of §4.5.6's three Escape-keeps-tool-armed exceptions
## (Way/Route/Measure), so this replicates `app.gd`'s own default disarm
## after cleaning up the in-progress stroke -- the same pattern
## `GlobalTools._region_escape` already uses for the same reason.
func _sculpt_escape() -> void:
	bridge.sculpt_cancel_stroke()
	_sculpt_stroke_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)
	var btn: BaseButton = app.tool_group.get_pressed_button()
	if btn != null:
		btn.button_pressed = false
	app.arm_tool("inspect")

## Leaving "sculpt" for any other tool must not strand an in-progress stroke
## (rare -- a hotkey pressed mid-drag -- but `sculpt_cancel_stroke` is a safe
## no-op otherwise). Arming "sculpt" pins the right dock to the stamp stack;
## leaving both Sculpt and Paint hides the brush cursor, which only either of
## those two tools ever shows.
func _on_tool_armed(id: String) -> void:
	if id != "sculpt" and not _sculpt_stroke_points.is_empty():
		bridge.sculpt_cancel_stroke()
		_sculpt_stroke_points = PackedVector2Array()
		app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)
	if id != "sculpt" and id != "paint":
		app.viewport.tool_overlay.set_brush_cursor(false, 0.0, 0.0, 0.0)
	if id == "sculpt" and app.right_dock_ctrl.has_method("show_sculpt_stack"):
		app.right_dock_ctrl.show_sculpt_stack()

## §10's brush ring, wired from `on_cursor_sampled` per the tool-arming
## substrate's own instructions -- `app.gd`'s `_wire_selection` forwards every
## viewport cursor sample to any workspace that implements this method.
func on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	if app == null or app.viewport == null or app.viewport.tool_overlay == null:
		return
	var overlay := app.viewport.tool_overlay
	if app.armed_tool == "sculpt":
		overlay.set_brush_cursor(valid, gx, gy, _sculpt_brush_radius_cells())
	elif app.armed_tool == "paint":
		overlay.set_brush_cursor(valid, gx, gy, float(_paint_brush.get("radius", 6.0)))
	else:
		overlay.set_brush_cursor(false, 0.0, 0.0, 0.0)

## §5.2: "Radial features show their radius control here rather than using
## the global brush size" -- Volcano's own `volcRadius` control is the one
## case among the 13 with a control literally named that; every other
## feature (including Lake, whose own table row is "radial, brush = radius"
## with no radius control of its own) falls back to the shared global
## `brush_size`. A simplification, not a full per-feature falloff-shape
## reproduction -- §5.2's Brush shape block is explicitly not built (see
## `_build_sculpt_unbuilt_note`).
func _sculpt_brush_radius_cells() -> float:
	if not bridge.has_world:
		return 0.0
	var radius := float(bridge.sculpt_get_globals().get("brush_size", 0.0))
	var params := bridge.sculpt_get_feature_params()
	for k in params.keys():
		if String(k).to_lower().ends_with("radius"):
			radius = float(params[k])
	return radius

# -- §4.5.2 Biome paint ---------------------------------------------------------
#
# `get_paint_layers()`/`get_paint_palette()` are the live registry (three
# layers -- Biome/Terrain/Splat, `paint_bridge.rs`'s own "answered" note on
# which fields `PaintStamp` may legally write); nothing here hardcodes §4.5.2's
# own target table.

func _build_paint(parent: Control) -> void:
	for child in parent.get_children():
		parent.remove_child(child)
		child.queue_free()

	if not bridge.has_world:
		var sec := DccWidgets.section(parent, "Biome paint")
		DccWidgets.note(sec, "Generate a world first.")
		return
	var layers := bridge.get_paint_layers()
	if layers.is_empty():
		var sec2 := DccWidgets.section(parent, "Biome paint")
		DccWidgets.note(sec2, "No paint editor for this world -- a loaded save has no draft session, same ceiling as Sculpt.")
		return

	var sec := DccWidgets.section(parent, "Biome paint")
	DccWidgets.note(sec,
		"§4.5.2's PAINT · BIOME tool options row, hosted in this dock -- this port's real " +
		"tool options bar (app.gd) is outside this file's own task boundary.")

	var layer_options: Array = []
	var layer_index := 0
	for i in layers.size():
		layer_options.append(String(layers[i]).capitalize())
		if String(layers[i]) == _paint_layer:
			layer_index = i
	DccWidgets.choice(sec, "Target field", layer_options, layer_index, _on_paint_layer_changed.bind(layers))

	var palette := bridge.get_paint_palette(_paint_layer)
	if not palette.is_empty():
		var value_options: Array = []
		var value_index := 0
		for i in palette.size():
			var pd: Dictionary = palette[i]
			value_options.append(String(pd.get("label", "?")))
			if int(pd.get("index", -1)) == int(_paint_brush["value"]):
				value_index = i
		DccWidgets.choice(sec, "Value", value_options, value_index, _on_paint_value_changed.bind(palette))

	DccWidgets.slider(sec, "Radius", 1.0, 40.0, 1.0, float(_paint_brush["radius"]), " cells", _on_paint_radius_changed)
	DccWidgets.slider(sec, "Hardness", 0.0, 1.0, 0.01, float(_paint_brush["hardness"]), "", _on_paint_hardness_changed,
		"Stored and echoed back but never consumed -- painting is a hard disc with no soft falloff (paint_bridge.rs's own module doc).")
	DccWidgets.slider(sec, "Softness", 0.0, 1.0, 0.01, float(_paint_brush["softness"]), "", _on_paint_softness_changed,
		"Stored and echoed back but never consumed, same as Hardness above.")
	DccWidgets.toggle(sec, "Erase", bool(_paint_brush["erase"]), _on_paint_erase_changed,
		"Every dab writes 0 (unpainted) regardless of Value. Holding Shift while painting does the same without changing this switch.")
	DccWidgets.toggle(sec, "Land only", bool(_paint_brush["land_only"]), _on_paint_land_only_changed,
		"Gates the dab against this world's water-body classification -- a toggle here, unlike the reference's hard-always gate (paint_bridge.rs's own module doc).")

	var counts: Dictionary = bridge.paint_painted_counts()
	var total := int(counts.get("total", 0))
	var legend := DccWidgets.group(sec, "Legend · painted counts")
	if total == 0:
		DccWidgets.note(legend, "Nothing painted yet on this layer.")
	else:
		var by_index: Dictionary = counts.get("counts", {})
		for i in palette.size():
			var pd2: Dictionary = palette[i]
			var idx := int(pd2.get("index", i + 1))
			var n := int(by_index.get(idx, 0))
			if n > 0:
				DccWidgets.note(legend, "%s -- %d" % [String(pd2.get("label", "?")), n])

	var actions := DccWidgets.group(sec, "Commit")
	var commit_btn := DccWidgets.action(actions, "%s Commit" % DccIcons.SYMBOLS["tick"], _on_paint_commit, true)
	commit_btn.disabled = total == 0
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_paint_discard)
	discard_btn.disabled = total == 0
	DccWidgets.note(sec,
		"Commit writes every layer's pending dabs into their own override arrays and marks " +
		"ecology/biomes and resources/soils stale -- it never touches height, hydrology or " +
		"climate. No renderer currently draws a painted cell into the real map; this preview " +
		"is this port's own overlay convention, not the reference's (paint_bridge.rs's own " +
		"swatch_color doc).")

func _on_paint_layer_changed(i: int, layers: PackedStringArray) -> void:
	_paint_layer = String(layers[i])
	bridge.paint_set_layer(_paint_layer)
	_paint_brush["value"] = 1
	_sync_paint_brush()
	_build_paint(_paint_body)

func _on_paint_value_changed(i: int, palette: Array) -> void:
	var pd: Dictionary = palette[i]
	_paint_brush["value"] = int(pd.get("index", 1))
	_sync_paint_brush()

func _on_paint_radius_changed(v: float) -> void:
	_paint_brush["radius"] = v
	_sync_paint_brush()

func _on_paint_hardness_changed(v: float) -> void:
	_paint_brush["hardness"] = v
	_sync_paint_brush()

func _on_paint_softness_changed(v: float) -> void:
	_paint_brush["softness"] = v
	_sync_paint_brush()

func _on_paint_erase_changed(v: bool) -> void:
	_paint_brush["erase"] = v
	_sync_paint_brush()

func _on_paint_land_only_changed(v: bool) -> void:
	_paint_brush["land_only"] = v
	_sync_paint_brush()

func _sync_paint_brush() -> void:
	bridge.paint_set_brush(
		int(_paint_brush["value"]), float(_paint_brush["radius"]),
		float(_paint_brush["hardness"]), float(_paint_brush["softness"]),
		bool(_paint_brush["erase"]), bool(_paint_brush["land_only"]))

func _on_paint_commit() -> void:
	var summary: Dictionary = bridge.paint_commit()
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture())
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	_build_paint(_paint_body)

func _on_paint_discard() -> void:
	bridge.paint_discard()
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture())
	_build_paint(_paint_body)

# -- §4.5.2 Biome paint: stroke capture (map_clicked/map_dragged/map_released) -
#
# Paint has no begin/end pair (`paint_stroke_at`'s own doc: every call is
## already one complete, independently undo-able draft entry), so click and
# drag both just apply one dab; release only refreshes the panel (painted
# counts, Commit's disabled state) once per gesture rather than once per
# motion sample, since the panel rebuild itself is not cheap enough to do on
# every dab the way the live preview texture already is.

func _paint_apply_dab(gx: float, gy: float) -> void:
	## §4.5.2: "Drag paints cells, ⇧ erases" -- Shift is a momentary modifier
	## on top of whatever the Erase toggle already says, not a replacement
	## for it, so this ORs the two rather than overwriting `_paint_brush`.
	var shift := Input.is_key_pressed(KEY_SHIFT)
	bridge.paint_set_brush(
		int(_paint_brush["value"]), float(_paint_brush["radius"]),
		float(_paint_brush["hardness"]), float(_paint_brush["softness"]),
		bool(_paint_brush["erase"]) or shift, bool(_paint_brush["land_only"]))
	bridge.paint_stroke_at(gx, gy)
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture())

func _paint_click(gx: float, gy: float) -> void:
	_paint_apply_dab(gx, gy)

func _paint_drag(gx: float, gy: float) -> void:
	_paint_apply_dab(gx, gy)

func _paint_release(_gx: float, _gy: float, _valid: bool) -> void:
	if is_instance_valid(_paint_body):
		_build_paint(_paint_body)
