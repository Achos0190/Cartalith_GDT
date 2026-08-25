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
	 "gap": "Only the stream-power carve and the Glacial group's fjord carve are ported. Droplet hydraulic, Hillslope diffuse, Velocity (momentum), Glacial erosion itself and Coastal are each a separate manual pass in the reference with no cartalith-engine equivalent -- the groups below for those five are honest placeholders, not missing controls. Two more reference passes have no group at all because they are not passes over this stage's own inputs: Evolve climate <-> terrain (evoCyc / state.stream.cycles, read only by evolveCoupled()) re-runs erosion and climate against each other for n cycles, and Sediment fill deposits into basins afterward -- neither has a cartalith-engine equivalent."},
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

## **v3's nine WORLD categories** (2026-08-24, `design/Cartalith Menu Structure
## v3.dc.html`), and which of `STAGES` above each one hosts.
##
## v3's migration audit is explicit about what this table is doing: *"Split by
## subject rather than by run order … The numbered 01-10 stage list disappears
## as navigation and survives as pipeline status."* So the pipeline is still
## exactly the ten stages, and `generate()` still runs all ten in one call --
## nothing about the engine changed. What changed is that the dock is now
## organised by what a control **is about** rather than by when it runs, which
## is what makes "where do I set river density" answerable without knowing that
## hydrology is stage 07.
##
## `stages` is in dependency order within a category, so a category hosting two
## stages still reads top-to-bottom the way the pipeline runs. A category with
## an empty list is one v3 names that the engine does not parameterise at all --
## it carries prose, never a dead control.
const CATEGORIES: Array = [
	{"name": "Generate", "stages": [2],
	 "lead": "The one act: seed, extent, steering, run. Every parameter in the eight categories below feeds this call, and this call resolves all ten pipeline stages at once -- there is no partial recompute in this engine or in the app it ports."},
	{"name": "Terrain", "stages": [5],
	 "lead": "The surface itself: what erosion does to it, and what a hand does to it. Elevation, slope, curvature and relief are readable as analysis fields -- Cartography ▸ Visibility / zoom ▸ Data overlays."},
	{"name": "Geology", "stages": [3, 4],
	 "lead": "What the rock is and where it was pushed: plates, uplift, volcanism, impacts and rock resistance. Everything here runs before erosion and is what erosion cuts into."},
	{"name": "Hydrology", "stages": [6],
	 "lead": "Rivers, lakes, drainage and flow accumulation, derived from the finished surface."},
	{"name": "Climate", "stages": [7],
	 "lead": "Temperature, rainfall, wind and currents, over the finished surface and under the planet's own geometry."},
	{"name": "Biomes", "stages": [8],
	 "lead": "Classification off the finished temperature/rainfall/elevation fields, and the brush that overrides it by hand. Biome *colours* are Cartography's -- v3's own split."},
	{"name": "Ecology", "stages": [],
	 "lead": ""},
	{"name": "Resources", "stages": [9],
	 "lead": "Soil, ore and fertility, downstream of geology, climate and biomes."},
	{"name": "World data", "stages": [0, 1],
	 "lead": "The planet the world sits on and the scale it is measured in. Everything here is an input to generation rather than a product of it."},
]

var _sculpt_body: VBoxContainer
var _paint_body: VBoxContainer
## ECOLOGY's whole body (`GUI_GAP_REGISTER.md` WW-14). Refilled wholesale on
## every generate/load for the same reason the two above are: every number in
## it is this world's.
var _ecology_body: VBoxContainer
## WORLD DATA's coordinate-system readout (`GUI_GAP_REGISTER.md` WW-15).
## Refilled on every generate/load: the frame is this world's, and before the
## first one there is no frame at all.
var _crs_body: VBoxContainer
var _stage_state_labels: Array = []  ## stage index -> the trailing state Label.

## §5.1's Finalize foot (`GUI_GAP_REGISTER.md` WW-01). `_bake_depth` defaults
## to 3 -- the reference's own `bakeAllDepth` default, and 85 tiles, which is
## the deepest bake that finishes in a plausible interactive wait.
var _bake_depth := 3
var _bake_button: Button
var _unfinalize_button: Button
var _bake_status: Label

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

	## **The two-button switch (Generation pipeline | Sculpt) is gone**
	## (2026-08-24, v3). It was a mode selector over one domain, and v3 has no
	## such control anywhere: Sculpt is a row inside TERRAIN, which is where a
	## person looking for "change the shape of the ground" would go. Removing it
	## also removes the one place in this shell where a dock had a hidden half
	## -- the accordion is now the only disclosure WORLD uses, like CIVIL and
	## CARTO. `_sculpt_body`/`_paint_body` still exist and are still rebuilt
	## wholesale on every generate; they are parented into their categories
	## instead of into a mode panel.
	_sculpt_body = VBoxContainer.new()
	_sculpt_body.add_theme_constant_override("separation", 0)
	_sculpt_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	## Biome paint stays a panel of its own, shown whenever the Biome-paint
	## tool is armed -- the same "arming a tool never changes the workspace"
	## independence §4.5 establishes for every other domain. It now lives
	## inside the BIOMES category rather than at the foot of the dock.
	_paint_body = VBoxContainer.new()
	_paint_body.add_theme_constant_override("separation", 0)
	_paint_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_paint_body.visible = false

	_ecology_body = VBoxContainer.new()
	_ecology_body.add_theme_constant_override("separation", 0)
	_ecology_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	_crs_body = VBoxContainer.new()
	_crs_body.add_theme_constant_override("separation", 0)
	_crs_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	_build_categories()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)

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

