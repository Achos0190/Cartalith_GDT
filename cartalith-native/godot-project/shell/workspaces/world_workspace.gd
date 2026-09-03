extends Workspace
class_name WorldWorkspace

## WORLD domain (`DCC_SHELL_SPEC.md` §5): a two-button switch between the
## ten-stage Generation Pipeline (§5.1) and the Sculpt panel (§5.2).
##
## Every stage row reads and writes through `bridge.param_keys()` /
## `param_info()` / `param_get()` / `param_set()` -- the same live table
## `main.gd`'s old Generate menu built its per-stage dialogs from
## (`cartalith-godot/src/params.rs`, 85 parameters -- `grep -c "ParamSpec { key:"`,
## 2026-09-02; this line read 58 until 2026-09-01 and 81 until this count, so
## re-count rather than cite it).
## No range, step, label or default is copied into this file; only which stage
## a group/key belongs to, which rows are L5 Advanced, and the prose -- exactly
## the division main.gd's own GEN_STAGES comment already argued for.
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

## The only three tectonics/volcanism dials `deriveFromWorldStructure()`
## (reference HTML lines 2528-2538, ported at `generate_terrain_inner`,
## `cartalith-engine/src/lib.rs:676-684`) overrides -- verified live against
## that function's own body, not assumed from the World structure group's
## field names. Once `world_structure.enabled` is true, `tect.plates`,
## `tect.vel` and `volc.count` are ALWAYS replaced by the archetype's own
## fragmentation/tectonic_energy/hotspot_density, every Generate; this is a
## faithful port of the reference, not an engine bug, but until now nothing
## disclosed it, and `world_structure.enabled` defaults to `false`
## (`WorldParams::default()`, lib.rs:554) so most sessions never see the
## three sliders start moving and doing nothing.
##
## The rest of the tectonics/volcanism keys -- `tect.warp`, `tect.blur_r`,
## `tect.alpha`, `tect.beta`, `tect.age_inf`, `tect.ridged`, `tect.flexure`,
## `tect.hetero`, `tect.resist`, `tect.dynamic_lithology`, `tect.lloyd`,
## `volc.age`, `volc.provinces`, `crater.count`, `crater.age` -- are untouched
## by that override block and stay fully live regardless of World structure,
## so only these three are ever gated by it.
const WS_OVERRIDDEN_KEYS: Array[String] = ["tect.plates", "tect.vel", "volc.count"]

## Prefixed onto a `WS_OVERRIDDEN_KEYS` row's tooltip whenever
## `world_structure.enabled` is on, both at build time (`_build_param_row`)
## and live (`_refresh_ws_override_rows`). Names the toggle by the label its
## own row actually draws ("Enable continental steering",
## `params.rs`' `world_structure.enabled` -- World structure category, above)
## rather than its dotted key.
const WS_OVERRIDE_REASON := "World Structure is on (Enable continental steering, above) -- deriveFromWorldStructure() replaces this value from the archetype every Generate, so this dial has no effect until World Structure is turned off."

## `editable = false` stops the drag; it does not reliably say so -- this
## dock's own slider skin (`DccWidgets._style_slider`) draws the filled
## portion of the track in full accent colour whether or not the control is
## editable, since Godot's stock `Slider` theme has no separate disabled
## stylebox for `grabber_area`. `cartography_workspace.gd`'s `_mark_inert` /
## `INERT_DIM` hit the same gap first and fixed it the same way: a `modulate`
## over the whole row is the cheapest signal that reaches every child control
## at once. Reused here at the same 0.55 ratio -- `DccTheme`'s own
## `text_ghost`/`text` step against `panel` -- rather than inventing a fourth
## ink level; that helper is `cartography_workspace.gd`-private so this is a
## local copy of the ratio, not a shared call.
const WS_OVERRIDE_DIM := 0.55

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
##
## **This table is index-coupled to the engine's own
## `cartalith-engine/src/progress.rs::STAGE_NAMES`, and both sides used to
## leave that unsaid.** `bridge.generation_stage` reports a stage as an
## *index*, and `_paint_stage_rows`/`_log_stage`/`_stale_note_text` all turn
## that index into a name by reading `STAGES[i]["name"]` here -- so a stage
## inserted, removed or renamed in `progress.rs` would relabel every row
## below it with no error anywhere. `_on_generation_stage` now checks the
## engine's own `stage_name` against this row on the first tick of each
## stage (see `_assert_stage_names()`), which costs one string compare and
## turns that silent relabelling into a `push_error` naming the index.
const STAGES: Array = [
	{"name": "Planet", "needs": "—",
	 "produces": "gravity, rotation, tilt, geoid, tides → 02 Extent & scale, 08 Climate",
	 "groups": ["planet"], "keys": [],
	 ## The old note here said "geoid AND tides ... with no cartalith-engine
	 ## equivalent yet" and was only half true, which is why it is now two
	 ## sentences. Geoid: still nothing -- `params.rs` carries no geoid entry
	 ## and `cartalith-climate::geoid::refresh_geoid` (geoid.rs:135) has no
	 ## caller outside its own tests. Tides: ported and live, just not under
	 ## this stage -- `passes.tidal_flats` IS the tides enable (its own engine
	 ## doc, cartalith-engine/src/lib.rs, says "This port has no separate
	 ## enable: this toggle is it, and turning it on computes the tide
	 ## field"), so the honest thing is to name where the row actually is
	 ## rather than deny it exists.
	 "gap": "Geoid sea level is a default-off reference sub-system with no cartalith-engine equivalent yet. Tides ARE ported: there is no separate enable here because `passes.tidal_flats` is it -- turning that toggle on (06 Erosion ▸ Stream-power carve) computes the tide field, and Layers ▸ Tides previews it. The moon roster (mass, distance, k₂) is what is not exposed: `PlanetParams` carries none, so the field is built with a single Earth-Moon-equivalent companion at this world's own gravity."},
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
	 "gap": "Corrected 2026-08-30 while wiring the staged progress readout, against `generate_terrain_inner` (cartalith-engine/src/lib.rs) directly rather than trusting this note: it was stale on six of its seven claims. Stream-power carve, the Glacial group's fjord carve, Hillslope diffuse, Velocity (momentum), Glacial erosion, Coastal, Evolve climate <-> terrain (evoCyc) and Sediment fill are ALL ported and ALL run as generation-time `passes.*` toggles inside this stage's own block -- off by default, so a default world is unaffected by any of them existing (`_build_erosion_passes` below already said as much for the first four; this note had not caught up). Only Droplet hydraulic has no generate()-time equivalent: it is `erode_op`, a separate op the reference itself runs from its own `#erodeBtn`, never from `generate()` -- see the Droplet hydraulic group below."},
	{"name": "Hydrology", "needs": "06 Erosion",
	 "produces": "rivers, lakes, drainage, flow accumulation → 08 Climate, 09 Ecology & biomes",
	 "groups": [], "keys": ["carve_rivers", "river_density"],
	 "gap": "Min stream order and lakes-as-water are reference render filters, not generation parameters -- Cartography's map-mode work, not this stage."},
	{"name": "Climate", "needs": "01 Planet, 02 Extent & scale, 06 Erosion",
	 "produces": "temperature, rainfall, wind, currents → 09 Ecology & biomes, 10 Resources & soils",
	 "groups": ["climate", "weather"], "keys": [],
	 "gap": "Ported, and live -- this row said \"not ported\" until 2026-09-03 and was wrong. Köppen-Geiger is cartalith-climate/src/koppen.rs (compute_seasons, build_koppen, classify_koppen, koppen_color, compute_temp_into), golden-tested by tests/golden_parity_koppen.rs, and drawn today as Layers ▸ Climate ▸ Köppen climate. Two narrower things are true, and they are what this row means. Seasons are not computed in THIS stage: compute_seasons runs on demand when that layer is picked (sample_bridge.rs' koppen arm), which is the reference's own lazy build and costs two further temperature+weather solves, one per solstice. And no dial below exposes the classifier's own setting -- KoppenParams.max_rain_mm, the reference's state.climate.maxRainMm -- which that call site passes as a flat 3000."},
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
## The other three columns of `04-left-dock.md` §4.1's stage row -- the
## zero-padded number, the state dot and the stage name -- held so
## `_paint_stage_rows()` can recolour them per state. The spec colours all four
## elements of the row, not just the trailing label: the number and the name go
## accent/bright the moment a stage is editing, stale or running, and back to
## faint/secondary when it resolves.
var _stage_dot_labels: Array = []
var _stage_name_labels: Array = []
var _stage_number_labels: Array = []
## Per-stage wall-clock timing for the readout above, indexed the same way as
## `_stage_state_labels` and rebuilt alongside it every `_build_generate_head`
## call. `-1` means "not reached yet" (`_stage_start_msec`) / "not finished
## yet" (`_stage_elapsed_ms`); `Time.get_ticks_msec()` throughout, matching
## `last_generate_ms`'s own clock in `engine_bridge.gd`.
var _stage_start_msec: Array = []
var _stage_elapsed_ms: Array = []
## A short rolling log of "NN Name -- 0.42s" lines, newest last, shown under
## the ten rows -- the spec's own "per-stage progress + log".
var _stage_log: Array = []
const STAGE_LOG_MAX := 12
## Which stage indices `_assert_stage_names()` has already judged this run,
## so one real disagreement is reported once rather than on every tick.
## Cleared by `_reset_stage_progress()`, which is what a new run calls.
var _stage_name_checked := {}
var _stage_log_label: Label
## The earliest stage index a live-edited parameter touched since the last
## finished generate, or `-1` when the world is not stale. Cleared on
## `generation_finished`, not on `generation_started`: the badge stays up
## for the run it caused, then clears once that run has made the world match
## the dials again. This engine has no partial recompute (`_regenerate_live`'s
## own doc comment, verified live against the reference), so the note is
## informational -- "here is where the edit that triggered this run landed"
## -- not a claim that Generate skips stages before it.
var _stale_from_stage := -1
var _stale_note_label: Label