## A new/loaded world means a fresh (or absent) `SculptEditor`/`PaintEditor`
## on the Rust side -- both panels rebuild from scratch rather than trusting
## whatever they showed for the previous world.
func _on_generation_finished(_ok: bool) -> void:
	_refresh_stage_states()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)
	_fill_ecology(_ecology_body)
	_build_crs(_crs_body)

func _on_world_loaded() -> void:
	_refresh_stage_states()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)
	_fill_ecology(_ecology_body)
	_build_crs(_crs_body)

# -- v3's nine categories ------------------------------------------------------

## One L2 category per `CATEGORIES` row, each hosting whichever pipeline stages
## own its subject. Everything a stage contributes -- its dependency prose, its
## params.rs groups, its loose keys, its disclosed gap -- is drawn by the same
## `_build_stage_body()` the numbered list used; only the container changed.
func _build_categories() -> void:
	for i in CATEGORIES.size():
		var cat: Dictionary = CATEGORIES[i]
		var name := String(cat["name"])
		var body := DccWidgets.category(self, name, categories, i == 0)
		if not String(cat["lead"]).is_empty():
			DccWidgets.note(DccWidgets.pad(body, 14, 8, 12, 0), String(cat["lead"]))

		match name:
			"Generate": _build_generate_head(body)
			"Terrain": _build_terrain_head(body)
			"Biomes": pass
			"Ecology": _build_ecology(body)
			_: pass

		var stages: Array = cat["stages"]
		for s in stages:
			_build_stage_body(body, int(s), stages.size() > 1 or name != String(STAGES[int(s)]["name"]))

		match name:
			"Generate": _build_generate_foot(body)
			"Terrain": body.add_child(_sculpt_body)
			"Geology": _build_geology_foot(body)
			"Hydrology": _build_hydrology_foot(body)
			"Biomes": body.add_child(_paint_body)
			"World data": _build_world_data_foot(body)
			_: pass

## v3 puts the **river network** under HYDROLOGY, and asks for per-reach rows
## (navigability, discharge, catchment, tributaries). CIVIL's old Rivers
## category was the only place in the shell that disclosed why there are none;
## it was retired in the same pass that moved the subject here, so the finding
## has to be re-drawn here or it is simply gone. `rivers_note()` is its single
## owner (`GUI_GAP_REGISTER.md` IN-01).
func _build_hydrology_foot(parent: Control) -> void:
	## Deliberately NOT "River network" -- `KEYS_SECTION_TITLES` already gives
	## the stage's own carve/density dials that heading, and two sections with
	## one name in one category is how a reader ends up reading the wrong one.
	DccWidgets.note(DccWidgets.section(parent, "Not built"),
		InfrastructureWorkspace.rivers_note())