## §5.1's Finalize foot (`GUI_GAP_REGISTER.md` WW-01). `_bake_depth` defaults
## to 3 -- the reference's own `bakeAllDepth` default, and 85 tiles, which is
## the deepest bake that finishes in a plausible interactive wait.
##
## **Since 2026-08-30 it is a mirror of `DccSettings.bake_depth()`, not a
## private field.** §2.5 lists "LOD levels 0-8" in Preferences ▸ Tiles & LOD,
## and this dock foot is the only place the number was settable; a Preferences
## ladder over a private field would have been a second copy free to disagree
## with what `bake_all()` is actually called with. Both surfaces write the one
## key now, and this reads it back in `_refresh_finalize()` so a change made in
## the menu is visible here without a rebuild.
var _bake_depth := 3
var _bake_depth_choice: OptionButton
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

## Every `STAGES` group name, checked once against the engine's own
## `get_param_groups()` before the dock is built.
##
## `_build_group_section()` and `_build_erosion_passes()` both find their rows
## by filtering `param_keys()` on `info["group"] == <a name hardcoded above>`.
## A filter that matches nothing is not an error in GDScript -- it is an empty
## loop -- so renaming a group in `params.rs` would leave that stage section
## rendering as a heading with no rows under it, silently, on every world.
## That is exactly the silent-degradation shape `audit_wiring.py`'s question C
## exists to catch, and it is cheap to catch here instead: one pass over a
## ten-row table at first paint.
##
## `push_error` rather than `assert()`: `assert` is stripped from a release
## build, and a stale name is precisely the kind of drift that survives to a
## release build unnoticed. It names the offending group and what the engine
## does offer, so the fix is the next line of the message rather than a hunt.
##
## Silent when the probe itself is unavailable: `param_groups()` answers with
## an empty `PackedStringArray` on an older GDExtension with no
## `get_param_groups()` (`EngineBridge._has` has already warned about that
## once by then), and treating "I could not ask" as "every group is missing"
## would fire ten false errors for one real cause.
func _assert_stage_groups() -> void:
	var known := bridge.param_groups()
	if known.is_empty():
		return
	for stage: Dictionary in STAGES:
		for group_name: String in stage["groups"]:
			if known.has(group_name):
				continue
			push_error(
				"Cartalith: stage \"%s\" filters params on group \"%s\", which params.rs no longer defines. "
				% [String(stage["name"]), group_name]
				+ "That section would have rendered EMPTY with no other symptom. "
				+ "The engine's own groups are: %s." % ", ".join(known))

func _build() -> void:
	## Before anything reads a group name: see `_assert_stage_groups()` for
	## why a hardcoded group that params.rs dropped is a silent failure.
	_assert_stage_groups()

	## Every left dock opens with the TOOLS block, the four global tools then
	## the domain's own (`04-left-dock.md` §2.4). Its own WORLD row is three
	## pills -- `Sculpt` (no key), `Freehand` **F**, `Biome paint` **B** -- and
	## this stage (GUI replacement stage 4) re-examined rather than inherited
	## that gap: only Biome paint is a TOOLS-block button here, deliberately.
	##
	## `Sculpt` and `Freehand` both arm the one shared "sculpt" tool id, and
	## the *only* place that id's granularity is chosen -- which of the 13
	## `get_sculpt_features()` entries, Freehand's 13th among them -- is the
	## feature-picker grid below (`_build_feature_picker`), which already
	## shares `app.tool_group` with everything else this dock arms. A second,
	## coarser "Sculpt" pill in the same `ButtonGroup` would either duplicate
	## that grid's own pressed-state bookkeeping (`set_pressed_no_signal`,
	## chosen there specifically to avoid re-firing `toggled` and resetting
	## the live feature's parameters) or drift from it -- correct sculpting
	## either way, since the tool id and its parameters are unaffected by
	## which button drew the click, but a second indicator of the same state
	## that can show a coarser answer than the fine-grained one sitting next
	## to it is the kind of two-state-computations bug this project has paid
	## for before (the bake button, the recompute rows). Reachability through
	## the existing grid costs one extra click (open Terrain) beyond what a
	## TOOLS-block pill would; `F` closes the one real gap that click cost
	## has -- see `_build_feature_picker`'s own Freehand `Shortcut` below.
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

	bridge.generation_started.connect(_reset_stage_progress)
	bridge.generation_stage.connect(_on_generation_stage)
	bridge.generation_finished.connect(_on_generation_finished)
	bridge.world_loaded.connect(_on_world_loaded)
	_paint_stage_rows()

## A new/loaded world means a fresh (or absent) `SculptEditor`/`PaintEditor`
## on the Rust side -- both panels rebuild from scratch rather than trusting
## whatever they showed for the previous world.
func _on_generation_finished(ok: bool) -> void:
	if ok:
		## Close out whichever stage was still "running" when the signal
		## landed -- `_on_generation_stage` only closes a stage out once a
		## LATER one arrives, so the run's own last stage needs closing here.
		var now := Time.get_ticks_msec()
		for i in _stage_elapsed_ms.size():
			if _stage_elapsed_ms[i] < 0 and _stage_start_msec[i] >= 0:
				_stage_elapsed_ms[i] = now - _stage_start_msec[i]
				_log_stage(i)
	## The world this generate produced now matches every dial -- the stale
	## badge from whichever edit caused this run clears.
	_stale_from_stage = -1
	if _stale_note_label != null:
		_stale_note_label.text = _stale_note_text()
	_paint_stage_rows()
	## Covers the OTHER way `world_structure.enabled` reaches true besides a
	## live click on its own row: File ▸ New world applying a World-Structure
	## archetype preset sets the params THEN generates, so this signal is the
	## first point back in this workspace where the three overridden rows can
	## be re-gated against what a preset just changed -- `_on_bool_row_changed`
	## alone never fires for that path. Cheap even when nothing changed: three
	## property writes per row, no rebuild, and `world_structure.enabled` is
	## re-read live rather than trusted from any argument here.
	_refresh_ws_override_rows()
	_build_sculpt(_sculpt_body)
	_build_paint(_paint_body)
	_fill_ecology(_ecology_body)
	_build_crs(_crs_body)