## v3 GENERATE's own top rows: the three global actions the reference calls
## `#genBtn` / `#reseedBtn` / `#centerBtn`, then the pipeline-status readout
## that is all that survives of the numbered stage list.
##
## The buttons are shortcuts onto `app.gd`'s own handlers, not second
## implementations -- the tool-options bar presses exactly the same three, and
## this shell has repeatedly been bitten by two controls with two independent
## state computations (the bake button, the recompute rows).
func _build_generate_head(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Run")
	var gen := DccWidgets.action(sec, "Generate world", _on_generate_pressed, true)
	gen.tooltip_text = "The reference's #genBtn. Runs all ten stages against the current parameters and the current seed. The same button sits in the tool options bar above the map."
	var seed_btn := DccWidgets.action(sec, "New seed", func(): app._new_seed())
	seed_btn.tooltip_text = "The reference's #reseedBtn. Rolls a new seed in File ▸ New world and regenerates from it."
	var centre := DccWidgets.action(sec, "Center landmasses", func(): app._center_landmasses())
	centre.tooltip_text = "The reference's #centerBtn. Rotates the world in longitude so the emptiest meridian sits at the map edge, then feathers the join it moved into the interior. Whole-world mode only; the outcome is reported in the status bar."

	var status := DccWidgets.section(parent, "Pipeline status")
	## The one surviving state readout. `_stage_state_labels` is still the
	## array `_refresh_stage_states()` writes to -- it just has one entry now
	## instead of ten, because ten identical labels was ten copies of one fact.
	_stage_state_labels.append(DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 1))
	status.add_child(_stage_state_labels[0])
	DccWidgets.note(status,
		"There is no partial recompute and no per-stage stale flag: one generate() "
		+ "resolves all ten stages, every call, in this engine and in the app it "
		+ "ports (verified live against the reference -- every parameter row here "
		+ "regenerates on release rather than waiting for a run button). What CAN "
		+ "go stale is the civilisation layer over an edited world, and that has "
		+ "its own badge and its own button: Civilization ▸ Settlements ▸ "
		+ "Recompute.")
	DccWidgets.note(status,
		"Resolution, working and render, is a creation-time call argument rather "
		+ "than a stored parameter -- File ▸ New world sets it. Map extent (world "
		+ "/ region) is the same.")

## v3 GENERATE's `› Bake & finalize` group, plus the LOD row beside it. The
## finalize foot is unchanged (WW-01); what v3 adds here is the disclosure that
## the reference's per-tile refine passes are not part of it.
func _build_generate_foot(parent: Control) -> void:
	_build_finalize(parent)

	var lod := DccWidgets.section(parent, "LOD terrain data")
	DccWidgets.note(lod,
		"v3 moves tile refine and atlas bake out of View and into this category, "
		+ "on the correct reasoning that both produce terrain *data*. The atlas "
		+ "half is the Bake above -- it writes every tile of the pyramid to disk. "
		+ "The refine half is not ported: the reference's per-tile Burn rivers and "
		+ "Micro-erode passes have no cartalith-spatial equivalent (pyramid_tile's "
		+ "own doc records that as deliberate), so deep zoom synthesises detail "
		+ "rather than re-eroding it. Auto-detail on zoom, tile size and the chunk "
		+ "debug overlay stay program scope -- Preferences ▸ Tiles & LOD.")

	var not_stage := DccWidgets.section(parent, "Not a generation stage")
	DccWidgets.note(not_stage,
		"GPU acceleration and multi-GPU → Preferences ▸ Performance. Render quality, lighting, 3D viewport → Preferences ▸ Graphics. Auto-detail on zoom, tile size, chunk debug → Preferences ▸ Tiles & LOD. Terrain appearance, style presets, ramps → Cartography. Settlements, routes, politics → Civilization.")