func _on_world_loaded() -> void:
	## A load is not a generate: it never goes through `bridge.generation_stage`
	## at all, so any timing left over from a previous run is stale and the
	## readout should show a plain "resolved" for every row.
	_reset_stage_progress()
	## A loaded save can carry `world_structure.enabled == true` too --
	## `EngineBridge.load_save`'s own comment: "The dials moved to whatever
	## the save carried, so anything reading param_get has to re-read them."
	## Re-gate the three overridden rows against it for the same reason
	## `_on_generation_finished` does. Every OTHER parameter row in this panel
	## does not resync its displayed value against a loaded save at all -- a
	## real, separate gap this call does not attempt to close; see this file's
	## `_ws_override_sliders` doc for why only these three are handled here.
	_refresh_ws_override_rows()
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
## (navigability, discharge, catchment, tributaries). Three of those four are
## real readings now -- `get_rivers(min_order)` carries `discharge`,
## `catchment_km2` and `tributaries` per traced run -- and navigability is the
## one v3 asks for that nothing computes per river. CIVIL's old Rivers category
## was the only place in the shell that disclosed the state of this subject; it
## was retired in the same pass that moved the subject here, so the finding has
## to be re-drawn here or it is simply gone. `rivers_note()` is its single
## owner (`GUI_GAP_REGISTER.md` IN-01).
func _build_hydrology_foot(parent: Control) -> void:
	## Deliberately NOT "River network" -- `KEYS_SECTION_TITLES` already gives
	## the stage's own carve/density dials that heading, and two sections with
	## one name in one category is how a reader ends up reading the wrong one.
	## Not "Not built" any more: `get_rivers()`/`river_at()` landed, so a heading
	## that files the whole subject as absent now disagrees with the first
	## sentence of the note under it.
	DccWidgets.note(DccWidgets.section(parent, "River entities"),
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
	## `bridge.generation_stage` (`engine_bridge.gd`, 2026-08-30) made this a
	## REAL per-stage readout rather than the single collapsed "generating…"
	## label this section used to draw -- see that signal's own doc comment
	## and `cartalith-engine/src/progress.rs` for how each stage's own bump
	## point was chosen. `_stale_note_label` is built first so it sits above
	## the ten rows, matching the spec's own "stale-from note, then staged
	## progress" order.
	_stale_note_label = DccWidgets.note(status, _stale_note_text())
	_stage_state_labels = []
	_stage_start_msec.clear()
	_stage_elapsed_ms.clear()
	## `04-left-dock.md` §4.1's stage header row, four elements wide: an 18px
	## zero-padded number, the state dot, the name, then the state label. Built
	## as four Labels rather than one string, because the spec colours each of
	## them independently per state and one string cannot carry three colours
	## (see `_paint_stage_rows()` for the table).
	_stage_dot_labels = []
	_stage_name_labels = []
	_stage_number_labels = []
	for i in STAGES.size():
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		status.add_child(row)
		var number_label := DccTheme.mono_label("%02d" % (i + 1),
			"text_faint", DccTheme.FS_MICRO, 1)
		number_label.custom_minimum_size.x = 18
		row.add_child(number_label)
		var dot_label := DccTheme.mono_label(DccIcons.SYMBOLS["off"],
			"text_ghost", DccTheme.FS_MICRO, 0)
		dot_label.custom_minimum_size.x = 9
		row.add_child(dot_label)
		var name_label := DccTheme.mono_label(String(STAGES[i]["name"]),
			"text_secondary", DccTheme.FS_MICRO, 1)
		name_label.custom_minimum_size.x = DccWidgets.ROW_LABEL_W - 33
		row.add_child(name_label)
		var state_label := DccTheme.mono_label("pending", "text_ghost", DccTheme.FS_MICRO, 1)
		state_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		## DS-03. Most of this column is two words -- "pending", "no world",
		## "running...". Two rows are not: stages 9 and 10 append
		## `_paint_stage_rows()`'s gap note, and the finished string is
		## "done  3.83s  (no engine work this run -- see gap note above)". A
		## `Label`'s minimum width is its whole text unless it wraps, measured
		## 545 px here, and this one is `SIZE_EXPAND_FILL` in a row that
		## already spends 18 + 9 + `ROW_LABEL_W - 33` px on its three fixed
		## siblings -- so with a world generated the left dock was forced from
		## 400 px to **783** on tablet, taking that width off the map. It only
		## appears after a generate, which is why the boot-state sweep in
		## `_ds03fit_probe.gd` misses it and the world sweep does not.
		state_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		row.add_child(state_label)
		_stage_number_labels.append(number_label)
		_stage_dot_labels.append(dot_label)
		_stage_name_labels.append(name_label)
		_stage_state_labels.append(state_label)
		_stage_start_msec.append(-1)
		_stage_elapsed_ms.append(-1)
	_stage_log_label = DccWidgets.note(status, "")
	DccWidgets.note(status,
		"Real per-stage progress (`GenerationProgress`, `engine_bridge.gd`), not "
		+ "a simulated animation -- but still ONE generate() that resolves all "
		+ "ten stages every call: this engine has no partial recompute, so every "
		+ "row above runs in full on every Generate, whichever stage an edit came "
		+ "from (`Stale from NN` above names where the edit landed, not where the "
		+ "run starts). What CAN go stale independently is the civilisation layer "
		+ "over an edited world, and that has its own badge and its own button: "
		+ "Civilization ▸ Settlements ▸ Recompute.")
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
				## `DccUnits.format_area`, not a raw `km²` through `_thousands`:
				## an ecoregion's area is a map distance squared and converts
				## with the rest. Corrected 2026-09-03 -- `format_area`'s own
				## doc names this exact shape as the half-fix this project
				## treats as a defect, and a verifier found this readout still
				## printing `km²` in a file the same pass had just edited.
				"%s — %s, %d species, NPP %d" % [
					String(r.get("biome_name", "?")),
					DccUnits.format_area(float(r.get("area_km2", 0.0))),
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
			("%d × %d cells over %s × %s, so one cell is %s on a side. "
			+ "Rows run %.1f° to %.1f° — %.4f° of latitude per row, which is what "
			+ "the climate model integrates over.")
			% [int(crs.get("grid_w", 0)), int(crs.get("grid_h", 0)),
				DccUnits.format(float(crs.get("map_width_km", 0.0))),
				DccUnits.format(float(crs.get("map_height_km", 0.0))),
				DccUnits.format(float(crs.get("cell_km", 0.0)), 3),
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
	##
	## **The trailing units sentence is gone, not merely reworded (2026-09-03).**
	## The row this note used to close on -- "Units are km-only; the reference's
	## km/mi toggle is not ported (PR-15)" -- named a real PR-15 (`DccUnits`,
	## `GUI_GAP_REGISTER.md` §7.8, "Build, and add nautical miles"), but that
	## ticket is about the km/mi/nm *display formatter*, not about reprojection,
	## and confusingly sat inside this paragraph's own different subject. It is
	## also simply false now: `DccUnits` shipped (real as of 2026-09-02,
	## `menus.gd`'s Preferences ▸ Units), is already the formatter behind
	## `right_dock.gd`'s measure readouts and `viewport_host.gd`'s scale bar,
	## and the figures directly above now go through it too -- so this panel is
	## no longer km-only either. Reprojection is the one part of the old
	## sentence that was never PR-15's: nothing here claims a map projection,
	## and that is still true and still unresolved, which is the whole of what
	## this note is about below.
	DccWidgets.note(sec,
		"Reprojection  ·  needs a decision\n"
		+ "Every field is grid space and nothing reprojects, so the planar "
		+ "kilometres are not a projection of the latitudes beside them, and a GIS "
		+ "reading them as WGS84 degrees is misreading the file. Which projection "
		+ "a fictional world should claim is an authoring decision, not a defect "
		+ "(GUI_GAP_REGISTER.md WW-15).\n"
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
## Tiles in a pyramid of `depth` levels: (4^(depth+1) - 1) / 3, thousands
## separated. Written out rather than read from `bake_estimate()` because this
## builds NINE labels at once and `bake_estimate` opens the atlas directory --
## the arithmetic is exact and the estimate's other fields (bytes, seconds) are
## world-dependent, so those stay on the status line where a world exists.
func _pyramid_tiles(depth: int) -> String:
	var n := 0
	for z in range(0, depth + 1):
		n += (1 << z) * (1 << z)
	var out := ""
	var s := str(n)
	for i in s.length():
		if i > 0 and (s.length() - i) % 3 == 0:
			out += " "
		out += s[i]
	return out

func _build_finalize(parent: Control) -> void:
	var foot := DccWidgets.section(parent, "Finalize")
	_bake_status = DccWidgets.note(foot, "")

	## §2.5's full 0-8 ladder, not the four rungs this used to offer. The cost
	## is on the label rather than behind it, because depth 8 is 87 381 tiles
	## and a synchronous bake that deep is a decision, not a click -- the same
	## reasoning that already put the tile count on the row below.
	_bake_depth = DccSettings.bake_depth()
	var depth_labels: Array[String] = []
	for d in range(0, 9):
		depth_labels.append("LOD 0–%d   %s tile%s" % [
			d, _pyramid_tiles(d), "" if d == 0 else "s"])
	_bake_depth_choice = DccWidgets.choice(foot, "Bake depth", depth_labels,
		clampi(_bake_depth, 0, 8),
		func(i: int):
			_bake_depth = i
			DccSettings.set_bake_depth(i)
			_refresh_finalize(),
		"How deep the pyramid is baked. Level z holds 2^z x 2^z tiles, so the total is (4^(depth+1)-1)/3. Already-baked chunks are skipped, so raising the depth later only fills the gaps. The same setting lives in Preferences > Tiles & LOD > LOD levels -- one store, two entry points.")

	_bake_button = DccWidgets.action(foot, "Bake ALL levels & finalize", _on_bake_all, true)
	## **The read side is not wired, and this tooltip no longer says it is**
	## (2026-09-01). It promised "deep zoom then reads bytes instead of
	## re-synthesising octaves", which no draw path performs:
	## `viewport_host.gd`'s `_build_lod_tile()` opens with an unconditional
	## `_bridge.lod_synthesize_tile()` and has no atlas branch, and
	## `atlas_tile_png()` -- the reader -- is wrapped in `engine_bridge.gd`
	## and called by no shell file. `menus.gd`'s `_build_atlas_cache_menu`
	## header carries why that is not a one-line branch (a baked chunk is a
	## stored picture; a drawn tile is a shade ratio the LOD shader
	## multiplies in). What the bake really buys -- a persistent store, the
	## skip, and the finalize lock -- is what this now says instead.
	_bake_button.tooltip_text = "Pre-render every tile of the pyramid to the on-disk atlas, then lock the world. The store persists across sessions and already-baked chunks are skipped, so a later re-bake or a deeper one only fills the gaps. It does NOT speed up panning: nothing reads the atlas at draw time yet, so the deep-zoom layer still synthesises every tile it draws. This blocks the UI while it runs -- see the size and tile count above before committing."
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
	## Re-read the shared store: `Preferences > Tiles & LOD > LOD levels` writes
	## the same key, and this dock is refreshed on every atlas change, so a
	## change made in the menu shows up here without either surface knowing
	## about the other.
	var stored := DccSettings.bake_depth()
	if stored != _bake_depth:
		_bake_depth = stored
		if _bake_depth_choice != null:
			_bake_depth_choice.selected = clampi(stored, 0, 8)
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
		_bake_status.text = "FINALIZED. %s Generation parameters and sculpting are locked; Cartography stays live." % String(st.get("text", ""))
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
## glacial, coastal -- each its own group with its own run button".
##
## The old note on the five non-stream-power groups ("Not ported -- a separate
## manual pass in the reference with no cartalith-engine equivalent") was
## false on all five, which `PARITY_AUDIT.md` §23 F11 caught for the droplet
## pass and which is just as wrong for the other four:
## `cartalith-erosion::passes` carries `hillslope_diffuse`,
## `velocity_erode_kernel`, `glacial_kernel` and `coastal_process`, and
## `cartalith_engine::ErosionPassParams` exposes every one of them as a
## `passes.*` parameter (DECISIONS.md §7d: the same kernels run at the END of
## generation rather than as buttons). Those rows are already on screen --
## they are in the "Stream-power carve" group above, because `params.rs` files
## them all under `group: "erosion"`. So the honest note is "run as a
## generation toggle above", not "not ported".
func _build_erosion_passes(body: VBoxContainer, stage_index: int) -> void:
	var real := DccWidgets.group(body, "Stream-power carve", true)
	for key in bridge.param_keys():
		var info := bridge.param_info(key)
		if String(info.get("group", "")) == "erosion":
			_build_param_row(real, key, stage_index)

	## Droplet is the one that is genuinely a BUTTON in this port too -- the
	## reference runs it from `#erodeBtn` over the finished field and
	## `generate()` never touches it, so it has no `passes.*` toggle and needs
	## its own control. §23 F11.
	_build_droplet_erosion(DccWidgets.group(body, "Droplet hydraulic", false))

	for pass_name in ["Hillslope diffuse", "Velocity (momentum)", "Glacial", "Coastal"]:
		var grp := DccWidgets.group(body, pass_name, false)
		DccWidgets.note(grp,
			"Ported. This port runs it as a generation-time toggle rather than a button " +
			"(DECISIONS.md §7d) -- its passes.* switch and dials are in Stream-power carve above, " +
			"off by default. There is no separate run button because the pass is part of generate().")
		## The reference's Glacial panel carries two buttons, not one:
		## `#glacBtn` (glacialErode, which this port runs as `passes.glacial`)
		## and `#fjordBtn` (carveFjordsOp), a real, golden-verified port since
		## 2026-08-23 and the one true opt-in button of the four.
		if pass_name == "Glacial":
			_build_fjord_row(grp)

# -- §23 F11 · the reference's `erode()` op --------------------------------------
#
# `#erodeBtn` -> `erode()` (reference HTML line 3898): `dropletKernel` ->
# `erodeFinish` (3892) -> `erodeThermal` -> clamp to [0,1] -> `isostaticRebound`
# -> `computeFlow(true)` + `refreshClimate()`. All four kernels are ported in
# `cartalith-erosion` with golden-parity coverage; nothing ever assembled them.
#
# The assembly lives in `cartalith_engine::erode_op` rather than in the bridge:
# `cartalith-godot` does not depend on `cartalith-erosion` and the engine
# re-exports none of it, so `erode_bridge.rs` cannot name `droplet_kernel` or
# `erode_thermal` at all. `WorldGen::erode_op(opts)` is the thin caller.
#
# The `bridge._has("erode_op")` guard stays, for the same reason every other
# `_has` guard in this shell does: a shipped `.gd` can meet an older native
# library, and F14 records what a silently-missing binding costs. It resolves
# true against this build.

## The shell's transcription of `ErodeOpts::default()`
## (`cartalith-engine/src/erode_op.rs`), which is what actually runs:
## `erode_opts_from` in `erode_bridge.rs` fills every key this dictionary
## omits from it, so a number that drifted from the engine's would be a lie
## the sliders told and the op ignored. It has to be transcribed. `ErodeOpts`
## is a separate struct from `WorldParams` -- droplet erosion is an op over
## the finished field, not a generation stage -- so none of these five keys is
## a row in `cartalith-godot/src/params.rs`' `PARAMS` table, and the engine
## exposes no getter for the struct either. `_erode_defaults()` still asks
## `param_default(key)` first, so the day a key does become a table row the
## engine wins without this table moving; today it answers `null` for all
## five and these literals stand.
##
## The five keys are the ones the reference's own Erosion panel exposes
## (`#drops`/`#estr`/`#edep`/`#ethr`/`#etal`, bound by `eparam()` at reference
## lines 12922-12926), and the literals are `state.erosion`'s (reference HTML
## line 2268). The other nine `dropletParams()` fields -- inertia, capacity,
## minSlope, evaporate, gravity, maxLifetime, initSpeed, initWater, radius --
## have no reference control and are never named here at all.
const ERODE_DEFAULTS := {
	"droplets": 60000,        ## #drops, slider 0..100 x1500, default 40
	"erode": 0.35,            ## #estr,  slider 0..100 /100,  default 35
	"deposit": 0.30,          ## #edep,  slider 0..100 /100,  default 30
	"thermal_passes": 8,      ## #ethr,  slider 0..30,        default 8
	"talus": 0.012,           ## #etal,  slider 1..40 /1000,  default 12
}

## Live op parameters. NOT world parameters: `erode()` is an op over the
## finished field, `generate()` never runs it, and nothing here reaches
## `WorldParams` or any generation-derived hash.
##
## Seeded from `ERODE_DEFAULTS` directly, because a member initialiser runs
## at instantiation -- before `setup()` has assigned a `bridge`.
## `_build_droplet_erosion` re-seeds it from `_erode_defaults()` on the first
## build, which is the earliest moment `param_default()` can be consulted at
## all; today that returns the same five literals.
var _erode_op: Dictionary = ERODE_DEFAULTS.duplicate()
var _erode_defaults_cache: Dictionary = {}
## True once `_erode_defaults()` has found at least one of the five keys in
## the engine's parameter table. Set by that call, read only by the Reset
## button's tooltip, so the tooltip states the source it actually got.
var _erode_defaults_from_engine := false
var _erode_op_seeded := false

## The five exposed keys' defaults: `param_default(key)` where the engine's
## parameter table carries the key, `ERODE_DEFAULTS` where it does not.
## Today that is all five -- see `ERODE_DEFAULTS` for why -- so this resolves
## to the transcription; the lookup stays because it is the one accessor that
## would answer if an `ErodeOpts` key were ever added to `PARAMS`, and because
## `_erode_defaults_from_engine` reports which source was used rather than
## letting the Reset tooltip guess.
##
## Cached: the answer is static for the session (a Rust `Default` impl, not
## world state), and this is read once per panel build plus once per Reset.
func _erode_defaults() -> Dictionary:
	if _erode_defaults_cache.is_empty():
		var d: Dictionary = ERODE_DEFAULTS.duplicate()
		for k in d:
			var v = bridge.param_default(String(k))
			if v != null:
				d[k] = v
				_erode_defaults_from_engine = true
		_erode_defaults_cache = d
	return _erode_defaults_cache

func _build_droplet_erosion(grp: Control) -> void:
	var live := bridge._has("erode_op")
	## First build only. Later builds must NOT re-seed: this panel is rebuilt
	## wholesale after every generate, and overwriting `_erode_op` there would
	## silently discard whatever the person had dialled in.
	if not _erode_op_seeded:
		_erode_op_seeded = true
		_erode_op = _erode_defaults().duplicate()
	DccWidgets.note(grp,
		"The reference's #erodeBtn. Particle hydraulic erosion over the finished surface: " +
		"droplets follow the inertia-blended gradient, erode or deposit against carrying " +
		"capacity, then thermal talus relaxation, a clamp to [0,1] and isostatic rebound of " +
		"the unloaded crust. Opt-in, exactly as in the reference -- it never runs during " +
		"generate, so a default world is unchanged by this control existing. Flow and climate " +
		"are recomputed afterwards.")

	## The reference's own five sliders, in its own panel order, at its own
	## ranges -- derived from the `<input type=range>` bounds times the
	## `eparam()` mapping (reference lines 1094-1098 and 12922-12926), not
	## invented here. `is_int` per row so `droplets`/`thermal_passes` reach the
	## op as ints, the same split `_build_param_row` already makes.
	var rows: Array = [
		["droplets", "Droplets", 0.0, 150000.0, 1500.0, true,
			"Reference control #drops (slider 0-100, x1500)."],
		["erode", "Strength", 0.0, 1.0, 0.01, false,
			"Reference control #estr. How hard a droplet cuts when it is under capacity."],
		["deposit", "Deposition", 0.0, 1.0, 0.01, false,
			"Reference control #edep. How much sediment drops when a droplet is over capacity."],
		["thermal_passes", "Thermal", 0.0, 30.0, 1.0, true,
			"Reference control #ethr. Talus relaxation passes run after the droplets."],
		["talus", "Slope limit", 0.001, 0.040, 0.001, false,
			"Reference control #etal. The angle of repose the thermal passes relax toward."],
	]
	var sliders: Array = []
	for r: Array in rows:
		var made := DccWidgets.slider(grp, String(r[1]), float(r[2]), float(r[3]), float(r[4]),
			float(_erode_op[String(r[0])]), "", _on_erode_param.bind(String(r[0]), bool(r[5])),
			String(r[6]))
		sliders.append(made["slider"])

	var btn := DccWidgets.action(grp, "Erode (droplet)", _run_erode, true)
	btn.disabled = not live
	btn.tooltip_text = ("The reference's #erodeBtn. Runs over the whole map and pushes one " +
		"undo step; 60k droplets is not instant at 2048².") if live else \
		"This build's GDExtension has no WorldGen.erode_op()."

	## Writing `HSlider.value` re-emits `value_changed`, which is what updates
	## both the readout and `_erode_op` -- so the dictionary is restored by the
	## same path a drag uses, not by a second assignment that could disagree
	## with what is drawn.
	var reset := DccWidgets.action(grp, "Reset dials", func():
		var d := _erode_defaults()
		for i in rows.size():
			(sliders[i] as HSlider).value = float(d[String((rows[i] as Array)[0])]))
	if _erode_defaults_from_engine:
		reset.tooltip_text = ("Back to ErodeOpts::default() -- read from the engine's own "
			+ "parameter table.")
	else:
		reset.tooltip_text = ("Back to ErodeOpts::default(), which is state.erosion's own "
			+ "defaults (reference HTML line 2268). ErodeOpts is not part of the engine's "
			+ "parameter table -- droplet erosion is an op over the finished field, not a "
			+ "generation stage -- so these are the shell's transcribed copies.")

func _on_erode_param(v: float, key: String, is_int: bool) -> void:
	_erode_op[key] = int(round(v)) if is_int else v

func _run_erode() -> void:
	if not bridge.has_world or not bridge._has("erode_op"):
		return
	var r: Dictionary = bridge.world_gen.erode_op(_erode_op)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Erode: %s" % String(r.get("reason", "unavailable")), "accent")
		return
	## `build_color_texture()` reads the live field fresh on every call, so
	## writing `map_view.texture` directly is enough -- the same reason
	## `_on_sculpt_commit` does it this way rather than calling
	## `ViewportHost.refresh()`, which would also reset the camera to fit.
	app.viewport.map_view.texture = bridge.color_texture()
	var cells := int(r.get("cells_changed", 0))
	## `climate_coupled` false means this world carried no rainfall and the
	## droplets spawned uniformly instead of through the rain field. A real
	## outcome worth naming, not an error -- the erosion pattern differs.
	var coupled := "" if bool(r.get("climate_coupled", false)) \
		else " (no rainfall on this world -- droplets spawned uniformly)"
	app.set_status("hint", "Eroded %d cells, %d lowered, in %.0f ms.%s"
		% [cells, int(r.get("cells_lowered", 0)), float(r.get("ms", 0.0)), coupled],
		"text_ghost")

## `#fjordBtn` / `carveFjordsOp` (reference HTML line 3245). Opt-in, exactly
## as in the reference -- it never runs during generate, so a default world
## is unchanged by this control existing.
func _build_fjord_row(grp: Control) -> void:
	## The old wording here ("Flow, rivers and climate are not recomputed
	## afterwards") was wrong on two of the three: `carve_fjords` marks
	## `PipelineStage::Height` and runs the staleness graph, which is
	## `computeFlow(true)` + `refreshClimate()`. Only the vector river network
	## is left as it was -- `carve_fjords`' own Rust doc comment says exactly
	## this under "What it re-runs, and what it does not".
	DccWidgets.note(grp, "Fjord carving is ported: it overdeepens the glacially-carvable coastal valleys into drowned inlets, leaving the ridges between them high. Preview the mask first with Layers ▸ Hydrology ▸ Fjord mask. Flow and climate are recomputed afterwards; the vector river network is not.")
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

## `WS_OVERRIDDEN_KEYS` -> its live `HSlider`, so `_refresh_ws_override_rows`
## can re-gate editable/tooltip/modulate the moment `world_structure.enabled`
## itself flips, without a full panel rebuild -- the same bound-Control
## pattern `_sculpt_commit_btn` etc. use. Populated once, by `_build_param_row`
## during the ONE `_build_categories()` pass `setup()` runs: unlike the
## Sculpt/Paint/Ecology/CRS panels below, this dock's parameter rows are never
## rebuilt after the first `_build()` -- a live drag or toggle writes straight
## through `bridge.param_set`, so there is nothing here for a generate to
## resync.
var _ws_override_sliders: Dictionary = {}
## The same three keys -> their tooltip text with `WS_OVERRIDE_REASON` left
## out -- what the row reads while World Structure is off. Kept alongside the
## slider so the live re-gate needs no second call to `param_info` /
## `param_default` to rebuild it.
var _ws_override_base_hint: Dictionary = {}

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

	## Per-row "back to the engine's default", off `param_default(key)` --
	## the accessor `EngineBridge` has filled since `_read_param_table()` was
	## written and which, until now, nothing read. Named in the tooltip
	## because a right-click that is not advertised is not a feature; the
	## default itself is printed there too, so the row answers "what would
	## this go back to?" without being clicked.
	var reset_to = bridge.param_default(key)
	if reset_to != null:
		hint += " Right-click the row to reset to the engine's default (%s)." % str(reset_to)

	## Tectonics: World-Structure archetype override -- see `WS_OVERRIDDEN_KEYS`
	## for what this is and why. `_ws_override_base_hint` is recorded whether
	## or not the toggle is on right now: `_refresh_ws_override_rows` needs the
	## "off" text back the moment World Structure flips off again.
	var ws_overridden := false
	if WS_OVERRIDDEN_KEYS.has(key):
		_ws_override_base_hint[key] = hint
		ws_overridden = bool(bridge.param_get("world_structure.enabled"))
		if ws_overridden:
			hint = "%s %s" % [WS_OVERRIDE_REASON, hint]

	if kind == "bool":
		## A checkbox toggle is atomic -- there is no "dragging" phase to defer
		## past -- so it regenerates immediately, matching the reference's own
		## `<input type=checkbox>` `change` handlers (fired on click, not on a
		## release distinct from a press).
		var cb := DccWidgets.toggle(parent, label, bool(bridge.param_get(key)),
			_on_bool_row_changed.bind(key, stage_index), hint)
		if reset_to != null:
			## Writing `button_pressed` re-emits `toggled`, so the reset lands
			## through `_on_bool_row_changed` -- the same one path a click uses.
			## Guarded on "already there" so a right-click on an untouched row
			## is a no-op rather than a full regenerate for no change.
			_wire_row_reset(cb, func():
				if cb.button_pressed != bool(reset_to):
					cb.button_pressed = bool(reset_to))
		return

	var is_int := kind == "int"
	## `tparam()`'s exact split: `input` (every drag tick) only updates the
	## value; `change` (release) applies it and regenerates. `DccWidgets.slider`
	## already gives the continuous half via `on_change` -- `on_release` is new,
	## wired to `HSlider.drag_ended`, which is Godot's one-shot release signal.
	var made := DccWidgets.slider(parent, label, float(info.get("min", 0.0)), float(info.get("max", 1.0)),
		float(info.get("step", 0.01)), float(bridge.param_get(key)), unit,
		_on_float_row_input.bind(key, is_int), hint,
		_on_float_row_released.bind(key, is_int, stage_index))
	var s := made["slider"] as HSlider
	if WS_OVERRIDDEN_KEYS.has(key):
		## Held for `_refresh_ws_override_rows`; see that dictionary's own doc.
		_ws_override_sliders[key] = s
		if ws_overridden:
			s.editable = false
			## The WHOLE row, not just the slider -- label and readout dim
			## too, matching `_mark_inert`'s own reasoning (this block's doc
			## comment on `WS_OVERRIDE_DIM`): a dimmed slider next to a
			## full-brightness number and name would still read as live.
			(made["row"] as Control).modulate = Color(1.0, 1.0, 1.0, WS_OVERRIDE_DIM)
		## `_row()` sets `tooltip_text` only on the row `HBoxContainer` it
		## returns; the slider is its own `Control` with the default
		## `MOUSE_FILTER_STOP`, so a hover landing on the grip/track itself
		## shows nothing unless the slider carries its own copy --
		## `_dead_slider`'s own precedent (`cartography_workspace.gd`).
		s.tooltip_text = hint
	if reset_to != null:
		## Writing `HSlider.value` re-emits `value_changed`, which is what
		## repaints the readout AND calls `_on_float_row_input` -- so only the
		## release half has to be fired by hand here. `drag_ended` never fires
		## for a programmatic write, which is exactly why `_build_droplet_
		## erosion`'s own Reset gets away without it: that one writes no engine
		## parameter and needs no regenerate.
		##
		## `if not s.editable: return` makes the reset a no-op on a
		## World-Structure-overridden row exactly like the drag it stands in
		## for -- left generic rather than special-cased to
		## `WS_OVERRIDDEN_KEYS`, because `s.editable` is live: it already
		## reflects whatever `_refresh_ws_override_rows` last set, so this
		## closure needs no second copy of that state to go stale against.
		_wire_row_reset(s, func():
			if not s.editable:
				return
			if is_equal_approx(s.value, float(reset_to)):
				return
			s.value = float(reset_to)
			_on_float_row_released(key, is_int, stage_index))

## Right-click on a parameter row -> `revert`. Connected to the CONTROL rather
## than to the row `HBoxContainer`: `HSlider` and `CheckBox` both default to
## `MOUSE_FILTER_STOP`, so an event over them stops there whether or not they
## accepted it, and a handler on the row alone would be dead over the half of
## the row a person actually aims at. The row is wired as well so the label
## side works too.
##
## `MOUSE_BUTTON_RIGHT` and not a context `PopupMenu`: every other reset in
## this dock is a single act with a single outcome, and a one-item popup to
## reach it would be the only such menu in the workspace.
func _wire_row_reset(control: Control, revert: Callable) -> void:
	var on_input := func(event: InputEvent):
		var mb := event as InputEventMouseButton
		if mb != null and mb.pressed and mb.button_index == MOUSE_BUTTON_RIGHT:
			revert.call()
	control.gui_input.connect(on_input)
	var row := control.get_parent() as Control
	if row != null:
		row.gui_input.connect(on_input)

func _on_bool_row_changed(v: bool, key: String, stage_index: int) -> void:
	bridge.param_set(key, v)
	if key == "world_structure.enabled":
		_refresh_ws_override_rows()
	_mark_stale_from(stage_index)
	_regenerate_live()

## Live counterpart to `_build_param_row`'s own gate, re-applied to whichever
## `WS_OVERRIDDEN_KEYS` rows survived from the one `_build_categories()` pass
## -- three property writes per row, no rebuild, the same "cheap enough to
## run on every change" discipline `_refresh_sculpt_draft` already uses for
## the Sculpt panel's Draft section.
##
## Reads `world_structure.enabled` itself rather than taking it as an
## argument, so every caller -- a live click on that row
## (`_on_bool_row_changed`), a generate that followed a File ▸ New world
## archetype preset (`_on_generation_finished`), or a loaded save that
## carried a different value (`_on_world_loaded`) -- re-gates against
## whatever is actually true now, not against whichever of the three paths
## happened to call this.
##
## `is_instance_valid` rather than trusting the dictionary: a row this dock
## built once could in principle have been freed from under it by then, and a
## dead `HSlider` reference is a crash, not a silent no-op, without the guard.
func _refresh_ws_override_rows() -> void:
	var enabled := bool(bridge.param_get("world_structure.enabled"))
	for key in _ws_override_sliders:
		var s := _ws_override_sliders[key] as HSlider
		if not is_instance_valid(s):
			continue
		s.editable = not enabled
		var base := String(_ws_override_base_hint.get(key, ""))
		var text := ("%s %s" % [WS_OVERRIDE_REASON, base]) if enabled else base
		s.tooltip_text = text
		var row := s.get_parent() as Control
		if row != null:
			## The whole row dims, not just the slider -- see the matching
			## build-time comment in `_build_param_row`.
			row.modulate = Color(1.0, 1.0, 1.0, WS_OVERRIDE_DIM) if enabled else Color.WHITE
			row.tooltip_text = text

## Writes the value continuously (cheap: `param_set` is an in-memory Rust
## write, no recompute) but does not regenerate -- matches `tparam()`'s
## `input` handler updating only the label.
func _on_float_row_input(v: float, key: String, is_int: bool) -> void:
	bridge.param_set(key, (int(round(v)) if is_int else v))

func _on_float_row_released(key: String, is_int: bool, stage_index: int) -> void:
	_mark_stale_from(stage_index)
	_regenerate_live()

## `Editing any field sets stale from NN` (the Android spec). `stage_index`
## is whichever of `STAGES` the edited row lives under -- the earliest one
## wins, matching "stale from" naming the FIRST stage an edit could have
## invalidated, not the last. Cleared on `generation_finished`, not here:
## see `_stale_from_stage`'s own doc comment for why the badge outlives the
## edit that set it.
func _mark_stale_from(stage_index: int) -> void:
	if _stale_from_stage < 0 or stage_index < _stale_from_stage:
		_stale_from_stage = stage_index
	if _stale_note_label != null:
		_stale_note_label.text = _stale_note_text()

func _stale_note_text() -> String:
	if _stale_from_stage < 0:
		return "Not stale -- the map matches every dial below."
	return "Stale from %02d %s -- edited since the last Generate." % [
		_stale_from_stage + 1, String(STAGES[_stale_from_stage]["name"])]

## The one thing every generation control now triggers on release, exactly
## like the reference's own `withBusy('generating…', generate)`: the whole
## world, from stage 01, with whatever the dock's sliders currently say. No
## staleness to track -- by the time this returns, the map matches the dials
## again, same as it always did in the app being ported.
##
## **Guarded since 2026-09-01, because it is destructive and did not say so.**
## `WorldGen::absorb` (`lib.rs`) replaces `self.icons`, `self.labels`,
## `self.paint`, `self.infra`, `self.civ_tools` and `self.sculpt` with fresh,
## empty editors on every `generate()` -- deliberately, since grid coordinates
## from a previous generation mean nothing over a new one. That is defensible
## for File ▸ New world. It was not defensible here: releasing a slider is not
## an action a person reads as "throw away every icon, label, painted cell,
## hand-drawn way and route on this map", and nothing asked them first.
##
## So the prompt fires only when there is something to lose, which makes it
## self-limiting: once the user accepts, the layers are gone and the count is
## zero, so the next twenty slider drags are silent again.
##
## **Cancel leaves the parameter written and the world not rebuilt**, which
## is a state this dock already has a name and a readout for: the value went
## into the engine on `param_set` during the drag, `_mark_stale_from()` ran
## before this call, and the badge reads "Stale from NN -- edited since the
## last Generate" until a Generate actually happens. Reverting the dial on
## Cancel would be the surprising behaviour, not this one.
##
## **One Generate button is still outside this guard**, and it is not in this
## file: `app.gd`'s tool-options row calls its own `_run_pipeline()`, which
## reaches `bridge.generate()` directly. `_on_generate_pressed()` -- the
## dock's own copy of that button -- routes through here and does prompt.
func _regenerate_live() -> void:
	if app == null or app.new_world_dialog == null or bridge.generating:
		return
	var at_stake := _authored_inventory()
	if at_stake.is_empty():
		_regenerate_now()
		return
	_confirm_discard(at_stake)

func _regenerate_now() -> void:
	if app == null or app.new_world_dialog == null or bridge.generating:
		return
	bridge.generate(app.new_world_dialog.request())

## What a regenerate would destroy, as ready-to-print phrases. Empty when the
## world carries no hand-authored work at all.
##
## **Not exhaustive, and the prompt is worded so that it does not have to be.**
## `paint_painted_counts()` reports the *active* paint layer only
## (`paint_bridge::PaintEditor::painted_counts` reads `active_layer()`), so
## committed dabs on a layer the panel is not currently showing are not counted
## here; `paint_draft_count()` covers all three layers but only their pending
## halves. Iterating the layers to close that would mean calling
## `paint_set_layer()` three times as a side effect of a read, and `set_layer`
## clamps the brush value -- a read that quietly edits the brush is worse than
## an undercount. The dialog therefore names every category in prose and counts
## the ones it can count.
func _authored_inventory() -> PackedStringArray:
	var out := PackedStringArray()
	if not bridge.has_world:
		return out
	var stamps := bridge.sculpt_stamp_count()
	if stamps > 0:
		out.append("%d sculpt stamp%s on the draft" % [stamps, "" if stamps == 1 else "s"])
	var icons := bridge.icon_list().size()
	if icons > 0:
		out.append("%d placed map icon%s" % [icons, "" if icons == 1 else "s"])
	var labels := bridge.label_list().size()
	if labels > 0:
		out.append("%d map label%s" % [labels, "" if labels == 1 else "s"])
	var painted := int(bridge.paint_painted_counts().get("total", 0)) + bridge.paint_draft_count()
	if painted > 0:
		out.append("%d painted cell%s" % [painted, "" if painted == 1 else "s"])
	var routes := bridge.route_count()
	if routes > 0:
		out.append("%d route%s" % [routes, "" if routes == 1 else "s"])
	var ways := 0
	for w in bridge.roads():
		if (w as Dictionary).get("manual", false):
			ways += 1
	for w in bridge.sea_routes():
		if (w as Dictionary).get("manual", false):
			ways += 1
	if ways > 0:
		out.append("%d hand-drawn way%s" % [ways, "" if ways == 1 else "s"])
	return out

## The destructive answer is named after what it does, never "OK" -- the same
## wording rule `app.gd`'s own `_confirm()` follows. Built here rather than
## borrowed from `app.gd` because that helper is private to that file and this
## pass does not own it.
func _confirm_discard(at_stake: PackedStringArray) -> void:
	var dlg := ConfirmationDialog.new()
	dlg.title = "Regenerate this world?"
	dlg.dialog_text = ("Generating rebuilds the world from stage 01 and starts every "
		+ "hand-authored layer over it empty again: sculpt drafts, painted cells, map "
		+ "icons and labels, hand-drawn ways and routes. None of it is recoverable -- "
		+ "there is no undo across a generate.\n\nOn this world right now:\n  • "
		+ "\n  • ".join(at_stake)
		+ "\n\nSave the project first if you want to keep it.")
	dlg.ok_button_text = "Regenerate and discard"
	dlg.confirmed.connect(_regenerate_now)
	dlg.visibility_changed.connect(func(): if not dlg.visible: dlg.queue_free())
	add_child(dlg)
	dlg.popup_centered()

## Clears per-stage timing and the log, then repaints -- for the start of a
## NEW run (`generation_started`) or a freshly loaded world (`world_loaded`),
## both of which leave any previous run's timing stale. NOT called mid-run:
## `_on_generation_stage` calls the read-only `_paint_stage_rows` instead, so
## a stage that already finished is never wiped by a later stage's own
## signal arriving.
func _reset_stage_progress() -> void:
	_stage_name_checked.clear()
	for i in _stage_start_msec.size():
		_stage_start_msec[i] = -1
		_stage_elapsed_ms[i] = -1
	_stage_log.clear()
	if _stage_log_label != null:
		_stage_log_label.text = ""
	_paint_stage_rows()

## §5.1's state column, repainted from the CURRENT `_stage_start_msec`/
## `_stage_elapsed_ms`/`bridge.generating`/`bridge.has_world` state without
## resetting anything -- the read-only half `_on_generation_stage` (mid-run),
## `_on_generation_finished` (end of run) and `_build()` (first paint) all
## share, so a stage that already finished is never shown as reset by a
## later call. Nothing here claims a stage finished before
## `bridge.generation_stage` actually said so, and this engine still has no
## partial recompute -- see `_stale_from_stage`'s own doc comment.
func _paint_stage_rows() -> void:
	for i in _stage_state_labels.size():
		var lbl: Label = _stage_state_labels[i]
		if not bridge.has_world and not bridge.generating:
			lbl.text = "no world"
			lbl.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		elif i < _stage_elapsed_ms.size() and _stage_elapsed_ms[i] >= 0:
			## Real timing exists for this stage (`progress_api` true on this
			## build) -- show it rather than a generic "resolved".
			var gap_note := "  (no engine work this run -- see gap note above)" \
				if i == 8 or i == 9 else ""
			lbl.text = "%s done  %.2fs%s" % [
				DccIcons.SYMBOLS["tick"], _stage_elapsed_ms[i] / 1000.0, gap_note]
			lbl.add_theme_color_override("font_color", DccTheme.c("text_dim"))
		elif bridge.generating and i < _stage_start_msec.size() and _stage_start_msec[i] >= 0:
			lbl.text = "%s running…" % DccIcons.SYMBOLS["on"]
			lbl.add_theme_color_override("font_color", DccTheme.c("accent"))
		elif bridge.generating:
			lbl.text = "pending"
			lbl.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		else:
			## Has a world, not generating, no timing recorded for this row --
			## either an older cdylib with no `GenerationProgress`
			## (`progress_api` false) or a loaded save, which never goes
			## through a per-stage-signalled generate at all.
			lbl.text = "%s resolved" % DccIcons.SYMBOLS["tick"]
			lbl.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	_push_dock_readout()

## Wired to `bridge.generation_stage` -- fires once per stage the engine
## actually reaches (`engine_bridge.gd`'s own doc comment on why it is
## change-only, not once per frame). `index` can jump by more than one: a
## stage with no code of its own in `generate_terrain_inner` (Planet, Extent
## & scale -- `cartalith-engine/src/progress.rs`'s own doc comment) can tick
## through between two polls, so every not-yet-closed stage below `index` is
## closed out here too, not just `index` itself.
func _on_generation_stage(index: int, stage_name: String, total: int) -> void:
	if index < 0 or index >= _stage_state_labels.size():
		return
	_assert_stage_names(index, stage_name, total)
	var now := Time.get_ticks_msec()
	for i in index:
		if _stage_elapsed_ms[i] < 0:
			if _stage_start_msec[i] < 0:
				_stage_start_msec[i] = now
			_stage_elapsed_ms[i] = now - _stage_start_msec[i]
			_log_stage(i)
	if _stage_start_msec[index] < 0:
		_stage_start_msec[index] = now
	_paint_stage_rows()

## Appends one line to the rolling "per-stage progress + log" (the Android
## spec's own phrase). `_paint_stage_rows` is what actually paints
## `_stage_state_labels`; this only maintains the separate scrolling log
## text under the ten rows.
func _log_stage(i: int) -> void:
	_stage_log.append("%02d %s -- %.2fs" % [i + 1, String(STAGES[i]["name"]), _stage_elapsed_ms[i] / 1000.0])
	while _stage_log.size() > STAGE_LOG_MAX:
		_stage_log.pop_front()
	if _stage_log_label != null:
		_stage_log_label.text = "\n".join(_stage_log)

## The other half of `_assert_stage_groups()`, and the one it could not do:
## group names can be checked before a run, but a stage's *name* only
## crosses the boundary while one is happening, on `generation_stage`'s own
## second argument. That argument was received as `_stage_name` and thrown
## away, which left `STAGES` free to disagree with `progress::STAGE_NAMES`
## silently -- the rows would carry the wrong labels and the "stale from NN"
## badge would name the wrong stage, with nothing anywhere to say so.
##
## At most one report per index per run: a `push_error` on every stage tick
## would bury the first, real message under repeats of itself.
##
## `push_error` rather than `assert()`, for `_assert_stage_groups()`'s own
## stated reason -- `assert` is stripped from a release build, and this is
## precisely the drift that survives to one unnoticed.
func _assert_stage_names(index: int, stage_name: String, total: int) -> void:
	if stage_name.is_empty() or _stage_name_checked.has(index):
		return
	_stage_name_checked[index] = true
	if total > 0 and total != STAGES.size():
		push_error(
			"Cartalith: this dock draws %d generation stages; the engine reports %d "
			% [STAGES.size(), total]
			+ "(cartalith-engine/src/progress.rs STAGE_NAMES/STAGE_COUNT). Every row "
			+ "below the first difference is labelled with the wrong stage.")
	var mine := String(STAGES[index]["name"])
	if mine == stage_name:
		return
	push_error(
		"Cartalith: stage %02d is \"%s\" in this dock's STAGES table and \"%s\" "
		% [index + 1, mine, stage_name]
		+ "in the engine's own progress::STAGE_NAMES. The two are index-coupled: "
		+ "the progress rows, the per-stage log and the \"stale from NN\" badge "
		+ "are all naming the wrong stage until STAGES is brought back in line.")

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
		## `04-left-dock.md` §2.4's global keydown map is `{v,m,r,b,l,i,f}` --
		## `f` is Freehand, and unlike the five broken CIVIL letters the spec's
		## own defect note calls out, this one names a real chip. Wired here,
		## on the chip itself, rather than as a second TOOLS-block button (see
		## this function's own header comment for why a coarse "Freehand"
		## pill was not added): `DccWidgets.tool_button()`'s `Shortcut` only
		## fires while its button `is_visible_in_tree()`, so `F` arms Freehand
		## exactly when this grid is on screen (world domain, Terrain category
		## open, a world generated) and is inert everywhere else -- narrower
		## than the spec's apparently-global binding, but never claims a reach
		## this chip does not actually have.
		if key == "freehand":
			var freehand_key := InputEventKey.new()
			freehand_key.keycode = KEY_F
			var freehand_shortcut := Shortcut.new()
			freehand_shortcut.events = [freehand_key]
			btn.shortcut = freehand_shortcut
			btn.shortcut_in_tooltip = false
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
	dice.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	dice.pressed.connect(_on_sculpt_seed_dice)
	seed_row.add_child(dice)

func _on_global_changed(v: float, key: String, is_int: bool) -> void:
	bridge.sculpt_set_globals({key: (round(v) if is_int else v)})

func _on_sculpt_seed_dice() -> void:
	bridge.sculpt_set_seed(randi())
	_build_sculpt(_sculpt_body)

## GUI replacement stage 4 re-verified this against the current authority,
## `04-left-dock.md` §5.5 (Brush shape: 8 shape chips, Import brush, operation,
## falloff, mirror) and §5.6 (Stroke & grid: 13 stamp-geometry chips) --
## neither `cartalith-terrain::sculpt` nor `sculpt_bridge.rs` exposes a shape,
## an operation override, a falloff curve, a mirror flag, or any per-stamp
## control-point edit (`grep`-checked this pass: the bridge's sculpt surface
## is feature/preset/globals/freehand-mode/seed/stroke/stamp-stack only).
##
## That absence is not a reason to build them as decoration -- §5.5/§5.6's own
## text says the *prototype itself* mocks them ("every one of the 13 is a
## mock", `Import brush -- ... (mock)`), so a toast-only build here would be
## copying a mock, not closing a gap. Honest "not built" prose stays the
## right call under this port's own rule that a control doing nothing is
## drawn disabled with its real reason, not drawn as a working-looking button
## that silently does less than it appears to.
func _build_sculpt_unbuilt_note(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	DccWidgets.note(sec,
		"Brush shape (8 falloff shapes, Import brush, custom Falloff), Stroke & grid " +
		"(Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align control-point " +
		"editing) and Actions (Flip X/Y, Rot Left/Right, Flatten) have no engine behind them. " +
		"The design's own prototype mocks these too (04-left-dock.md §5.5-§5.6: every one of " +
		"its 13 Stroke & grid chips just toasts \"edits the stamp control points, not the " +
		"heightfield (mock)\") -- new, unscoped design work, not a port gap.")

## The draft/stamp-stack summary and Commit/Discard -- §5.2 places these at
## the foot of the left dock (`#sculptCommitBtn`/`#sculptDiscardBtn`); the
## full stamp-by-stamp list with its own Undo/Redo lives in the right dock
## (§6, `right_dock.gd`'s `_build_sculpt`) since that is where §6 puts it.
## Held so `_refresh_sculpt_draft()` can re-gate them when the stack changes.
## Before 2026-09-01 this section read the count once and never again, so a
## stroke drawn while the panel was already built left Commit and Discard
## greyed over a non-empty draft. The right dock never had the bug because
## `show_sculpt_stack()` rebuilds it wholesale.
var _sculpt_count_note: Label
var _sculpt_commit_btn: Button
var _sculpt_discard_btn: Button

func _build_sculpt_draft(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Draft")
	var count := bridge.sculpt_stamp_count()
	_sculpt_count_note = DccWidgets.note(sec, "%d stamp%s on the draft." % [count, "" if count == 1 else "s"])

	var actions := DccWidgets.group(sec, "Commit")
	var commit_btn := DccWidgets.action(actions, "%s Commit to map" % DccIcons.SYMBOLS["tick"], _on_sculpt_commit, true)
	commit_btn.disabled = count == 0
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_sculpt_discard)
	discard_btn.disabled = count == 0
	_sculpt_commit_btn = commit_btn
	_sculpt_discard_btn = discard_btn
	if not bridge.sculpt_draft_changed.is_connected(_refresh_sculpt_draft):
		bridge.sculpt_draft_changed.connect(_refresh_sculpt_draft)
	DccWidgets.note(sec,
		"Commit bakes the whole stamp stack into the heightfield in one pass and marks the " +
		"tiles it touched stale -- it deliberately does not re-run erosion, hydrology or " +
		"climate (measured ~7s/stroke at 2048² and rejected on that ground; " +
		"DCC_SHELL_SPEC.md header correction #1). A draft carries no lock state of its " +
		"own: the lock is per-world and lives in the Finalize section above, which bakes " +
		"the LOD pyramid and then refuses further sculpting until it is un-finalized.")
	_build_force_lake_row(sec)

## Re-gate the Draft section against the live stack. Cheap enough to run on
## every change: three property writes, no rebuild.
func _refresh_sculpt_draft() -> void:
	if not is_instance_valid(_sculpt_commit_btn):
		return
	var count := bridge.sculpt_stamp_count()
	_sculpt_commit_btn.disabled = count == 0
	_sculpt_discard_btn.disabled = count == 0
	if is_instance_valid(_sculpt_count_note):
		_sculpt_count_note.text = "%d stamp%s on the draft." % [count, "" if count == 1 else "s"]
## `buildWaterBodies`' `opts.forceLake` (reference HTML lines 5808-5809) --
## `PARITY_AUDIT.md` §23 F13. The Lake stamp already accumulates a mask on every
## commit (`WaterState::lake_mask`) and `cartalith_civ::apply_force_lake` has
## been ported and tested since milestone C, but nothing joined the two: a
## painted lake was terrain that happened to be lower, and every civ tool that
## reads the water-body classification still saw land there.
##
## Its own button rather than an automatic tail on Commit because the join
## would have to live in `sculpt_commit` (`lib.rs`), which this task does not
## own -- and because the reference's own `forceLake` is an option a caller
## passes, not something `buildWaterBodies` does by itself.
func _build_force_lake_row(parent: Control) -> void:
	var grp := DccWidgets.group(parent, "Painted lakes", false)
	DccWidgets.note(grp,
		"Reclassifies every cell a Lake stamp has deposited as a lake, whether or not its " +
		"floor ended up below sea level or its basin catches enough rain to pool -- the " +
		"reference's own forceLake semantic. Affects settlement placement, routing, trade and " +
		"the Journey Planner, which all read the water-body classification. It does not touch " +
		"the height field, marks nothing stale, and is undone by the next full civ recompute.")
	var live := bridge._has("apply_force_lake")
	var btn := DccWidgets.action(grp, "Count painted lakes as water", _on_force_lake)
	btn.disabled = not live
	btn.tooltip_text = "cartalith_civ::apply_force_lake, over this world's live classification." \
		if live else "This build's GDExtension has no WorldGen.apply_force_lake()."

func _on_force_lake() -> void:
	if not bridge.has_world or not bridge._has("apply_force_lake"):
		return
	var r: Dictionary = bridge.world_gen.apply_force_lake()
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Painted lakes: %s" % String(r.get("reason", "unavailable")), "accent")
		return
	var forced := int(r.get("forced", 0))
	app.set_status("hint",
		("Painted lakes: every stamped cell was already water." if forced == 0
			else "Painted lakes: %d cell%s reclassified (%d lake cells now)."
				% [forced, "" if forced == 1 else "s", int(r.get("lake_cells", 0))]),
		"text_ghost")

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
	## `05-right-dock-and-bars.md` §1.8. Mirrors the Sculpt line right above --
	## `leave_paint_context()` is the other half, called from `app.gd`'s own
	## workspace-switch handler the same way `leave_sculpt_context()` is,
	## since Biome paint is a WORLD-only tool with nothing else that clears
	## this context on a domain switch.
	if id == "paint" and app.right_dock_ctrl.has_method("show_paint"):
		app.right_dock_ctrl.show_paint(_paint_layer, _on_paint_value_picked_from_dock)

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
	## `DECISIONS.md` §7k -- bound 2026-08-31 (`LARGE_ITEM_RULINGS.md`). Both
	## feather the dab's own edge (which cells get touched, never which
	## palette index a touched one receives -- painting is still a hard
	## disc's worth of *values*, just not of *coverage*). This is the sole
	## surviving copy of Hardness: `tool_bar.gd`'s own row used to draw a
	## second one live at the same time, and `UNWIRED_FUNCTIONS.md` flagged
	## that as its own defect on top of the falloff being unwired -- resolved
	## by deleting that copy rather than this one, since this dock owns the
	## actual `_paint_brush` state and is where Softness already lived too.
	DccWidgets.slider(sec, "Hardness", 0.0, 1.0, 0.01, float(_paint_brush["hardness"]), "", _on_paint_hardness_changed,
		"At 1.0, with Softness at 0.0, every cell inside Radius paints solid -- the historical hard disc, unchanged. Lower it to open a mottled, probabilistic edge band instead of a sharp circle; no palette index is ever blended (paint_bridge.rs's own module doc, DECISIONS.md §7k).")
	DccWidgets.slider(sec, "Softness", 0.0, 1.0, 0.01, float(_paint_brush["softness"]), "", _on_paint_softness_changed,
		"The same edge band as Hardness, from the other side: raising this alone still feathers the rim even with Hardness held at 1.0 -- the two add together, clamped to how wide the band can get.")
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
	_refresh_right_dock_paint()

## `right_dock.gd`'s CTX_PAINT (§1.8) reads `bridge.paint_painted_counts()`,
## which answers for whichever layer is active server-side -- so this dock
## must re-announce itself every time that changes here (layer switch,
## commit, discard, stroke release), the same "no private draft" contract
## every other `show_*` call in this file already keeps. A no-op while some
## other context owns the right dock, since `show_paint` would otherwise
## steal it back from e.g. a Settlement selection made mid-paint.
func _refresh_right_dock_paint() -> void:
	if app.armed_tool == "paint" and app.right_dock_ctrl.has_method("show_paint"):
		app.right_dock_ctrl.show_paint(_paint_layer, _on_paint_value_picked_from_dock)

## Bound into `show_paint()` so this dock's own legend can arm a palette
## value without right_dock.gd guessing at radius/hardness/softness/
## land_only -- see that method's own doc. Mirrors `_on_paint_value_changed`
## exactly, just keyed by the real palette index rather than a position into
## the `OptionButton`'s own option list (the right dock already has the real
## index off `get_paint_palette()`, so no lookup is needed here).
func _on_paint_value_picked_from_dock(value_index: int) -> void:
	_paint_brush["value"] = value_index
	_sync_paint_brush()
	if is_instance_valid(_paint_body):
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
	_refresh_right_dock_paint()

func _on_paint_discard() -> void:
	bridge.paint_discard()
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture())
	_build_paint(_paint_body)
	_rebuild_tool_bar()
	_refresh_right_dock_paint()

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
	_refresh_right_dock_paint()

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