## v3 TERRAIN's head: the heightmap entry point, above the erosion passes.
func _build_terrain_head(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Heightmap")
	var load_btn := DccWidgets.action(sec, "Load heightmap…",
		func(): app.open_data_manager("Import"))
	load_btn.tooltip_text = "The reference's #loadBtn. Opens Data ▸ Import, whose Heightmaps route decodes a PNG, takes it as the elevation field and infers tectonics under it."
	DccWidgets.note(sec,
		"An imported heightmap replaces the generated surface. Tectonics are "
		+ "inferred from it rather than kept -- see Geology below for that pass "
		+ "on its own.")

## v3 GEOLOGY's foot: the one geology action that is not a parameter.
func _build_geology_foot(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "From an imported surface")
	var infer := DccWidgets.action(sec, "Infer tectonics from heightmap…",
		func(): app.open_data_manager("Import"))
	infer.tooltip_text = "The reference's #inferTectBtn. Runs as part of the heightmap import (cartalith_engine::import::infer_tectonics) -- there is no separate #[func] to re-run it over an already-imported surface, so this opens the import that performs it."

## v3 names ECOLOGY as its own category, and `GUI_GAP_REGISTER.md` **WW-14**
## registered it as having nothing behind it -- "ecological productivity and
## flora/fauna distribution do not exist in this port or in the reference: no
## crate computes either".
##
## **Both halves of that were wrong**, and this category is the correction.
## Productivity is `cartalith_civ::build_npp`, the Miami model, ported and
## golden-verified; fauna distribution is `cartalith_civ::wildlife`'s ecoregion
## segmentation with per-guild rosters and per-species population estimates,
## likewise. What was actually missing was any way to *reach* them from here:
## NPP was computed only inside `wildlife_regions` and discarded, and the
## ecoregion records were reachable only by clicking the map while the Wildlife
## debug view happened to be open.
##
## So this is a readout, not a parameter panel -- the engine genuinely has no
## ecology *dials*, which is the part of the old note that was true and is kept.
func _build_ecology(parent: Control) -> void:
	parent.add_child(_ecology_body)
	_fill_ecology(_ecology_body)

## Refilled on every generate/load, the same wholesale-rebuild discipline
## `_build_sculpt`/`_build_paint` use: every number here is this world's.
func _fill_ecology(parent: Control) -> void:
	for c in parent.get_children():
		c.queue_free()
		parent.remove_child(c)

	var eco := bridge.ecology_summary()
	var sec := DccWidgets.section(parent, "Productivity")
	if eco.is_empty():
		DccWidgets.note(sec,
			"No world yet. Net primary productivity, ecoregions and their fauna are "
			+ "all derived from a generated world's climate and biome fields.")
	else:
		DccWidgets.note(sec,
			("Net primary productivity averages %d g/m²/yr over %s land cells, "
			+ "peaking at %d. That is the Miami model -- the lower of a "
			+ "temperature and a precipitation ceiling, both capped at 3000 -- and "
			+ "it is the same field the wildlife scorer reads.")
			% [int(round(float(eco.get("npp_mean", 0.0)))),
				_thousands(int(eco.get("land_cells", 0))),
				int(round(float(eco.get("npp_max", 0.0))))])
	var npp := DccWidgets.action(sec, "Show productivity on the map",
		func():
			app.viewport.set_debug_layer("npp")
			app.set_status("hint", "Analysis field: Net primary productivity (g/m²/yr).", "text"))
	npp.alignment = HORIZONTAL_ALIGNMENT_LEFT
	npp.tooltip_text = "build_npp(): 0-3000 g/m²/yr of dry matter, land only. One of the Layers popover's analysis fields -- this is a shortcut onto that one picker, not a second copy of it."

	var fauna := DccWidgets.section(parent, "Fauna")
	var regions: Array = eco.get("regions", [])
	if regions.is_empty():
		DccWidgets.note(fauna,
			"No ecoregions. The segmentation runs over the Cartalith biome grid, "
			+ "which needs the civilisation layer's water bodies -- so a loaded "
			+ ".zip save has productivity but no fauna, the same condition the "
			+ "Wildlife and Biomes analysis fields already report.")
	else:
		DccWidgets.note(fauna,
			("%d ecoregions carrying %d species records between them. Each is a "
			+ "connected component of one biome class, scored on productivity, "
			+ "terrain ruggedness, water access and latitude, then given a guild "
			+ "roster with a population estimate per species.")
			% [int(eco.get("region_count", 0)), int(eco.get("species_total", 0))])
		for r: Dictionary in regions:
			DccWidgets.note(fauna,
				"%s — %s km², %d species, NPP %d" % [
					String(r.get("biome_name", "?")),
					_thousands(int(round(float(r.get("area_km2", 0.0))))),
					int(r.get("richness", 0)),
					int(round(float(r.get("npp", 0.0))))])
		DccWidgets.note(fauna,
			"The eight largest by area. Open the Wildlife field and click a region "
			+ "marker for its full guild roster.")
	var wild := DccWidgets.action(fauna, "Show fauna on the map",
		func():
			app.viewport.set_debug_layer("wildlife")
			app.set_status("hint", "Analysis field: Wildlife -- click a region marker for its roster.", "text"))
	wild.alignment = HORIZONTAL_ALIGNMENT_LEFT
	wild.tooltip_text = "current_wildlife(): ecoregions coloured by species richness. Clicking a marker fills the right dock with that region's guilds and per-species population estimates."

	var sec2 := DccWidgets.section(parent, "Not parameterised")
	DccWidgets.note(sec2,
		"Vegetation density and soil are computed off the finished biome, climate "
		+ "and lithology fields with no dials of their own in cartalith-engine, "
		+ "and neither has productivity: the Miami model's only tunable is "
		+ "state.climate.maxRainMm, which this port pins at the reference's own "
		+ "3000 default. Everything above is derived, not set.")
	DccWidgets.note(sec2,
		"Still missing (GUI_GAP_REGISTER.md WW-14): *flora* distribution as a "
		+ "species-level counterpart to the fauna rosters. The wildlife tables are "
		+ "animals only -- there is no plant-species vocabulary anywhere in "
		+ "cartalith-civ or in the reference, and biome class is as fine as the "
		+ "vegetation answer gets.")

## `1234567` -> `1,234,567`. Local rather than a `DccWidgets` addition: the two
## call sites are both in this file's Ecology readout.
func _thousands(v: int) -> String:
	var s := str(absi(v))
	var out := ""
	while s.length() > 3:
		out = "," + s.substr(s.length() - 3) + out
		s = s.substr(0, s.length() - 3)
	return ("-" if v < 0 else "") + s + out

## v3 WORLD DATA's foot: the field browser and the GeoJSON export, both of
## which already exist as program windows.
func _build_world_data_foot(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Read the fields")
	var tables := DccWidgets.action(sec, "World data tables…",
		func(): app.open_world_data())
	tables.alignment = HORIZONTAL_ALIGNMENT_LEFT
	var geo := DccWidgets.action(sec, "Export GeoJSON…",
		func(): app.open_data_manager("Export"))
	geo.alignment = HORIZONTAL_ALIGNMENT_LEFT
	geo.tooltip_text = "The reference's #exportGeoBtn. Data ▸ Export ▸ GIS writes coastlines, rivers, settlements, ways and territory as one FeatureCollection."
	parent.add_child(_crs_body)
	_build_crs(_crs_body)

## v3 WORLD DATA ▸ `Coordinate system · projection` — `GUI_GAP_REGISTER.md`
## **WW-15**, and a correction to it.
##
## The register said the export "writes a plain lon/lat-shaped frame with no
## CRS declared". It has always declared one, in the document's own `note`
## property (`geojson::CRS_NOTE`, quoted verbatim from the reference) — RFC
## 7946 deprecated the `crs` member, so a note is the declaration a GeoJSON
## file gets to make. What was missing is any way to read the frame *here*.
##
## And the frame is real, and it is two different ones depending on world
## mode, which is the part worth stating: the climate pipeline runs on real
## latitudes either way.
func _build_crs(parent: Control) -> void:
	for c in parent.get_children():
		c.queue_free()
		parent.remove_child(c)
	var sec := DccWidgets.section(parent, "Coordinate system")
	var crs := bridge.world_crs()
	if crs.is_empty():
		DccWidgets.note(sec, "No world yet.")
	else:
		DccWidgets.note(sec,
			("%s. Origin is the north-west cell; X runs east, Y runs south, and "
			+ "the GeoJSON export flips Y so north is up there.")
			% String(crs.get("frame", "?")).capitalize())
		DccWidgets.note(sec,
			("%d × %d cells over %.0f × %.0f km, so one cell is %.3f km on a side. "
			+ "Rows run %.1f° to %.1f° — %.4f° of latitude per row, which is what "
			+ "the climate model integrates over.")
			% [int(crs.get("grid_w", 0)), int(crs.get("grid_h", 0)),
				float(crs.get("map_width_km", 0.0)), float(crs.get("map_height_km", 0.0)),
				float(crs.get("cell_km", 0.0)),
				float(crs.get("lat_n", 0.0)), float(crs.get("lat_s", 0.0)),
				float(crs.get("deg_per_row", 0.0))])
		if bool(crs.get("world", false)):
			DccWidgets.note(sec,
				"World mode, so the latitudes are the planet's own 90°N–90°S and "
				+ "the Climate stage's own lat_n / lat_s are ignored. Longitude "
				+ "is not modelled: the X axis is kilometres, and it wraps.")
		else:
			DccWidgets.note(sec,
				"Regional mode. Latitude is real and drives the climate model; "
				+ "longitude is not modelled at all, and X does not wrap.")
		DccWidgets.note(sec, "Export declares: \"%s\"" % String(crs.get("export_note", "")))
	## `GUI_GAP_REGISTER.md` §42's Not-built anatomy.
	DccWidgets.note(sec,
		"Reprojection  ·  needs a decision\n"
		+ "Every field is grid space and nothing reprojects, so the planar "
		+ "kilometres are not a projection of the latitudes beside them, and a GIS "
		+ "reading them as WGS84 degrees is misreading the file. Which projection "
		+ "a fictional world should claim is an authoring decision, not a defect "
		+ "(GUI_GAP_REGISTER.md WW-15). Units are km-only; the reference's km/mi "
		+ "toggle is not ported (PR-15).\n"
		+ "The frame itself is declared and honest -- the export's own note says "
		+ "exactly this, and Coordinate system above reads it back in-app.")

## §5.1's dock foot — `GUI_GAP_REGISTER.md` **WW-01**, built 2026-08-24.
##
## The design canvas splits the reference's three controls cleanly (Bake depth ·
## Bake ALL levels & finalize · Un-finalize) where the shell had compressed all
## three into one disabled button; `GUI_GAP_REGISTER.md` §7's own note says to
## take the canvas's three-row split when WW-01 is built, so that is what this
## is.
##
## The depth row shows the tile count *before* the user commits, because depth 5
## is 1365 tiles and finding that out by waiting is not an acceptable way to
## learn it.
func _build_finalize(parent: Control) -> void:
	var foot := DccWidgets.section(parent, "Finalize")
	_bake_status = DccWidgets.note(foot, "")

	DccWidgets.choice(foot, "Bake depth", ["LOD 0–2", "LOD 0–3", "LOD 0–4", "LOD 0–5"],
		_bake_depth - 2,
		func(i: int):
			_bake_depth = i + 2
			_refresh_finalize(),
		"How deep the pyramid is baked. Level z holds 2^z x 2^z tiles, so the total is (4^(depth+1)-1)/3 -- depth 3 is 85 tiles, depth 4 is 341, depth 5 is 1365. Already-baked chunks are skipped, so raising the depth later only fills the gaps.")

	_bake_button = DccWidgets.action(foot, "Bake ALL levels & finalize", _on_bake_all, true)
	_bake_button.tooltip_text = "Pre-render every tile of the pyramid to the on-disk atlas, then lock the world. Deep zoom then reads bytes instead of re-synthesising octaves. Already-baked chunks are skipped. This blocks the UI while it runs -- see the size and tile count above before committing."
	_unfinalize_button = DccWidgets.action(foot, "Un-finalize", func():
		bridge.set_finalized(false)
		_refresh_finalize()
		if app != null and app.has_method("refresh_atlas_status"):
			app.refresh_atlas_status())
	_unfinalize_button.tooltip_text = "Unlock the world for further generation and sculpting. The baked atlas is left on disk: re-finalizing needs no re-bake unless a generation parameter actually changed."

	var clear := DccWidgets.action(foot, "Clear this world's atlas", func():
		bridge.atlas_clear()
		_refresh_finalize()
		if app != null and app.has_method("refresh_atlas_status"):
			app.refresh_atlas_status())
	clear.tooltip_text = "Delete every baked chunk for this world (Preferences ▸ Memory ▸ Clear caches). Un-finalizes too: a lock protecting nothing would strand the world read-only for no reason."

	_refresh_finalize()

func _on_bake_all() -> void:
	if bridge.is_finalized():
		return
	var est: Dictionary = bridge.bake_estimate(_bake_depth)
	var remaining := int(est.get("remaining", 0))
	_bake_button.disabled = true
	_bake_button.text = "Baking %d tile%s…" % [remaining, "" if remaining == 1 else "s"]
	## One frame so the button's own label actually paints before the
	## synchronous bake blocks the main thread -- the bake is not threaded (see
	## `bake_all`'s own doc comment in lib.rs), so this is the whole of the
	## busy state the shell can honestly offer.
	await get_tree().process_frame
	var r: Dictionary = bridge.bake_all(_bake_depth)
	_bake_button.text = "Bake ALL levels & finalize"
	if not bool(r.get("ok", false)):
		_bake_status.text = "Bake failed: %s" % String(r.get("error", "unknown"))
		_refresh_finalize()
		return
	## Finalize only after a bake that actually put something in the atlas --
	## `set_finalized(true)` refuses on an empty one anyway, and letting the
	## button claim success on a no-op would be worse than the refusal.
	bridge.set_finalized(true)
	_refresh_finalize()
	if app != null and app.has_method("refresh_atlas_status"):
		app.refresh_atlas_status()
	_bake_status.text = "Baked %d, skipped %d, in %.1fs. %s" % [
		int(r.get("baked", 0)), int(r.get("skipped", 0)), float(r.get("seconds", 0.0)),
		String(bridge.atlas_status().get("text", ""))]

## Broadcast by `app.gd`'s `_refresh_world_dependent()` when a generate or a
## save load finishes. The Finalize foot describes one specific world's atlas,
## and both the enable state and the byte estimate move when that world does.
func on_world_changed() -> void:
	_refresh_finalize()

## The tool-options bar's copy of this control presses exactly this
## (`app.gd:_tool_options_generate`). A method rather than exposing
## `_bake_button` so the header cannot press a button this workspace considers
## disabled.
func bake_and_finalize() -> void:
	if _bake_button == null or _bake_button.disabled or not bridge.has_world:
		return
	_on_bake_all()

func _refresh_finalize() -> void:
	if _bake_button == null:
		return
	var st: Dictionary = bridge.atlas_status()
	var finalized := bool(st.get("finalized", false))
	var has_world: bool = bridge.has_world
	## The reference swaps the bake button for Un-finalize rather than showing
	## both -- `applyFinalizedUI`'s own `display` toggles, line 10861-10864.
	_bake_button.visible = not finalized
	_unfinalize_button.visible = finalized
	_bake_button.disabled = not has_world
	## The tool-options bar's copy mirrors this one rather than recomputing it.
	if app != null and app.has_method("set_bake_shortcut"):
		app.set_bake_shortcut(not finalized, not has_world,
			_bake_button.tooltip_text if has_world
			else "Generate a world before baking: the atlas is keyed to one.")
	var est: Dictionary = bridge.bake_estimate(_bake_depth)
	if not has_world:
		_bake_status.text = "No world yet: generate one before baking."
	elif finalized:
		_bake_status.text = "FINALIZED. %s Generation parameters and sculpting are locked; Cartography and the 3D view stay live." % String(st.get("text", ""))
	else:
		## The byte figure leads, because it is the one that binds: a depth-3
		## bake of a 2048x1311 world at 1024 px tiles is 234 MiB (measured),
		## and depth 5 at the same settings is about 3.7 GiB. A tile count
		## alone reads as small and is not.
		_bake_status.text = "%s Baking LOD 0–%d is %d tile%s of %d×%d px — about %s on disk (%d already baked)." % [
			String(st.get("text", "")), _bake_depth, int(est.get("tiles", 0)),
			"" if int(est.get("tiles", 0)) == 1 else "s",
			int(est.get("tile_w", 0)), int(est.get("tile_h", 0)),
			String(est.get("bytes_text", "?")), int(est.get("already_baked", 0))]

## One pipeline stage's content, drawn into whichever v3 category owns it.
##
## `label_stage` puts the stage's own number and name in as an L3 section
## heading first. That is on whenever a category hosts more than one stage, or
## hosts one whose name differs from the category's -- so "Geology" reads as
## `04 TECTONICS` / `05 VOLCANISM & IMPACTS`, while "Hydrology" (which is
## stage 07 Hydrology, whole) does not repeat its own name back at itself.
##
## The numbers stay because the `needs`/`produces` prose below refers to them
## ("needs — 01 Planet, 03 World structure"): dropping the labels while keeping
## the cross-references would leave a dangling numbering scheme.
func _build_stage_body(parent: Control, index: int, label_stage: bool) -> void:
	var stage: Dictionary = STAGES[index]
	var body: Control = parent
	if label_stage:
		body = DccWidgets.section(parent, "%02d %s" % [index + 1, String(stage["name"])])

	## The mockup indents a stage's `needs`/`produces` under its title rather
	## than running them to the dock's own edge, which is what `note()` on a
	## bare body does.
	var meta := VBoxContainer.new()
	meta.add_theme_constant_override("separation", 1)
	var meta_pad := MarginContainer.new()
	meta_pad.add_theme_constant_override("margin_left", 0 if label_stage else 14)
	meta_pad.add_theme_constant_override("margin_right", 0 if label_stage else 12)
	meta_pad.add_theme_constant_override("margin_top", 0 if label_stage else 6)
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

	## A stage that already carries its own `NN NAME` heading and holds exactly
	## one block of parameters does not get a second heading naming the same
	## thing -- `03 WORLD STRUCTURE ▸ WORLD STRUCTURE` was the shape the first
	## cut of this produced. Two or more blocks still get their own headings,
	## because then the heading is telling the reader something.
	var groups: Array = stage["groups"]
	var keys: Array = stage["keys"]
	var one_block := groups.size() + (1 if not keys.is_empty() else 0) == 1
	var heading := not (label_stage and one_block)

	for group_name: String in groups:
		_build_group_section(body, group_name, index, heading)

	if not keys.is_empty():
		var host: Control = body
		if heading:
			host = DccWidgets.section(body,
				String(KEYS_SECTION_TITLES.get(String(stage["name"]), String(stage["name"]))))
		var advanced_keys: Array = []
		for key: String in keys:
			if ADVANCED_KEYS.has(key):
				advanced_keys.append(key)
			else:
				_build_param_row(host, key, index)
		if not advanced_keys.is_empty():
			var adv := DccWidgets.advanced(host)
			for key in advanced_keys:
				_build_param_row(adv, key, index)

## One params.rs `group`, in the reference's own within-panel order (the
## engine builds PARAMS in that order, and Dictionary iteration in GDScript
## preserves insertion order, so no extra sort is needed here).
func _build_group_section(parent: Control, group_name: String, stage_index: int,
		heading: bool = true) -> void:
	var sec: Control = parent
	if heading:
		sec = DccWidgets.section(parent,
			String(GROUP_TITLES.get(group_name, group_name.capitalize())))
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
		## The reference's Glacial panel carries two buttons, not one:
		## `#glacBtn` (glacialErode, still unported -- the disabled button
		## above) and `#fjordBtn` (carveFjordsOp), which is a real,
		## golden-verified port since 2026-08-23. Only the second is live.
		if pass_name == "Glacial":
			_build_fjord_row(grp)

## `#fjordBtn` / `carveFjordsOp` (reference HTML line 3245). Opt-in, exactly
## as in the reference -- it never runs during generate, so a default world
## is unchanged by this control existing.
func _build_fjord_row(grp: Control) -> void:
	DccWidgets.note(grp, "Fjord carving is ported: it overdeepens the glacially-carvable coastal valleys into drowned inlets, leaving the ridges between them high. Preview the mask first with Layers ▸ Hydrology ▸ Fjord mask. Flow, rivers and climate are not recomputed afterwards.")
	var fjord := DccWidgets.action(grp, "Carve fjords", _carve_fjords)
	fjord.tooltip_text = "The reference's #fjordBtn. Cold, steep, competent-rock coast only -- a warm or low-relief world honestly carves nothing."

func _carve_fjords() -> void:
	if not bridge.has_world:
		return
	var r: Dictionary = bridge.carve_fjords()
	if not bool(r.get("ok", false)):
		push_warning("Carve fjords: %s" % String(r.get("reason", "unavailable")))
		return
	var carved := int(r.get("cells_carved", 0))
	if carved == 0:
		push_warning("Carve fjords: %d cells are fjord-eligible, none deep enough to carve -- this world's coast is too warm, too flat or too weak." % int(r.get("cells_masked", 0)))

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
			lbl.text = "%s all ten stages resolved" % DccIcons.SYMBOLS["tick"]
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
		app.set_dock_readout("left", "resolved")

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
		DccWidgets.note(sec, "Generate a world first -- the Sculpt editor is created fresh per generated world (World ▸ Generate).")
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
	_refresh_tool_bar()
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

	## `GUI_GAP_REGISTER.md` WW-13. Gated on the **pending draft**, not on
	## `total` above -- `total` is the composite of committed and pending, so
	## it stays non-zero after a commit and left both buttons live with
	## nothing left to act on. "Discard draft" was the worse half: it then
	## read as "remove the paint I can see" and did nothing at all.
	var actions := DccWidgets.group(sec, "Commit")
	var pending := bridge.paint_draft_count()
	var commit_btn := DccWidgets.action(actions, "%s Commit" % DccIcons.SYMBOLS["tick"], _on_paint_commit, true)
	commit_btn.disabled = pending == 0
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_paint_discard)
	discard_btn.disabled = pending == 0
	if pending == 0:
		var why := "Nothing pending. Paint on the map to enable this." if total == 0 \
			else "Nothing pending -- the %d painted cells above are already committed." % total
		commit_btn.tooltip_text = why
		discard_btn.tooltip_text = why
	DccWidgets.note(sec,
		"Commit writes every layer's pending dabs into their own override arrays, refreshes " +
		"the map (the committed Biome/Terrain layers are blended into it at the reference's " +
		"own 0.60 weight, landColorCore 7898) and marks ecology/biomes and resources/soils " +
		"stale -- it never touches height, hydrology or climate. The overlay above is the " +
		"in-flight draft only: it is opaque, so it stands in for the blend until you commit. " +
		"Splat has no map colour of its own -- it forces a pack ground texture, and shows " +
		"nothing without a pack loaded.")

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
	## The same pair `_on_sculpt_commit` uses, for the same reason: since
	## 2026-08-24 `build_color_texture()` composites the committed paint
	## layers itself (`landColorCore`'s 0.60 tint), so the map raster must be
	## re-fetched -- and the opaque draft overlay must come off, or it hides
	## that blend behind the flat swatch colour it was standing in for.
	app.viewport.map_view.texture = bridge.color_texture()
	app.viewport.set_preview_texture(null)
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	_build_paint(_paint_body)
	_rebuild_tool_bar()

func _on_paint_discard() -> void:
	bridge.paint_discard()
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture())
	_build_paint(_paint_body)
	_rebuild_tool_bar()

## The other half of the WW-13 cross-refresh -- see `rebuild_paint_panel()`.
func _rebuild_tool_bar() -> void:
	if app.tool_bar != null and app.tool_bar.has_method("rebuild"):
		app.tool_bar.rebuild()

## `tool_bar.gd` draws a **second** Commit chip for the same draft, and both
## are on screen together whenever Biome paint is armed. Committing from
## either one has to refresh the other, or the loser keeps a live button over
## an empty draft -- which is WW-13's own defect wearing a different hat.
func rebuild_paint_panel() -> void:
	if _paint_body != null:
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
	_refresh_tool_bar()

## The unified tool bar (`tool_bar.gd`) shows this panel's own stamp count /
## painted count in its options row, so a stroke that ends here has to tell
## it -- otherwise the bar keeps reading "0 stamps" / "0 painted" under a
## draft that is no longer empty, and its Commit chip stays disabled. Nothing
## is duplicated: the bar re-reads the same `bridge.sculpt_stamp_count()` /
## `paint_painted_counts()` this file does.
func _refresh_tool_bar() -> void:
	var bar := DccToolBar.instance()
	if bar != null:
		bar.refresh()
