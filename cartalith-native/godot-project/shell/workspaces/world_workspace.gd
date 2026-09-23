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
	 "groups": [], "keys": ["carve_rivers", "river_density", "integrate_drainage"],
	 "gap": "Min stream order and lakes-as-water are reference render filters, not generation parameters -- and not Cartography's either, which this line said until 2026-09-07. Neither is settable anywhere: since 2026-09-22 the drawn rivers are the traced polylines get_rivers(min_order) returns, but the viewport always asks for min_order 1 and no control changes it."},
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
## an empty list is one that carries no pipeline parameters of its own -- either
## prose and readouts (World data), or a hand-editing block (Terrain, Biomes).
##
## **Re-sorted 2026-09-21 by the owner** (`LARGE_ITEM_RULINGS.md` Ruling L;
## `design/owner-references-2026-09-12/left_rail_tree_resorted.md` is the
## specification and the tree there is leading where its notes disagree with
## it). v3's nine categories became **seven PIPELINE plus two SCULPT**, and the
## split is the mode pill's, not a category's:
##
## - `Terrain` stops doing double duty. Its stage-05/06 half moves out (erosion
##   water & ice to `Hydrology`, hillslope diffuse to `Geology`) and its
##   heightmap entry point moves to `Generate ▸ Import`, leaving the sculpt
##   block alone behind the SCULPT pill.
## - `Planet` is new, and is the *input* half of the old `World data` (01, 02)
##   plus `Generate`'s stage 03. `World data` keeps only readouts.
## - `Biomes` stops being a pipeline category: stage 09 folds into `Ecology`
##   ("stage 09 is one stage"), and `Biomes` becomes SCULPT's paint block --
##   Ruling L decision 2, which makes the left dock the ONE home for the brush.
## - `Resources` is removed outright -- Ruling L decision 3: its values are
##   calculated rather than set, so stage 10 stays a read-only row in Pipeline
##   status and nothing here claims a dial that does not exist.
##
## The order below IS the tree's order, and the last two rows are the ones
## `RAIL_NODES`' `world/b` gates (`dcc_shell.gd`).
const CATEGORIES: Array = [
	{"name": "Generate", "stages": [],
	 "lead": "The one act: seed, extent, steering, run -- plus the non-seed way in, and the terminal bake. Every parameter in the categories below feeds this call, and this call resolves all ten pipeline stages at once -- there is no partial recompute in this engine or in the app it ports."},
	{"name": "Planet", "stages": [0, 1, 2],
	 "lead": "The planet the world sits on, the scale it is measured in, and the continental steering that shapes it. Everything here is an input to generation rather than a product of it."},
	{"name": "Geology", "stages": [3, 4],
	 "lead": "What the rock is and where it was pushed: plates, uplift, volcanism, impacts and rock resistance. Everything here runs before erosion and is what erosion cuts into."},
	{"name": "Hydrology", "stages": [6],
	 "lead": "Water and ice over the finished surface: what they cut into it, and the rivers, lakes, drainage and flow accumulation that come out."},
	{"name": "Climate", "stages": [7],
	 "lead": "Temperature, rainfall, wind and currents, over the finished surface and under the planet's own geometry."},
	{"name": "Ecology", "stages": [8],
	 "lead": "Classification off the finished temperature/rainfall/elevation fields, and what lives on it. Biome *colours* are Cartography's -- v3's own split; the brush that overrides the classification by hand is Sculpt ▸ Biomes."},
	{"name": "World data", "stages": [],
	 "lead": "Readouts over the finished world: the field browser, the GeoJSON export, and the coordinate frame both are written in."},
	{"name": "Terrain", "stages": [],
	 "lead": "Height molding by hand, over the current surface. Elevation, slope, curvature and relief are readable as analysis fields -- Cartography ▸ Layers ▸ Data overlays."},
	{"name": "Biomes", "stages": [],
	 "lead": "Painting biome and terrain classes by hand, over the classified fields. Ruling L decision 2: this is the brush's one home, and Biome paint (B) in the tool palette is the way to arm it."},
]

var _sculpt_body: VBoxContainer
var _paint_body: VBoxContainer
## The TOOLS block's second row -- WORLD's one domain tool, `Biome paint (B)`.
## Ruling L shows it in SCULPT mode only; see `_build()` and `apply_mode()`.
var _paint_tool_row: Control
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

	## Tablet only, lane GRID 2026-09-13: a rotation has to reflow every
	## slider row between this dock's own landscape/desktop shape and the
	## canvas's two-line portrait cell -- `DccTheme.role_px()` alone
	## re-answers a plain numeric read on the very next layout pass with no
	## hook needed, but a STRUCTURAL swap (which container a control lives in)
	## needs an explicit one. See `DccTheme.watch_portrait`'s own doc for why
	## this is a plain Array of Callables rather than a signal, and
	## `_on_portrait_changed` for the reflow itself.
	##
	## Registered once: `_build()` runs exactly once per panel instance (this
	## file's own "the ONE `_build_categories()` pass `setup()` runs" note,
	## a few screens down), so the panel's whole lifetime needs exactly one
	## watcher. A panel freed by a project re-open (a fresh `app.tscn`, this
	## lane's own probe) leaves a dead entry that `set_portrait()` prunes the
	## next time the axis actually flips, not a second registration here.
	if DccTheme.is_tablet():
		DccTheme.watch_portrait(_on_portrait_changed)

	## **Off the phone this row is drawn in the top palette bar, not this dock**
	## (owner, 2026-09-23; `DccWidgets.tools_block()`), and `_paint_tool_row`
	## below is then null -- the bar applies the same SCULPT-only gate from the
	## entry's `modes`. The rest of this comment is the row's design history.
	##
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
	##
	## **Ruling L gates this row on SCULPT mode**, 2026-09-21 -- the tree's own
	## condition on the row (*"Biome paint (B) [tool] (SCULPT mode only)"*), and
	## the grouping rule behind it: *"each tab's Tools row lists only tools that
	## can be armed there"*, and WORLD ▸ PIPELINE has no brush in it. The `B`
	## shortcut follows the pill by construction rather than by a second rule:
	## `BaseButton::shortcut_input` fires only while the button
	## `is_visible_in_tree()`, which is the same mechanism `F` already relies on
	## in `_build_feature_picker`.
	##
	## The row is recovered from the button `tools_block()` just built rather
	## than by child index, because `tools_block()` returns nothing and a
	## positional read of a shared factory's output is a silent breakage the day
	## that factory adds a node.
	DccWidgets.tools_block(self, app, app.tool_group, [
		{"id": "paint", "glyph": "tool_paint", "label": "Biome paint (B)", "modes": ["b"]},
	])
	_paint_tool_row = _find_tool_row("Biome paint (B)")
	_refresh_paint_tool_row(app.active_mode(domain_id) if app.has_method("active_mode") else "")

	## **The two-button switch (Generation pipeline | Sculpt) came back on
	## 2026-09-05, in the dock chrome rather than here.** It was removed on
	## 2026-08-24 on the reading that v3 has no such control and that WORLD's
	## only disclosure should be the accordion, like CIVIL and CARTO. The owner's
	## 2026-09-05 ruling (`LARGE_ITEM_RULINGS.md` item 2) settles that the other
	## way for this domain: `04-left-dock.md` §2.1 band 2 and §2.3 draw a
	## two-segment mode pill, §3 rows 1 and 2 make WORLD ▸ a and WORLD ▸ b
	## genuine complements, and `DccShell.RAIL_NODES`' `shows` gate now hides
	## eight of WORLD's nine categories in Sculpt. **So WORLD does have a hidden
	## half again, deliberately, and the switch is the affordance that admits
	## it** -- `DccShell._build_mode_switch()` owns it, pinned above the scroll
	## body where the canvas puts it, so it never scrolls away from what it
	## hides. CIVIL and CARTO are still accordion-only; neither gates.
	##
	## **The pill's two captions are QUOTED FROM THE CANVAS.** `SCULPT` is
	## `ldSwB` verbatim and `PIPELINE` is `ldCollapsedLabel`'s own word for mode
	## a, both at `design/dcc-environment-2026-08-31/Cartalith DCC
	## Environment.dc.html:1937-1940`, and `04-left-dock.md:138` binds segment
	## B's text to `{{ ldSwB }}`.
	##
	## **This block said the opposite for one batch**, on the authority of
	## `LARGE_ITEM_RULINGS.md` §6's first consequence and `04-left-dock.md` §0's
	## truncation note. Both were stale: the canvas in this tree is **whole** --
	## 239 712 bytes, 1 994 lines, ending `</script></body></html>` -- since
	## commit `660cbef` ("Design answered: the files are whole"), and §0/§9.1
	## were never updated. Corrected 2026-09-06 after a verifier found this file
	## contradicting `dcc_shell.gd`, which had already been fixed in the same
	## batch. **Deriving here would have replaced two verbatim canvas strings
	## with a derivation and labelled a true provenance false.**
	##
	## The derivation below is kept as **corroboration, not provenance** -- it is
	## why the canvas's two words are the right ones for what these modes hold,
	## and it is probe-verified:
	##
	## - **mode `a`** owns seven of the nine `CATEGORIES` above -- Generate,
	##   Planet, Geology, Hydrology, Climate, Ecology, World data -- and
	##   `Generate`'s own `lead` says what they are collectively for: *"The
	##   one act: seed, extent, steering, run … this call resolves all ten
	##   pipeline stages at once."* Seven categories whose every parameter feeds
	##   one `generate()` call are a **pipeline**.
	## - **mode `b`** owns and `shows` exactly `Terrain` and `Biomes`, the two
	##   categories `_build_categories()` parents `_sculpt_body` and
	##   `_paint_body` into, and the two the hand tools jump the dock to. Two
	##   categories holding the height-molding and the biome brush are
	##   **sculpt** -- Ruling L's own grouping rule, *"everything done by hand
	##   on the map surface"*.
	##
	## `DccShell._MODE_SWITCH_LABELS` carries the resulting `PIPELINE` /
	## `SCULPT`. The rail-node labels (`Generation pipeline` / `Sculpt`) reach
	## the same two head nouns by a different route, which is corroboration and
	## not a source. **What is not claimed: that either word appears in any
	## canvas as the pill's own text.** Neither does.
	##
	## `_sculpt_body`/`_paint_body` are unaffected and are still parented into
	## their categories rather than into a mode panel -- the mode hides the
	## category, it does not re-home the body.
	_sculpt_body = VBoxContainer.new()
	_sculpt_body.add_theme_constant_override("separation", 0)
	_sculpt_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## **The `armed_tool == "sculpt"` gate is gone, 2026-09-21.** The 2026-09-07
	## owner instruction it implemented -- *"the sculpt tools must appear ONLY
	## when the sculpt menu is accessed"* -- is superseded **in mechanism, not
	## in intent**, by Ruling L: *"the re-sort makes SCULPT mode itself that
	## gate and has picking a feature arm the tool, so the armed-tool gate
	## goes."* The body is unconditionally visible and the SCULPT pill (plus
	## the `Terrain` accordion header) is what hides it, which is the design
	## recorded twenty lines up, restored deliberately rather than by drift.
	##
	## `_paint_body` below is the same shape for the same reason. It was set
	## `false` here and never set true, because Biome paint's controls lived in
	## the RIGHT dock only; Ruling L decision 2 makes `WORLD ▸ Sculpt ▸ Biomes`
	## the brush's real home, so the body it always built is now on screen. The
	## right-dock copy is a duplicate to retire once this is live -- not retired
	## here; see `_refresh_right_dock_paint()`, which keeps the two in step.
	_sculpt_body.visible = true

	## Biome paint stays a panel of its own, inside the BIOMES category --
	## SCULPT mode and the accordion are what show and hide it, the same two
	## gates `_sculpt_body` above answers to.
	_paint_body = VBoxContainer.new()
	_paint_body.add_theme_constant_override("separation", 0)
	_paint_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_paint_body.visible = true

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

## Ruling L's *"(SCULPT mode only)"* on the TOOLS row's `Biome paint (B)`.
##
## The base `apply_mode()` gates category wraps off `RAIL_NODES`' `shows`; the
## TOOLS block is not a category and is outside that gate, so this is the one
## extra thing WORLD's mode has to move. Called through the same single choke
## point every other transition into a mode passes through
## (`DccShell._select_domain()`), so there is no route into PIPELINE that can
## leave the pill on screen.
func apply_mode(mode: String) -> void:
	super.apply_mode(mode)
	_refresh_paint_tool_row(mode)

func _refresh_paint_tool_row(mode: String) -> void:
	if is_instance_valid(_paint_tool_row):
		_paint_tool_row.visible = mode == "b"

## **SCULPT owes a floor; PIPELINE does not.** `Workspace._floor_applies()`
## decides that by asking whether the active mode carries a `shows` list, which
## was a sound proxy while `world/b` was the only gated mode in the whole table.
## Ruling L makes PIPELINE the explicit complement (*"the mode pill is the only
## gate"*), so `world/a` carries a `shows` too and the proxy now answers "yes"
## for both halves -- which would re-open a PIPELINE header the user had just
## closed. That exact regression was fixed on 2026-09-05 and is pinned by
## `_leftdock12_probe.gd` §6(i) (*"closing it in an ungated mode leaves it
## closed"*); its own comment calls the pipeline view *"nine headers on screen,
## hiding nothing"*, which Ruling L turns into seven headers that do hide two,
## without changing what the floor was for.
##
## So WORLD names the mode that owes a floor rather than inferring it. The
## reason is SCULPT's alone and does not generalise: it renders the by-hand
## block and nothing else, so collapsing it leaves a dock of two headings with
## nothing under them, where PIPELINE's seven closing to zero is the same
## legible state CARTO's own headers are allowed to reach.
##
## This leaves the base's `mode_shows()` clause with no live caller -- CIVIL
## answers the first clause (`floor_category()` is `Populate`) and CARTO gates
## nothing -- so it is now the fallback for a domain that later gains a gate,
## which is what it was written as.
func _floor_applies() -> bool:
	if app == null or domain_id.is_empty() or not app.has_method("active_mode"):
		return false
	return String(app.call("active_mode", domain_id)) == "b"

## The TOOLS row holding the tool whose tooltip is `label` -- `tools_block()`
## builds one `HFlowContainer` per row and returns neither, and the button's own
## tooltip is its label (`DccWidgets.tool_button()`), so the row is found by
## what is in it rather than by where it sits.
func _find_tool_row(label: String) -> Control:
	var stack: Array = [self]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Button and String((n as Button).tooltip_text) == label:
			return n.get_parent() as Control
		stack.append_array(n.get_children())
	return null

# -- Ruling L's nine categories ------------------------------------------------

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
			## Ruling L files `06 Erosion · water & ice` above `River network`,
			## so it is a head rather than a foot: stage 05's water and ice
			## passes cut the surface stage 07 then drains.
			"Hydrology": _build_erosion_water_ice(body)
			_: pass

		var stages: Array = cat["stages"]
		for s in stages:
			_build_stage_body(body, int(s), stages.size() > 1 or name != String(STAGES[int(s)]["name"]))

		match name:
			"Generate": _build_generate_foot(body)
			"Terrain": body.add_child(_sculpt_body)
			"Geology": _build_erosion_hillslope(body)
			"Hydrology": _build_hydrology_foot(body)
			"Ecology": _build_ecology(body)
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

	_build_import(parent)

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
		## Stage 9/10's state label wraps to 3 lines when the dock is narrow
		## (see the `state_label` comment below); an HBoxContainer's default
		## cross-axis behaviour stretches/centres every child to the row's
		## own tallest member, so the fixed-size number/dot/name columns
		## drifted off the state label's own first line instead of sitting
		## beside it. `SIZE_SHRINK_BEGIN` pins every column's own natural
		## (unstretched) height to the row's top edge, so a 1-line label and
		## a 3-line label always share the same top line regardless of which
		## one made the row taller.
		number_label.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
		row.add_child(number_label)
		var dot_label := DccTheme.mono_label(DccIcons.SYMBOLS["off"],
			"text_ghost", DccTheme.FS_MICRO, 0)
		dot_label.custom_minimum_size.x = 9
		dot_label.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
		row.add_child(dot_label)
		var name_label := DccTheme.mono_label(String(STAGES[i]["name"]),
			"text_secondary", DccTheme.FS_MICRO, 1)
		name_label.custom_minimum_size.x = DccWidgets.ROW_LABEL_W - 33
		name_label.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
		## Lane GRID 2026-09-13, corrected same day (see `MISTAKES.md`'s
		## `clip_text` row): this four-column row (number, dot, name, state)
		## is this port's own readout mirror of the canvas's `hStage`
		## accordion row, and the name column is exactly `st.name` -- same
		## `white-space:nowrap;overflow:hidden;text-overflow:ellipsis` role
		## the canvas gives it, cited on `DccTheme.header()`'s own doc.
		## Without it, a name past the 99 px floor above (measured:
		## "Volcanism & impacts" 167, "Resources & soils" 149, "Ecology &
		## biomes" 140) is this dock's own widest driver in the portrait
		## 232 px dock -- wider than any slider cell this lane already fixed
		## -- because unlike `_row()`'s label this one is hand-built and
		## never had `clip_text` at all.
		##
		## **`is_tablet_portrait()`-gated at build time, not unconditional.**
		## The first cut of this clipped every geometry, reasoning that
		## ellipsis on text that already fits draws nothing -- true as far as
		## it goes, but it also cut "Volcanism & impacts" to "Volcanism &
		## im…" on a DESKTOP dock with room to spare, a real, measured
		## regression (`custom_minimum_size.x` is a floor here, not a
		## ceiling: a Label's natural minimum from its own text overrides it
		## upward when unclipped, which is why the full name drew past 99 px
		## on every geometry before this row existed at all). The build-time
		## read here only gets this row's INITIAL shape right; a later
		## rotation is `_on_portrait_changed`'s job, which re-derives every
		## entry in `_stage_name_labels` (appended two lines below) the same
		## way it already reflows slider cells -- see that function's own
		## doc for the landscape-boot-then-rotate gap this closes.
		if DccTheme.is_tablet_portrait():
			name_label.clip_text = true
			name_label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		row.add_child(name_label)
		var state_label := DccTheme.mono_label("pending", "text_ghost", DccTheme.FS_MICRO, 1)
		state_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		## Cross-axis stays `SIZE_SHRINK_BEGIN` (see the number/dot/name
		## columns above) so a 3-line wrap grows the row downward from a
		## shared top edge instead of the row's default stretch/centre
		## pushing this label's own first line up past its siblings.
		state_label.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
		state_label.vertical_alignment = VERTICAL_ALIGNMENT_TOP
		## DS-03. Most of this column is two words -- "pending", "no world",
		## "running...". Two rows are not: stages 9 and 10 append
		## `_paint_stage_rows()`'s gap note, and the finished string is
		## "done  3.83s  (no dials of its own -- see gap note above)" (2026-09-21:
		## reworded from "(no engine work this run -- see gap note above)",
		## which read as "this stage was skipped" to a PC-screenshot review --
		## it was not; see `cartalith-engine/src/progress.rs`'s doc comment.
		## Four characters SHORTER than the string the 545 px figure below was
		## measured against, so that figure still bounds this one). A
		## `Label`'s minimum width is its whole text unless it wraps, measured
		## 545 px for the old string, and this one is `SIZE_EXPAND_FILL` in a
		## row that already spends 18 + 9 + `ROW_LABEL_W - 33` px on its three
		## fixed siblings -- so with a world generated the left dock was forced
		## from 400 px to **783** on tablet, taking that width off the map. It
		## only appears after a generate, which is why the boot-state sweep in
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

	## **The `Not a generation stage` section is gone, 2026-09-21.** Ruling L
	## removes it outright -- *"the grouping rule says it"*: the re-sorted tree's
	## own grouping rules state where each of the five things it routed to
	## lives (Preferences for program scope, CARTO for map style, CIVIL for
	## people), so a section restating them under Finalize is a second source
	## for placement that the tree itself now answers.

## Ruling L's `Generate ▸ Import` -- *"the non-seed way in; both halves of it
## together"*. Both buttons already existed and neither changed: `Load
## heightmap…` was `Terrain ▸ Heightmap` and `Infer tectonics from heightmap…`
## was `Geology ▸ From an imported surface`, two sections apart in two
## categories, for one import that performs both acts in one call.
##
## The note's last clause used to read *"see Geology below for that pass on its
## own"*. That pointer is now false in the direction it points -- the pass sits
## in this same section, three rows down -- so it is rewritten rather than left
## to age (`MISTAKES.md`: hunt the prose that described the old placement).
func _build_import(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Import")
	var load_btn := DccWidgets.action(sec, "Load heightmap…",
		func(): app.open_data_manager("Import"))
	load_btn.tooltip_text = "The reference's #loadBtn. Opens Data ▸ Import, whose Heightmaps route decodes a PNG, takes it as the elevation field and infers tectonics under it."
	var infer := DccWidgets.action(sec, "Infer tectonics from heightmap…",
		func(): app.open_data_manager("Import"))
	infer.tooltip_text = "The reference's #inferTectBtn. Runs as part of the heightmap import (cartalith_engine::import::infer_tectonics) -- there is no separate #[func] to re-run it over an already-imported surface, so this opens the import that performs it."
	DccWidgets.note(sec,
		"An imported heightmap replaces the generated surface, and tectonics are "
		+ "inferred from it rather than kept -- which is why both rows above open "
		+ "the same import: one pass performs both.")

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

	_build_stage_meta(body, index, label_stage)

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

## A stage's `needs` / `produces` / `gap` prose, indented under its title.
##
## Split out of `_build_stage_body()` 2026-09-21: Ruling L takes stage 05 out
## of the `CATEGORIES` stage lists entirely and splits its two halves across
## `Hydrology` and `Geology`, so the meta block has a caller that is not a
## stage body any more (`_build_erosion_water_ice`, which owns the half the
## `needs`/`produces` prose is actually about).
##
## The mockup indents a stage's `needs`/`produces` under its title rather than
## running them to the dock's own edge, which is what `note()` on a bare body
## does.
func _build_stage_meta(body: Control, index: int, label_stage: bool) -> void:
	var stage: Dictionary = STAGES[index]
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
##
## **Ruling L splits this stage across two categories, 2026-09-21.** Five of the
## six passes cut with water or ice and go to `Hydrology`, above the drainage
## they feed (`_build_erosion_water_ice`); hillslope diffuse is creep, not
## water, and goes to `Geology` (`_build_erosion_hillslope`). Nothing about any
## pass changed -- the same `group()` bodies in the same order, with the same
## `stage_index` for staleness, in two homes instead of one.
##
## Decision 1 of Ruling L is what keeps `Erode (droplet)` and `Carve fjords`
## here rather than under Sculpt: *"on-demand passes grouped with the domain
## they act on"*.
func _build_erosion_water_ice(parent: Control) -> void:
	var body := DccWidgets.section(parent, "06 Erosion · water & ice")
	_build_stage_meta(body, EROSION_STAGE_INDEX, true)

	var real := DccWidgets.group(body, "Stream-power carve", true)
	for key in bridge.param_keys():
		var info := bridge.param_info(key)
		if String(info.get("group", "")) == "erosion":
			_build_param_row(real, key, EROSION_STAGE_INDEX)

	## Droplet is the one that is genuinely a BUTTON in this port too -- the
	## reference runs it from `#erodeBtn` over the finished field and
	## `generate()` never touches it, so it has no `passes.*` toggle and needs
	## its own control. §23 F11.
	_build_droplet_erosion(DccWidgets.group(body, "Droplet hydraulic", false))

	for pass_name in ["Velocity (momentum)", "Glacial", "Coastal"]:
		var grp := DccWidgets.group(body, pass_name, false)
		DccWidgets.note(grp, _erosion_pass_note())
		## The reference's Glacial panel carries two buttons, not one:
		## `#glacBtn` (glacialErode, which this port runs as `passes.glacial`)
		## and `#fjordBtn` (carveFjordsOp), a real, golden-verified port since
		## 2026-08-23 and the one true opt-in button of the four.
		if pass_name == "Glacial":
			_build_fjord_row(grp)

## The creep half of stage 06, filed under `Geology` by Ruling L -- *"06 Erosion
## · hillslope diffuse (note only -- generation-time toggle) ◄ Terrain ▸ 06
## Erosion (creep, not water)"*.
##
## `category_expander()` rather than a bare `DccWidgets.group()`: this is an
## expander at category level, beside two `§` sections, and a bare group draws
## flush against the dock edge while a section sits 14 px in. That is L2 of the
## CIVIL half's own repair round, and the helper is the one it added.
func _build_erosion_hillslope(parent: Control) -> void:
	DccWidgets.note(
		CivilizationWorkspace.category_expander(parent, "06 Erosion · hillslope diffuse", false),
		_erosion_pass_note())

## The one sentence the four note-only erosion passes share. One copy, because
## Ruling L now draws them in two different categories and two divergent
## transcriptions of the same disclosure is how a stale one survives.
func _erosion_pass_note() -> String:
	return ("Ported. This port runs it as a generation-time toggle rather than a button "
		+ "(DECISIONS.md §7d) -- its passes.* switch and dials are in Hydrology ▸ 06 Erosion · "
		+ "water & ice ▸ Stream-power carve, off by default. There is no separate run button "
		+ "because the pass is part of generate().")

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
		## Erosion is a WORLD pipeline category too -- Droplet hydraulic's five
		## sliders overflow the 232 px portrait dock exactly like every
		## `_build_param_row` slider does, and for the same reason
		## (`_wrap_slider_cell`'s own header). Nothing here reads `made["row"]`
		## afterward, so there is no second reference to redirect, and these
		## five have no right-click reset and no World-Structure override to
		## re-point at a new shape -- `Reset dials` below drives `sliders[i]`
		## directly, the one `HSlider` that never changes identity across a
		## reflow.
		if DccTheme.is_tablet():
			_wrap_slider_cell(made)
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
	## Same reason `_on_sculpt_commit` also calls this: an already-built LOD
	## tile keeps its OWN synthesized texture and a shader reference to the
	## OLD `map_view.texture`, so reassigning the field above alone leaves a
	## live deep-zoom pyramid compositing pre-erode relief. Cheap no-op when
	## nothing is live (`invalidate_lod_tiles()`'s own guard).
	app.viewport.invalidate_lod_tiles()
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
## The same three keys -> the row's CURRENT Control, resolved through
## `_current_row_ctl` rather than stored as one -- see that function and
## `_slider_reflow_entries` for why a tablet row cannot be a fixed reference.
## `_refresh_ws_override_rows` dims and tooltips whatever this resolves to.
var _ws_override_rows: Dictionary = {}

## Tablet only (both orientations), lane GRID 2026-09-13: every parameter
## slider row this build produces, so a live rotation
## (`DccTheme.watch_portrait`, wired once in `_build()`) can reflow each one
## in place -- see `_wrap_slider_cell` for what one entry holds and
## `_mount_wide`/`_mount_compact` for the reflow itself. Empty on every
## non-tablet geometry.
var _slider_reflow_entries: Array = []

## Tablet portrait only: `design/mcp-2026-09-07/Cartalith Tablet.dc.html`
## draws every generation-pipeline slider (`st.sliders`, lines 169-174) as a
## full-width, two-line cell -- a label/value baseline row, then its own
## `height:var(--rowD)` (44) touch band holding the track -- never
## label-beside-track the way this dock's own row is built for desktop and
## landscape tablet. It is NOT the 2-column grid that appears earlier in the
## same open body (`st.grid`/`g.cells`, lines 159-168): that grid is tap-to-
## cycle pill cells with a label over a value, a control kind no parameter row
## in this dock uses, and `TABLET_UI_SPEC.md` §2.4(a) already lists "a
## 2-column grid ..., sliders, ..." as two separate body elements for exactly
## this reason -- confirmed against the canvas's own markup rather than
## assumed from that sentence.
##
## This is why the row overflows the 232 px portrait dock in the first place:
## `ROW_LABEL_W` (132) + tablet `slider_track_w` (90) + `ROW_VALUE_W` (44) +
## 3×8 separation = 290, and neither constant takes a caller-side override --
## `DccWidgets._row()`/`slider()` hardcode them with no parameter to narrow.
## Reflow only (DS-03), and only what the "do not shrink globally" rule
## actually names: the label keeps its 132 px floor and the readout keeps its
## 44, both moved onto their own line rather than resized; only the slider's
## own track -- never named in that constraint -- changes from a fixed 90 px
## shrink-to-end control into one that expands to fill the new line's width,
## which is also a closer match to the canvas's own `flex:1` track. The three
## nodes are the exact ones `DccWidgets.slider()` built and wired: no control
## is replaced, no signal, value range or tooltip changes, so this belongs in
## `world_workspace.gd` rather than in the shared row builder it reflows.
##
## **Rewritten 2026-09-13** from a one-shot, one-way conversion (freed the
## wide row outright) to a two-shape swap: a landscape boot rotated to
## portrait used to keep reading the wide layout forever, and a portrait boot
## rotated to landscape kept the compact one -- `_build_param_row` runs once
## per row, so whichever shape that single call chose was permanent. Neither
## shape is ever rebuilt now; `_mount_wide`/`_mount_compact` reparent the same
## three live children (label, slider, readout) between two wrapper
## containers built once, here, and cached on the returned entry -- so a
## rotation changes no signal, no tooltip and no value, only which wrapper is
## in the tree.
##
## `made` is a `DccWidgets.slider()` result (`{"row","slider","readout",...}`);
## returns the reflow entry, appended to `_slider_reflow_entries` and also
## what callers store wherever they used to store `made["row"]`
## (`_current_row_ctl` reads it back).
func _wrap_slider_cell(made: Dictionary) -> Dictionary:
	var wide: HBoxContainer = made["row"]
	var slider: HSlider = made["slider"]
	var readout: Control = made["readout"]
	var label: Control = wide.get_child(0)
	## `DccWidgets.slider()`'s own spacer, between the label and the track --
	## see that function's body. Kept alive off-tree while compact is mounted
	## so `_mount_wide` restores it rather than needing a second copy of
	## `DccTheme.spacer()`'s construction.
	var spacer: Control = wide.get_child(1)

	var compact := VBoxContainer.new()
	compact.add_theme_constant_override("separation", 2)
	compact.tooltip_text = wide.tooltip_text

	var line1 := HBoxContainer.new()
	line1.add_theme_constant_override("separation", 8)
	compact.add_child(line1)

	var line2 := MarginContainer.new()
	## The canvas's own touch row for a slider track -- the same role
	## `_row()` itself uses for a whole row's height, reused here for the
	## line that now carries just the track.
	line2.custom_minimum_size.y = DccTheme.role_px("row_min_h")
	compact.add_child(line2)

	var entry := {
		"host": wide.get_parent(), "wide": wide, "compact": compact,
		"label": label, "slider": slider, "readout": readout, "spacer": spacer,
		"line1": line1, "line2": line2, "mounted": "wide",
		## Captured before either shape ever mutates them, so `_mount_wide`
		## restores the EXACT flags/width `DccWidgets.slider()` built rather
		## than a second, hand-copied guess at what they were.
		"label_wide_flags": label.size_flags_horizontal,
		"slider_wide_flags": slider.size_flags_horizontal,
		"slider_wide_min_x": slider.custom_minimum_size.x,
		"readout_wide_min_x": readout.custom_minimum_size.x,
	}
	_slider_reflow_entries.append(entry)
	if DccTheme.is_tablet_portrait():
		_mount_compact(entry)
	return entry

## Moves `entry`'s three live controls into the compact two-line cell and
## swaps which wrapper sits in `host`. A no-op if compact is already mounted,
## so `_on_portrait_changed` need not check first.
func _mount_compact(entry: Dictionary) -> void:
	if entry["mounted"] == "compact":
		return
	var host: Control = entry["host"]
	var wide: HBoxContainer = entry["wide"]
	var label: Control = entry["label"]
	var slider: HSlider = entry["slider"]
	var readout: Control = entry["readout"]
	var spacer: Control = entry["spacer"]
	var idx := wide.get_index()

	wide.remove_child(label)
	wide.remove_child(spacer)
	wide.remove_child(slider)
	wide.remove_child(readout)
	host.remove_child(wide)

	## Keeps its `ROW_LABEL_W` floor (untouched) and now grows to push the
	## readout to the line's right edge, matching the canvas's `flex:1` label
	## -- there is no spacer in this shape to give that slack to instead.
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	(entry["line1"] as HBoxContainer).add_child(label)
	## `ROW_VALUE_W` (44) is a fine floor for "0.30" but not for every unit
	## `params.rs` ships -- "6.5°C/km" (Lapse rate) measures 63 px unclipped,
	## and was this dock's own widest driver in Climate (`§ CLIMATE &
	## TEMPERATURE`, measured by this lane's own diagnostic walk) once every
	## slider cell was otherwise compact. Cleared to 0 -- sized to its own
	## text, exactly like the canvas's own `{{ f.disp }}` span, which carries
	## no width rule of its own at all -- rather than raised to a second fixed
	## figure that some longer future unit could exceed again. `readout` never
	## gains `clip_text`: eliding a NUMBER (unlike a name or a note) can make
	## it read as a different value, so this cell would rather be a few
	## pixels wider on the rare long unit than silently misreport one.
	readout.custom_minimum_size.x = 0
	(entry["line1"] as HBoxContainer).add_child(readout)

	## Was `SIZE_SHRINK_END` at a fixed tablet width (90 px): the compact cell
	## has no spacer to absorb slack, so the track takes the line's full width
	## instead. Height (14) is untouched.
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slider.custom_minimum_size.x = 0
	(entry["line2"] as MarginContainer).add_child(slider)

	host.add_child(entry["compact"])
	host.move_child(entry["compact"], idx)
	entry["mounted"] = "compact"

## The reverse of `_mount_compact` -- symmetric, not a rebuild: `wide` is
## still the exact `HBoxContainer` `DccWidgets.slider()` built, so putting the
## three children back in their original order (plus the spacer that gives
## the slider its slack) is the whole job. A no-op if wide is already mounted.
func _mount_wide(entry: Dictionary) -> void:
	if entry["mounted"] == "wide":
		return
	var host: Control = entry["host"]
	var compact: Control = entry["compact"]
	var label: Control = entry["label"]
	var slider: HSlider = entry["slider"]
	var readout: Control = entry["readout"]
	var spacer: Control = entry["spacer"]
	var idx := compact.get_index()

	(entry["line1"] as HBoxContainer).remove_child(label)
	(entry["line1"] as HBoxContainer).remove_child(readout)
	(entry["line2"] as MarginContainer).remove_child(slider)
	host.remove_child(compact)

	label.size_flags_horizontal = int(entry["label_wide_flags"])
	var wide: HBoxContainer = entry["wide"]
	wide.add_child(label)
	wide.add_child(spacer)
	slider.size_flags_horizontal = int(entry["slider_wide_flags"])
	slider.custom_minimum_size.x = float(entry["slider_wide_min_x"])
	wide.add_child(slider)
	## Restores `ROW_VALUE_W` (44) -- cleared to 0 by `_mount_compact` so a
	## long unit's readout is never truncated; captured, not re-read from the
	## constant, so a future caller-side override to that constant is still
	## honoured on the way back.
	readout.custom_minimum_size.x = float(entry["readout_wide_min_x"])
	wide.add_child(readout)

	host.add_child(wide)
	host.move_child(wide, idx)
	entry["mounted"] = "wide"

## `_ws_override_rows`' reader and `_build_param_row`'s own build-time dimmer
## -- resolves whichever shape a row's stored reference can be to the Control
## actually mounted right now. A plain `Control` (every non-tablet geometry:
## `row_ref` is `made["row"]` itself, never wrapped -- see `_build_param_row`)
## is returned as-is; a tablet row's reflow entry (`_wrap_slider_cell`'s own
## Dictionary) resolves through whichever of "wide"/"compact"
## `_mount_wide`/`_mount_compact` last mounted, so a caller never holds a
## reference stale across a rotation.
func _current_row_ctl(row_ref) -> Control:
	if row_ref is Dictionary:
		return (row_ref["wide"] if row_ref["mounted"] == "wide" else row_ref["compact"]) as Control
	return row_ref as Control

## `DccTheme.watch_portrait`'s callback (registered once, in `_build()`).
## Fires only on a real axis flip -- `DccTheme.set_portrait`'s own guard skips
## a same-orientation resize, so this costs nothing then either.
##
## Reflows every tablet slider row to the shape the NEW orientation wants,
## then re-derives World-Structure dimming once for the whole dock:
## `_refresh_ws_override_rows` reads `_ws_override_rows` fresh through
## `_current_row_ctl` each time, so a row whose wrapper identity just changed
## is not left holding a stale `modulate` from whichever shape it was dimmed
## in before the flip.
##
## Also re-derives the GENERATE stage-name labels' own clip, lane GRID
## 2026-09-13: `name_label.clip_text` below is set once, at construction, from
## whatever `is_tablet_portrait()` read during this panel's one-time
## `_build()` -- exactly `DccTheme.header()`'s own disclosed gap (that
## function's `_elide_labels` doc), reproduced here the same way, with
## `_worldportraitgrid_probe.gd --rotate --boot-landscape`: a landscape BOOT
## rotated into portrait left every stage name un-elided, because nothing
## revisited it after `_build_generate_head` ran. `_stage_name_labels` is
## already held for `_paint_stage_rows()`'s own per-state recolouring, so no
## new registry is needed -- just one more thing this existing hook reflows.
func _on_portrait_changed() -> void:
	var want_compact := DccTheme.is_tablet_portrait()
	for entry in _slider_reflow_entries:
		if want_compact:
			_mount_compact(entry)
		else:
			_mount_wide(entry)
	_refresh_ws_override_rows()
	for nl in _stage_name_labels:
		var lbl := nl as Label
		if lbl == null:
			continue
		lbl.clip_text = want_compact
		lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS if want_compact \
			else TextServer.OVERRUN_NO_TRIMMING

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
	## `_wrap_slider_cell`'s own header: tablet only (both orientations), so a
	## later rotation can reflow this row -- `row_ref` is either the plain
	## `HBoxContainer` `DccWidgets.slider()` returned (every non-tablet
	## geometry, completely untouched below) or the reflow entry
	## `_wrap_slider_cell` made; `_current_row_ctl` is the one place that
	## tells the two apart, and every later reference to "the row" (dimming,
	## tooltip) goes through it from here on instead of `made["row"]`.
	var row_ref = made["row"]
	if DccTheme.is_tablet():
		row_ref = _wrap_slider_cell(made)
	if WS_OVERRIDDEN_KEYS.has(key):
		## Held for `_refresh_ws_override_rows`; see those dictionaries' own doc.
		_ws_override_sliders[key] = s
		_ws_override_rows[key] = row_ref
		if ws_overridden:
			s.editable = false
			## The WHOLE row, not just the slider -- label and readout dim
			## too, matching `_mark_inert`'s own reasoning (this block's doc
			## comment on `WS_OVERRIDE_DIM`): a dimmed slider next to a
			## full-brightness number and name would still read as live.
			_current_row_ctl(row_ref).modulate = Color(1.0, 1.0, 1.0, WS_OVERRIDE_DIM)
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
		var revert := func():
			if not s.editable:
				return
			if is_equal_approx(s.value, float(reset_to)):
				return
			s.value = float(reset_to)
			_on_float_row_released(key, is_int, stage_index)
		## A tablet row's label lives in a DIFFERENT immediate container
		## depending on which shape is currently mounted (`wide` itself in the
		## landscape shape, `line1` in the portrait one -- see
		## `_wrap_slider_cell`), and a rotation can remount it at any time
		## after this call returns. `s` itself needs no such list: it never
		## changes identity, only its parent, and is wired directly below
		## regardless. `wide`/`line1` are wired here EVEN WHEN the other one is
		## currently mounted -- an off-tree Control receives no input at all,
		## so wiring the not-yet-visible shape ahead of time costs nothing and
		## is what makes a LATER rotation still land on a wired container
		## instead of silently missing the label side, which is what moving
		## the gesture onto the track alone did before this rewrite.
		if row_ref is Dictionary:
			_wire_row_reset(s, revert, [row_ref["wide"], row_ref["line1"]])
		else:
			_wire_row_reset(s, revert)

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
##
## `extra`, added lane GRID 2026-09-13: explicit "also wire these" list for a
## control whose row-level container cannot be derived from `control.get_parent()`
## alone -- a tablet slider's label can sit in either of two wrapper
## containers depending on the current orientation (see the one caller that
## passes this, `_build_param_row`). Omitted (every other caller: the bool
## toggle, and a non-tablet float row), behaviour is exactly what it was
## before this parameter existed -- `control.get_parent()` is still the only
## thing wired beside `control` itself.
func _wire_row_reset(control: Control, revert: Callable, extra: Array = []) -> void:
	var on_input := func(event: InputEvent):
		var mb := event as InputEventMouseButton
		if mb != null and mb.pressed and mb.button_index == MOUSE_BUTTON_RIGHT:
			revert.call()
	control.gui_input.connect(on_input)
	var targets: Array = extra.duplicate()
	if targets.is_empty():
		var row := control.get_parent() as Control
		if row != null:
			targets.append(row)
	for t in targets:
		(t as Control).gui_input.connect(on_input)

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
		## Resolved through `_current_row_ctl`, not derived by walking up from
		## `s` (which is only correct while the slider lives directly inside
		## the Control being dimmed) and not a bare stored Control either: a
		## tablet row's wrapper identity can have changed since the LAST call
		## here, if a rotation happened in between (`_on_portrait_changed`
		## calls this once after reflowing every row for exactly that reason).
		if not _ws_override_rows.has(key):
			continue
		var row := _current_row_ctl(_ws_override_rows[key])
		if row != null and is_instance_valid(row):
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
			var gap_note := "  (no dials of its own -- see gap note above)" \
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
	push_dock_readout()

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
##
## **The domain gate is the point, not a precaution.** This runs from
## `_paint_stage_rows()`, which is driven by `bridge.generation_stage` -- a
## signal this workspace stays subscribed to whether or not WORLD is the domain
## on screen. Without the gate a generate finishing while the reader is in
## CIVIL writes `resolved` into a dock that has no pipeline in it, which is the
## same defect `Workspace.push_dock_readout()` exists to fix, arriving by a
## different route. `Workspace` names this an override; the base takes over the
## moment the reader leaves.
func push_dock_readout() -> void:
	if app == null or app.active_domain() != "world":
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
# brush/noise globals -- §5.2's eight plus this port's own `falloff`, nine in
# all) comes straight off `bridge.get_sculpt_features()`
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
## controls (`#sBrush`…`#sSeed`) come from `get_sculpt_globals_info()`; the
## seed row is its own thing since a dice button has no `Control` entry.
##
## §5.2's table has **eight** rows and the engine now reports **nine**: the
## ninth, `falloff`, is this port's own addition and has no §5.2 counterpart
## to have drifted from (`sculpt_bridge.rs`'s `GLOBAL_RANGES` says so at the
## source). It arrives with `type == "enum"` and an `options` array, so it
## draws as a dropdown of names rather than as a 0..3 slider -- an ordinal
## is what the engine stores, never what a user should be asked to aim at.
## The branch is on `has("options")`, not on the key, so a second enum
## control needs no edit here.
##
## The grid-snap picker rides in the same section and is deliberately *not*
## one of those rows: snapping is applied at capture and never reaches a
## stamp, so it is tool state rather than a brush global (`SculptEditor`'s
## own `grid_snap` doc comment carries the reasoning).
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
		if cd.has("options"):
			var opts: Array = Array(cd["options"])
			var sel := clampi(int(round(cval)), 0, maxi(0, opts.size() - 1))
			DccWidgets.choice(sec, clabel, opts, sel, _on_global_enum_changed.bind(key),
				_falloff_tooltip() if key == "falloff" else "")
		else:
			DccWidgets.slider(sec, clabel, cmin, cmax, cstep, cval, unit, _on_global_changed.bind(key, is_int))
	_build_grid_snap(sec)

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

## An enum global's dropdown index **is** the value the engine stores, so
## this passes it straight through rather than mapping through a name: the
## `options` array came from the same ordinal the engine reads back.
func _on_global_enum_changed(i: int, key: String) -> void:
	bridge.sculpt_set_globals({key: float(i)})

## The one thing a user could reasonably expect the Falloff control to do
## and it will not: **Ridge ignores it.** Measured, not guessed -- Ridge
## multiplies coverage by its own perpendicular gaussian, which at the
## default `Width frac` has decayed to ~0.002 before the brush ramp starts
## to bend, so all four shapes land within 2e-5 of full height of each
## other (`sculpt.rs`'s
## `ridge_is_immune_to_the_falloff_because_its_own_gaussian_is_narrower`).
## Said here rather than left for the user to discover by drawing four
## identical ridges.
func _falloff_tooltip() -> String:
	return ("Shape of the brush's coverage ramp. Hardness sets how WIDE the ramp is; "
		+ "this sets its shape, which hardness cannot reach at any setting. "
		+ "Constant has no ramp at all, so edge noise cannot roughen it. "
		+ "Ridge is the one feature this barely affects -- its own Width frac "
		+ "gaussian is narrower than the ramp and decides the profile instead.")

## Grid snap: Off plus whatever ladder the engine offers, in grid cells.
##
## Off is the **first** entry and means the absence of a step, not a step of
## zero -- `sculpt_get_grid_snap()` returns an empty Dictionary rather than a
## `0`, and this reads it with `has("step")` for exactly that reason.
func _build_grid_snap(sec: Control) -> void:
	var steps := bridge.get_sculpt_grid_snap_steps()
	if steps.is_empty():
		return
	var live := bridge.sculpt_get_grid_snap()
	var options: Array = ["Off"]
	var selected := 0
	for i in steps.size():
		var step := float(steps[i])
		options.append("%d cell%s" % [int(step), "" if int(step) == 1 else "s"])
		if live.has("step") and float(live["step"]) == step:
			selected = i + 1
	DccWidgets.choice(sec, "Grid snap", options, selected, _on_grid_snap_changed.bind(steps),
		"Rounds each captured stroke point onto a lattice of this many grid cells. "
		+ "Applies to points captured after the change, not to the stroke in progress "
		+ "or to stamps already on the draft.")

func _on_grid_snap_changed(i: int, steps: PackedFloat64Array) -> void:
	## Index 0 is Off, so `i - 1` indexes the ladder. A 0.0 step is what the
	## engine reads as "off"; it is never stored as a snap value.
	bridge.sculpt_set_grid_snap(0.0 if i <= 0 else float(steps[i - 1]))

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
##
## **Re-verified 2026-09-08 (falloff-control audit).** `_build_brush_globals`'s
## own `falloff` dropdown, built since this note was last worded, IS §5.5's
## segmented control -- same four names (`Smooth`/`Linear`/`Sharp`/`Constant`),
## same `Smooth` default -- backed by `sculpt.rs`'s `Falloff` enum and read by
## `SculptStamp::apply_into` at three sites (`g.falloff.coverage(t)`). The
## note below used to say plain "custom Falloff", which read as the whole
## control still being unbacked; it now names only the hand-drawn curve
## beyond those four presets, which really has no engine behind it --
## `Falloff`'s own doc comment argues against ever adding one (Blender 5
## converted hand-authored curves back to its built-in Smooth preset).
func _build_sculpt_unbuilt_note(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	DccWidgets.note(sec,
		"Brush shape (8 shape chips, Import brush, a hand-drawn falloff curve), Stroke & grid " +
		"(Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align control-point " +
		"editing) and Actions (Flip X/Y, Rot Left/Right, Flatten) have no engine behind them. " +
		"Falloff's four named presets (Smooth/Linear/Sharp/Constant) are not on this list -- " +
		"they are built and live as their own dropdown in Brush & noise · global above; only a " +
		"hand-drawn curve beyond those four remains unbuilt. " +
		"The design's own prototype mocks the rest too (04-left-dock.md §5.5-§5.6: every one of " +
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
	## The line above is not enough on its own once the deep-zoom pyramid is
	## up: an already-built LOD tile keeps its OWN synthesized texture and a
	## shader reference to the OLD `map_view.texture`, and nothing about
	## reassigning that field tells `ViewportHost` to rebuild a tile it
	## already has (`OUTSTANDING_WORK.md`, "the in-session tile cache is not
	## invalidated by a sculpt"). `invalidate_lod_tiles()` is the same
	## camera-preserving trade as the line above -- it does not call
	## `reset_view()` either -- so Commit still never moves the camera.
	app.viewport.invalidate_lod_tiles()
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

## The raw pointer position goes to `sculpt_add_point`, which does its own
## grid snapping in f64; the **preview** point comes back from
## `bridge.sculpt_snap()` so the teal polyline draws where the stamp will
## actually land. With snapping off the two are the same point, and this
## costs one extra engine call per motion sample.
##
## The order matters: `sculpt_add_point` first, then the readback. Snapping
## is a pure function of the live step, so a readback taken before the push
## would agree today -- but every other pair in this file emits after the
## engine call for the reason `MISTAKES.md` records, and a preview drawn
## from a value read before its own write is that mistake waiting for the
## first stateful snap mode.
func _sculpt_click(gx: float, gy: float) -> void:
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gx, gy)
	_sculpt_stroke_points = PackedVector2Array([bridge.sculpt_snap(gx, gy)])
	app.viewport.tool_overlay.set_path_preview(_sculpt_stroke_points)

func _sculpt_drag(gx: float, gy: float) -> void:
	if _sculpt_stroke_points.is_empty():
		bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gx, gy)
	_sculpt_stroke_points.append(bridge.sculpt_snap(gx, gy))
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
	## **No `_sculpt_body.visible` write here any more, 2026-09-21.** It carried
	## the 2026-09-07 armed-tool gate (*"a single equality covers both edges"*),
	## and Ruling L replaces that mechanism with SCULPT mode itself -- see
	## `_build()`. `_follow_tool_to_its_block()` at the foot of this function is
	## what makes arming still land the user on the controls: it switches the
	## mode and opens the category, rather than revealing a body in place.
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
	_follow_tool_to_its_block(id)

## **`armTool`'s navigation half** (`04-left-dock.md` §2.4, and §4c of
## `02-rail-and-domains.md` which lists it among the writers of `domain`/`mode`):
## *"If id is `sculpt` or `freehand`, it also forces `domain: 'WORLD'` and
## `worldMode: 'b'` -- arming a sculpt tool from anywhere jumps the dock to
## WORLD·b."* Not reproduced until 2026-09-05, because until then there was
## nothing to jump to: WORLD's nine categories were all on screen in both modes,
## so arming Sculpt from CIVIL left the sculpt controls one domain click away
## and no mode was wrong.
##
## `RAIL_NODES`' `shows` gate changes that. `world/b` now renders `Terrain` and
## hides the other eight, so arming Sculpt and then walking to WORLD by hand
## would land on whichever mode was last active -- the pipeline, in the default
## state -- with the brush armed over a dock that does not show a brush. This is
## the transition INTO the gated state that the gate creates, and it is closed
## here rather than left as a corner.
##
## **Two tools, two destinations, each derived from where its controls actually
## are** rather than from the prototype's tool names. `_sculpt_body` is parented
## into the `Terrain` category and `_paint_body` into `Biomes` (`_build_categories()`),
## so Sculpt goes to the mode that renders `Terrain` and Biome paint to the mode
## that renders `Biomes` -- `mode_for_category()` is asked, so neither letter is
## written down twice.
##
## The prototype's third id, `freehand`, needs no arm of its own here: in this
## port Freehand is a sculpt **feature**, not a tool, and both routes to it
## (`tool_bar.gd`'s pill and this file's own feature picker) call
## `sculpt_set_feature("freehand")` and then `arm_tool("sculpt")` -- so it
## already arrives through the `sculpt` arm above, which is exactly where §2.4
## sends it.
func _follow_tool_to_its_block(id: String) -> void:
	var category := ""
	match id:
		"sculpt": category = "Terrain"
		"paint": category = "Biomes"
		_: return
	if DccShell.mode_for_category("world", category).is_empty():
		return
	## `select_domain_category`, not `select_domain_mode`: the mode is the half
	## that makes the block *visible*, and the category is the half that makes it
	## *open*. `select_domain_mode("world", "a")` opens that node's own category
	## (`Generate`) and would leave Biome paint armed over a closed `Biomes`
	## accordion — the tool's controls rendered, in a body nobody expanded.
	## `select_domain_category()` derives the same mode through
	## `mode_for_category()` and then opens the category that actually holds the
	## brush, so both halves land. It forces the domain too, which is the third
	## thing §2.4's `armTool` asks for.
	app.select_domain_category("world", category)

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
	## And the third line `_on_sculpt_commit` uses, for the same reason: a
	## live LOD tile keeps its own captured `base_tex` from the OLD texture,
	## so a paint commit at a fixed zoom would otherwise leave pre-paint
	## relief on screen (`OUTSTANDING_WORK.md`, "the in-session tile cache is
	## not invalidated by a sculpt" -- the row also covers paint).
	app.viewport.invalidate_lod_tiles()
	app.viewport.set_preview_texture(null)
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	_build_paint(_paint_body)
	_rebuild_tool_bar()
	_refresh_right_dock_paint()

func _on_paint_discard() -> void:
	bridge.paint_discard()
	## `true`: the committed layer that survives a discard is exactly the base
	## the next dab's window composites onto, so marking it here is what keeps
	## the first dab after a discard on the bounded path.
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture(), true)
	_build_paint(_paint_body)
	_rebuild_tool_bar()
	_refresh_right_dock_paint()

## The other half of the WW-13 cross-refresh -- see `rebuild_paint_panel()`.
func _rebuild_tool_bar() -> void:
	if app.tool_bar != null and app.tool_bar.has_method("rebuild"):
		app.tool_bar.rebuild()

## `tool_bar.gd` draws a **second Commit / Discard pair** for the same draft --
## its Discard landed 2026-09-05, from §2.2.6 -- and both pairs are on screen
## together whenever Biome paint is armed. Committing or discarding from either
## one has to refresh the other, or the loser keeps a live button over an empty
## draft, which is WW-13's own defect wearing a different hat.
func rebuild_paint_panel() -> void:
	if _paint_body != null:
		_build_paint(_paint_body)

# -- §4.5.2 Biome paint: stroke capture (map_clicked/map_dragged/map_released) -
#
# Paint has no begin/end pair (`paint_stroke_at`'s own doc: every call is
## already one complete, independently undo-able draft entry), so click and
# drag both just apply one dab; release only refreshes the panel (painted
# counts, Commit's disabled state) once per gesture rather than once per
# motion sample, since the panel rebuild is not cheap enough to do on every dab.
#
# **Corrected 2026-09-04.** This used to end "...the way the live preview
# texture already is", asserting the preview IS cheap per dab. Measured, it is
# not: a full-grid CPU rebuild costs **0.73 / 1.48 / 4.55 / 16.80 ms** per dab
# at 512/1024/2048/4096 squared, re-uploading 1 MB to 64 MB across the FFI each
# time, while `touched_bounds` covers only **1.80%** of the grid at 2048 and
# **1.32%** at 4096. So the deferral above is right and its stated reason was
# comparing against something that is not cheap either.
#
# **Wired 2026-09-04, later the same day**, so the paragraph above is now the
# history of this call site rather than its description: `_paint_apply_dab`
# below takes the bounded `build_paint_preview_patch()` and composites it onto
# the raster already on screen (`ViewportHost.set_preview_patch()`, whose own
# doc carries the shell-side measurement). The panel rebuild is still deferred
# to release, and still for its own reason -- it is a Control tree, not a
# raster, and nothing about the preview getting cheaper makes rebuilding it
# per motion sample right.

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
	_paint_show_preview()

## The bounded upload, with the full raster as its fallback rather than as its
## default. `set_preview_patch()` returns `false` for exactly one reason that
## can happen in ordinary use -- there is no raster on screen yet to composite
## onto, which is the first dab after a Commit -- and `true` for the empty
## Dictionary, which is the engine saying there is nothing to draw at all and
## is a state this must not turn into a full re-upload of nothing.
##
## The `true` on the fallback is what lets the *next* dab be a patch: it marks
## the texture just set as the base a window may be composited onto. Sculpt
## shares `_preview_layer`, and its rasters are the same size but a DIFFERENT
## format -- `build_sculpt_preview_texture` builds `Format::RGB8` (`lib.rs:8334`)
## where paint builds `Format::RGBA8` (`:9284`). The first version of this
## comment said "same size and format" and a verifier refuted it. The flag is
## therefore load-bearing for a STRONGER reason than that sentence gave: an
## inferred base could hand `blit_rect` a format it silently drops, so
## that flag is opt-in rather than inferred.
func _paint_show_preview() -> void:
	if app.viewport.set_preview_patch(bridge.build_paint_preview_patch()):
		return
	app.viewport.set_preview_texture(bridge.build_paint_preview_texture(), true)

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


# =============================================================================
# The phone GENERATE sheet -- `tabIsGen`
# =============================================================================
#
# `design/Cartalith-Android-2026-09-07.dc.html`, template bytes 26 332-39 559,
# state and handlers 104 500-112 000. Every metric below is transcribed in
# `ANDROID_UI_SPEC.md` §1, which is where the provenance lives; this file
# carries only the reasons a value here differs from the one drawn there.
#
# **Why this exists at all.** Owner, 2026-09-07, using the APK on a OnePlus 6T:
# *"I don't have sliders or fields to change parameters on how the world is
# generated. Nothing from map, generate, plan or more really leads to a deeper
# menu."* Confirmed at the symbol before building: `DccShell._pick_phone_tab()`
# routes GENERATE to `_pick_bar_domain("world")` -> `_select_domain("world")`,
# which decides what the LEFT DOCK would show; on a phone that dock is
# `left_dock` with `visible = false`, and the only two things that open it are
# `DccApp.toggle_region(ID_WIN_LEFT)` and `PhoneMenu._open_left_sheet()`
# (`grep -rn "_set_sheet_open" --include=*.gd`, 2026-09-07: seven call sites,
# none of them the bottom bar). The tab lifted the tool-options strip -- one
# `HBoxContainer` in a scroller -- and every generation parameter stayed behind
# a sheet the bottom bar cannot open. That is the "one line that barely scrolls
# properly and a lot of white space".
#
# **One parameter source, not two.** Every row here is built from
# `bridge.param_keys()` / `param_info()` / `param_get()` / `param_set()`,
# grouped by the identical predicate `_build_group_section()` uses -- match
# `info.group` against `STAGES[i]["groups"]`, then append `STAGES[i]["keys"]`.
# No range, step, label, unit or default is copied into this block. A parameter
# added to `crates/cartalith-godot/src/params.rs` appears here with no GDScript
# change, which is that file's own stated contract.

## Stage index -> the canvas fields that stage draws and **this engine cannot
## back**, with the reason each one is actually missing.
##
## Every entry was opened at its symbol before it was written, per
## `MISTAKES.md`'s "Dash a field with a reason" row -- a wrong reason reads as
## freshly checked and routes the next brief at the wrong subsystem.
##
## `route` is `"new_world"` for the two that are not missing at all but live on
## a different surface: those draw as a tappable row that opens File > New
## world rather than as a dead dash.
##
## **Re-audited on the phone, 2026-09-07, for one specific failure: a reason
## that is true of the DESKTOP and false of the handset it is printed on.**
## Archetype was exactly that -- `archetype_input` was built into
## `new_world_dialog.gd`'s `struct_sec`, which is parented to the `rest`
## container that file hides on a phone, so this row's *"Pick it in File ▸ New
## world"* sent a finger to a dialog that did not contain the control. Closed
## by lifting the control onto the phone card rather than by rewording the row
## (see that file for why that half was the right one), so the route below is
## now true on both. Three others were checked and held -- Erosion strength's
## *"the real dials are below"* (`STAGES[5]`'s `erosion` group is built into
## this same sheet by `_pg_fill_group`), Ecotone sharpness (`params.rs` has no
## `ecology` group: `grep -o 'group: "[a-z_]*"' params.rs | sort -u` gives
## civ / climate / erosion / planet / tectonics / volcanism / weather / world /
## world_structure and nothing else), and CANCEL below -- and **two failed for a
## different reason than the one hunted**: Min stream order and Rivers in biome
## view both named Cartography as their home, and Cartography does not have
## them either, on any density. Their reasons now say where each is actually
## missing from, which is the drawing side.
const PHONE_GEN_ABSENT: Array = [
	{"stage": 1, "label": "Working resolution", "route": "new_world",
	 "why": "Resolution is a creation-time call argument, not a stored parameter -- params.rs' \"world\" group holds world, sea_level, peak_m, carve_rivers, river_density, integrate_drainage and use_gpu, and no resolution key exists anywhere in the 93-row table. Set it in File > New world, which carries it on this phone's card as well as on the desktop form."},
	{"stage": 2, "label": "Archetype", "route": "new_world",
	 "why": "apply_archetype() is live and seeds the six world_structure dials below, but request()[\"archetype\"] is what decides which generation call runs, and new_world_dialog.gd's own NOTE_CREATION_ONLY says extent, resolution and archetype reallocate every field in the pipeline. Pick it in File > New world -- on this phone it is on that dialog's card, under World structure."},
	{"stage": 5, "label": "Erosion strength", "route": "",
	 "why": "No engine parameter means this. Stage 06 exposes 28 rows (stream.* and passes.*) and none of them is a single 0-1 strength; synthesising one over several would be a second parameter table that can drift from the desktop's. The real dials are below."},
	{"stage": 6, "label": "Min stream order", "route": "",
	 "why": "A filter over a vector river layer this port does not draw yet, so there is nowhere to set it -- not here, and not in Cartography either, which this row claimed until 2026-09-07. Strahler order is real (get_rivers(min_order) returns every traced run, and the right dock's River context picks one by it), but the rivers you can SEE are a flow-area tint inside the terrain raster (render.rs, WET_AREA_LO/HI over upstream drainage area), which carries no order to filter on. drawRiverWays -- the overlay that would -- is the one thing render.rs's module doc still lists as excluded."},
	{"stage": 8, "label": "Ecotone sharpness", "route": "",
	 "why": "Ecology is not parameterised in cartalith-engine: biome classification runs off the finished elevation/temperature/rainfall fields with no dials of its own."},
	{"stage": 8, "label": "Rivers in biome view", "route": "",
	 "why": "A render toggle with nothing behind it to toggle, here or anywhere: the biome view's rivers are the same flow-area tint baked into the terrain raster (render.rs, WET_AREA_LO/HI), and no parameter switches it off. This row said it was a Cartography layer option until 2026-09-07; cartography_workspace.gd's own note says the opposite, and is the one that is right."},
]

## Which stage index each group header opens at. The canvas ships `g1:true` and
## the rest closed (byte 77 286); index 0 is that same first group.
var _pg_open: Dictionary = {0: true}
var _pg_host: VBoxContainer          ## The sheet column this workspace fills.
var _pg_mode := "pipe"               ## `pipe` | `sculpt` -- the canvas's genMode.
var _pg_connected := false
var _pg_stage_index := -1            ## Live stage while `bridge.generating`.

## The sheet's own live nodes, so a progress tick repaints instead of rebuilding
## the whole column under the user's finger.
var _pg_prog_title: Label
var _pg_prog_pct: Label
var _pg_prog_fill: Control
var _pg_prog_track: Control
var _pg_log_label: Label
## Everything a parameter write has to repaint without rebuilding the column
## under the user's finger: one entry per group (`{index, num, state}`), the
## Generate button whose caption carries the stale range, and the stale note.
var _pg_stale_marks: Array = []
var _pg_gen_button: Button
var _pg_stale_label: Label

# -- Metrics ------------------------------------------------------------------
#
# `app` is a `DccApp`, which `extends DccShell`, so these three resolve
# statically. Boxes go through `_pscale`/`_ptap` and type through `_pfont`,
# which is `dcc_shell.gd`'s own split: a tap target is a finger measurement and
# does not grow when the OS text size does.

func _pg_px(px: float) -> int:
	return app._pscale(px) if app != null else int(round(px))

func _pg_tap(px: float) -> int:
	return app._ptap(px) if app != null else int(round(maxf(DccTheme.PHONE_TAP_MIN, px)))

func _pg_fs(px: float) -> int:
	return app._pfont(px) if app != null else int(round(px))

## `--chip: rgba(255,255,255,.05)`. Built off `text_bright` rather than written
## as a white literal so `DccTheme.remap()` can trace it: on the light palette
## `text_bright` is `#111210`, so the same expression gives a 5% BLACK wash,
## which is what a light theme wants. A literal `Color(1,1,1,.05)` would stay
## white and disappear.
func _pg_chip() -> Color:
	return Color(DccTheme.c("text_bright"), 0.05)

## `--warn: #e0a840`, drawn as `accent` (`#e0a34a`). Five and six units apart on
## two channels -- indistinguishable at 9.5 px -- and a token can be remapped
## where a twelfth near-accent literal could not. Said here rather than left as
## an unexplained substitution.
func _pg_warn() -> Color:
	return DccTheme.c("accent")

func _pg_box(bg: Color, radius: int, border_col: Color = Color(0, 0, 0, 0),
		border_px: int = 0) -> StyleBoxFlat:
	var b := StyleBoxFlat.new()
	b.bg_color = bg
	b.set_corner_radius_all(radius)
	if border_px > 0:
		b.border_color = border_col
		b.set_border_width_all(border_px)
	return b

func _pg_mono(text: String, token: String, px: float, spacing: int = 0,
		medium: bool = false) -> Label:
	var l := DccTheme.mono_label(text, token, _pg_fs(px), spacing, medium)
	return l

# -- Entry points -------------------------------------------------------------

## Called once by `DccShell._build_phone_gen_panel()` with the column it owns.
## Everything after that is `refresh_phone_generate()`.
func build_phone_generate(host: VBoxContainer) -> void:
	_pg_host = host
	if not _pg_connected:
		_pg_connected = true
		bridge.generation_started.connect(_pg_on_started)
		bridge.generation_stage.connect(_pg_on_stage)
		bridge.generation_finished.connect(_pg_on_finished)
		bridge.world_loaded.connect(func(): _pg_rebuild())
	_pg_rebuild()

## The shell calls this every time the GENERATE tab is tapped. A sheet that was
## built against a world that has since been regenerated, loaded or had its
## seed rolled would otherwise show the values it was built with.
func refresh_phone_generate() -> void:
	_pg_rebuild()

func _pg_on_started() -> void:
	_pg_stage_index = 0
	_pg_rebuild()

func _pg_on_stage(index: int, _name: String, _total: int) -> void:
	_pg_stage_index = index
	_pg_paint_progress()

func _pg_on_finished(_ok: bool) -> void:
	_pg_stage_index = -1
	_pg_rebuild()

# -- The column ---------------------------------------------------------------

func _pg_rebuild() -> void:
	if _pg_host == null or not is_instance_valid(_pg_host):
		return
	for child in _pg_host.get_children():
		_pg_host.remove_child(child)
		child.queue_free()
	_pg_prog_title = null
	_pg_prog_pct = null
	_pg_prog_fill = null
	_pg_prog_track = null
	_pg_log_label = null
	_pg_stale_marks.clear()
	_pg_gen_button = null
	_pg_stale_label = null
	_pg_host.add_theme_constant_override("separation", _pg_px(12))

	_pg_mode_segment(_pg_host)
	if _pg_mode == "pipe":
		_pg_seed_row(_pg_host)
		if bridge.generating:
			_pg_progress_card(_pg_host)
		else:
			_pg_idle_block(_pg_host)
		for i in STAGES.size():
			_pg_group(_pg_host, i)
		_pg_footnote(_pg_host)
	else:
		_pg_sculpt(_pg_host)
	_pg_open_gestures(_pg_host)

## **Let the finger drag reach the scroller.**
##
## Measured on the handset (`9608b26b`, ONEPLUS_A6013) twice, because the first
## fix was aimed at the wrong layer. A drag starting anywhere inside this column
## moved nothing; only a drag in the ~14 dp gutter either side of it scrolled,
## because that x misses every child and lands on the `ScrollContainer` itself.
## The first attempt set `MOUSE_FILTER_PASS` on the buttons and **changed
## nothing on glass** -- the buttons were never the blocker. `Control`'s default
## is `MOUSE_FILTER_STOP` and every `PanelContainer`, `MarginContainer`,
## `VBoxContainer` and `HBoxContainer` here inherits it, so the group boxes and
## their padding were eating the drag before any button saw it.
##
## So the rule is applied to the whole subtree, by kind:
##
## - **`IGNORE`** for everything inert -- labels, containers, the toggle track
##   and knob. The event passes straight through to the scroller.
## - **`PASS`** for `BaseButton`. It still takes its own taps, and the drag
##   continues up to the scroller so a fling that begins on a 50 dp group
##   header works. On a list that is mostly header rows this is most of the
##   surface.
## - **`STOP`** for `Range` (the sliders). A slider has to own its horizontal
##   drag or it cannot be set at all.
##
## That last line was the whole rule until 2026-09-07 and it is only half of
## one: **`STOP` is right for the horizontal drag and wrong for the vertical
## one**, and a vertical swipe beginning on a slider rewrote the parameter
## instead of scrolling (Ocean depth `0.60` -> `0.14` in one gesture, found on
## glass). The filter is unchanged -- the arbitration happens *inside* the
## control now, in `DccWidgets.PgSlider`, which is what every `Range` in this
## column
## actually is. Read that class before changing anything here: it needs the
## `STOP` this line sets.
##
## Reasserted on every rebuild rather than at each construction site: a single
## missed `mouse_filter` at one of two dozen sites is invisible until someone
## drags exactly there, which is how this survived the first fix.
func _pg_open_gestures(node: Node) -> void:
	for child in node.get_children():
		var c := child as Control
		if c != null:
			if c is Range:
				c.mouse_filter = Control.MOUSE_FILTER_STOP
			elif c is BaseButton:
				c.mouse_filter = Control.MOUSE_FILTER_PASS
			else:
				c.mouse_filter = Control.MOUSE_FILTER_IGNORE
		_pg_open_gestures(child)

## `PIPELINE | SCULPT`. `min-height:44px; border-radius:22px;
## font:500 10px mono; letter-spacing:.16em`.
##
## Writes `select_domain_mode("world", ...)` as well as the local flag, so the
## desktop dock and this sheet cannot disagree about which mode the WORLD
## domain is in -- the shell's `_domain_mode` is the one store for that, and
## `_refresh_mode_switch()` and the rail foot both read it.
func _pg_mode_segment(parent: Control) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pg_px(8))
	parent.add_child(row)
	for entry in [["pipe", "PIPELINE"], ["sculpt", "SCULPT"]]:
		var id := String(entry[0])
		var on: bool = _pg_mode == id
		var b := Button.new()
		b.text = String(entry[1])
		b.focus_mode = Control.FOCUS_NONE
		## **`MOUSE_FILTER_PASS`, not the `Button` default `STOP`, and the reason is a
		## measurement.** On the handset (`9608b26b`, ONEPLUS_A6013) a finger drag
		## starting anywhere on this column's controls moved nothing: the child
		## swallowed the drag and the `ScrollContainer` never saw an
		## `InputEventScreenDrag`, so a list that is mostly 50 dp header rows could
		## only be scrolled from the ~14 dp gutter at either edge. `PASS` delivers the
		## event to the control AND to its ancestors, so the button still takes taps
		## and the scroller still takes flings. Invisible to `_genphone_probe.gd`,
		## which reaches a group with `ensure_control_visible()` -- a programmatic
		## scroll that never exercises the gesture.
		b.mouse_filter = Control.MOUSE_FILTER_PASS
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.custom_minimum_size.y = _pg_tap(44)
		b.add_theme_font_override("font", DccTheme.mono(2, true))
		b.add_theme_font_size_override("font_size", _pg_fs(10))
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_secondary"))
		b.add_theme_color_override("font_hover_color",
			DccTheme.c("accent") if on else DccTheme.c("text_secondary"))
		b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
		var box := _pg_box(DccTheme.c("accent_wash") if on else Color(0, 0, 0, 0),
			_pg_px(22), DccTheme.c("accent") if on else DccTheme.c("line"), 1)
		for state in ["normal", "hover", "pressed", "focus"]:
			b.add_theme_stylebox_override(state, box)
		b.pressed.connect(_pg_set_mode.bind(id))
		row.add_child(b)

func _pg_set_mode(id: String) -> void:
	if _pg_mode == id:
		return
	_pg_mode = id
	if app != null and app.has_method("select_domain_mode"):
		## `RAIL_NODES`' own mode ids for WORLD are `a` and `b`, not the words --
		## `railFoot`'s binding is `wm==='b' ? 'SCULPT' : ...`.
		app.select_domain_mode("world", "b" if id == "sculpt" else "a")
	_pg_rebuild()

## `SEED   483102   [dice]`. `padding:10px 12px; border-radius:16px;
## background:var(--chip); gap:10px`.
##
## The seed is `new_world_dialog.request()["seed"]` -- the same value the
## desktop's Generate sends -- and the roll is `app._new_seed()`, which is
## `randomise_seed()` on that dialog. Nothing is stored here.
##
## The canvas draws the roll button `42 x 40` and its glyph as `U+2684` (die
## face five). Both are changed, and both for measured reasons. `42 x 40` is
## under the 44 dp floor on both axes, so it goes through `_ptap()`. `U+2684`
## is **absent from the shipped `IBMPlexMono-Regular.ttf`** -- checked against
## that file's own cmap on 2026-09-07, along with `U+2304` (present in the
## fallback chain, which is why `DccIcons.SYMBOLS["chevron"]` may use it) and
## `U+FF0B` (absent, which is why the stepper below draws `+` and not the
## canvas's fullwidth form). `U+21BB`, an open clockwise arrow, IS in the font
## and means re-roll, so that is what the cell carries.
func _pg_seed_row(parent: Control) -> void:
	var pc := PanelContainer.new()
	pc.add_theme_stylebox_override("panel", _pg_box(_pg_chip(), _pg_px(16)))
	parent.add_child(pc)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pg_px(12))
	pad.add_theme_constant_override("margin_right", _pg_px(12))
	pad.add_theme_constant_override("margin_top", _pg_px(10))
	pad.add_theme_constant_override("margin_bottom", _pg_px(10))
	pc.add_child(pad)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pg_px(10))
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	pad.add_child(row)
	row.add_child(_pg_mono("SEED", "text_dim", 9.5, 2))
	var seed_text := "--"
	if app != null and app.new_world_dialog != null:
		seed_text = str(int(app.new_world_dialog.request().get("seed", 0)))
	var value := _pg_mono(seed_text, "text_bright", 13, 0, true)
	value.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(value)
	var dice := Button.new()
	dice.text = "↻"
	dice.tooltip_text = "Roll a new seed. Stage 01 onward goes stale until you regenerate."
	dice.focus_mode = Control.FOCUS_NONE
	dice.mouse_filter = Control.MOUSE_FILTER_PASS
	dice.custom_minimum_size = Vector2(_pg_tap(42), _pg_tap(40))
	dice.add_theme_font_override("font", DccTheme.mono())
	dice.add_theme_font_size_override("font_size", _pg_fs(14))
	dice.add_theme_color_override("font_color", DccTheme.c("accent"))
	dice.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	dice.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	for state in ["normal", "hover", "pressed", "focus"]:
		dice.add_theme_stylebox_override(state,
			_pg_box(DccTheme.c("accent_wash"), _pg_px(14)))
	dice.pressed.connect(_pg_roll_seed)
	row.add_child(dice)

## `hDice` sets a seed **and** calls `_markStale(1)`. Both halves, in that order
## -- `MISTAKES.md`'s "Emit a change signal" row: the stale mark is written
## after the value it describes, not before it.
func _pg_roll_seed() -> void:
	if app == null:
		return
	app._new_seed()
	_mark_stale_from(0)
	_pg_rebuild()

## The running card: `border:1px solid var(--acc); border-radius:18px;
## padding:13px 14px; gap:9px`, a 5 px bar, the last three log lines, CANCEL.
func _pg_progress_card(parent: Control) -> void:
	var pc := PanelContainer.new()
	pc.add_theme_stylebox_override("panel",
		_pg_box(Color(0, 0, 0, 0), _pg_px(18), DccTheme.c("accent"), 1))
	parent.add_child(pc)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pg_px(14))
	pad.add_theme_constant_override("margin_right", _pg_px(14))
	pad.add_theme_constant_override("margin_top", _pg_px(13))
	pad.add_theme_constant_override("margin_bottom", _pg_px(13))
	pc.add_child(pad)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _pg_px(9))
	pad.add_child(col)

	var head := HBoxContainer.new()
	col.add_child(head)
	_pg_prog_title = _pg_mono("", "accent", 10.5, 2, true)
	_pg_prog_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_pg_prog_title)
	_pg_prog_pct = _pg_mono("", "text_dim", 10)
	head.add_child(_pg_prog_pct)

	_pg_prog_track = PanelContainer.new()
	(_pg_prog_track as PanelContainer).add_theme_stylebox_override("panel",
		_pg_box(_pg_chip(), _pg_px(3)))
	_pg_prog_track.custom_minimum_size.y = _pg_px(5)
	col.add_child(_pg_prog_track)
	_pg_prog_fill = PanelContainer.new()
	(_pg_prog_fill as PanelContainer).add_theme_stylebox_override("panel",
		_pg_box(DccTheme.c("accent"), _pg_px(3)))
	(_pg_prog_track as PanelContainer).add_child(_pg_prog_fill)

	_pg_log_label = _pg_mono("", "text_dim", 9.5)
	_pg_log_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(_pg_log_label)

	## CANCEL is drawn by the canvas and **cannot be backed**. `EngineBridge`
	## has `generate()` and no cancellation entry point at all -- grep for
	## `cancel` in `engine_bridge.gd` returns `sculpt_cancel_stroke` and
	## `label_cancel_edit` and nothing that stops a run (2026-09-07). The
	## prototype's own CANCEL is `clearInterval` over a simulated timer, which
	## is not a claim about this engine. Drawn dashed with that reason rather
	## than as a button that would do nothing.
	_pg_dash_row(col, "CANCEL",
		"generate() cannot be interrupted: cartalith-godot exposes no cancel entry point, so a run holds the worker until its ten stages finish.")
	_pg_paint_progress()

## Repaint only -- called on every `generation_stage` tick, so it must not
## rebuild anything the user could be touching.
func _pg_paint_progress() -> void:
	if _pg_prog_title == null or not is_instance_valid(_pg_prog_title):
		return
	var i: int = clampi(_pg_stage_index, 0, STAGES.size() - 1)
	_pg_prog_title.text = "%02d · %s" % [i + 1, String(STAGES[i]["name"])]
	## The canvas's own arithmetic is `round((i*100 + min(pct,100)) / 10)`,
	## where `pct` is a simulated within-stage percentage. This engine reports
	## stage boundaries and nothing finer (`GenerationProgress` carries a stage
	## index and a count, not a fraction), so the readout is whole stages: the
	## same formula with `pct = 0`. Stated rather than interpolated -- a smooth
	## bar over a value the engine does not produce is a fake measurement.
	_pg_prog_pct.text = "%d%%" % int(round(i * 100.0 / float(STAGES.size())))
	if is_instance_valid(_pg_prog_track):
		var w: float = _pg_prog_track.size.x * (float(i) / float(STAGES.size()))
		_pg_prog_fill.custom_minimum_size.x = maxf(0.0, w)
		_pg_prog_fill.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	if is_instance_valid(_pg_log_label):
		## The desktop's own rolling log (`_stage_log`, maintained by
		## `_log_stage()`), tailed to the canvas's three lines. Not a second
		## store: the same array both surfaces read.
		var tail: Array = _stage_log.slice(maxi(0, _stage_log.size() - 3))
		_pg_log_label.text = "\n".join(tail)

## Idle: the stale note, the big button, and the two-ended footer.
func _pg_idle_block(parent: Control) -> void:
	if _stale_from_stage >= 0:
		var pc := PanelContainer.new()
		pc.add_theme_stylebox_override("panel",
			_pg_box(Color(0, 0, 0, 0), _pg_px(14), Color(_pg_warn(), 0.4), 1))
		parent.add_child(pc)
		var pad := MarginContainer.new()
		pad.add_theme_constant_override("margin_left", _pg_px(14))
		pad.add_theme_constant_override("margin_right", _pg_px(14))
		pad.add_theme_constant_override("margin_top", _pg_px(11))
		pad.add_theme_constant_override("margin_bottom", _pg_px(11))
		pc.add_child(pad)
		## `_stale_note_text()` is the desktop's own sentence for this state.
		var note := _pg_mono(_stale_note_text(), "text", 10)
		note.add_theme_color_override("font_color", _pg_warn())
		note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		pad.add_child(note)
		_pg_stale_label = note

	var gen := Button.new()
	## `genBtnLabel`: `REGENERATE NN -> 10` only when something is stale AND a
	## run has already happened. `bridge.has_world` is this shell's form of the
	## canvas's `lastRun !== '-'`.
	gen.text = ("REGENERATE %02d → 10" % (_stale_from_stage + 1)) \
		if (_stale_from_stage >= 0 and bridge.has_world) else "GENERATE WORLD"
	gen.focus_mode = Control.FOCUS_NONE
	gen.mouse_filter = Control.MOUSE_FILTER_PASS
	gen.custom_minimum_size.y = _pg_tap(52)
	gen.add_theme_font_override("font", DccTheme.mono(2, true))
	gen.add_theme_font_size_override("font_size", _pg_fs(11))
	for key in ["font_color", "font_hover_color", "font_pressed_color"]:
		gen.add_theme_color_override(key, DccTheme.c("accent_ink"))
	for state in ["normal", "hover", "pressed", "focus"]:
		gen.add_theme_stylebox_override(state,
			_pg_box(DccTheme.c("accent"), _pg_px(26)))
	## `_regenerate_live()`, not `bridge.generate()`: it is the guarded path
	## that prompts before discarding hand-authored sculpt stamps, icons and
	## painted cells. The tool-options bar's own Generate skips that guard and
	## this one must not copy it.
	gen.pressed.connect(_regenerate_live)
	parent.add_child(gen)
	_pg_gen_button = gen

	var foot := HBoxContainer.new()
	parent.add_child(foot)
	## `last run` has no engine reading. Nothing in `EngineBridge` records a
	## wall-clock time for the last generate, so this is the one field the
	## canvas draws whose VALUE would have to be invented -- and an em dash is
	## what the canvas itself puts there before a run
	## (`gen.lastRun` starts `'-'`). Drawn as the total elapsed of the last run
	## instead, which IS measured: `_stage_elapsed_ms` is summed by the same
	## timing the ten desktop rows print.
	var elapsed := 0
	for ms in _stage_elapsed_ms:
		if int(ms) > 0:
			elapsed += int(ms)
	var left := _pg_mono(
		("last run · %.1f s" % (elapsed / 1000.0)) if elapsed > 0 else "last run · —",
		"text_faint", 9.5)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	foot.add_child(left)
	foot.add_child(_pg_mono("%d stages · dependency order" % STAGES.size(),
		"text_faint", 9.5))

## `Editing a stage marks everything downstream stale...` -- the canvas's
## closing line, rewritten where it is false here.
##
## The canvas says *"Volcanism (05) and Resources (10) run with defaults"*.
## That is true of the prototype and **false of this engine**: `params.rs`
## files nine live parameters under `group: "volcanism"`, all of which this
## sheet draws under `05`. Resources (10) genuinely has none. The GPU/LOD half
## of the sentence is true and kept.
func _pg_footnote(parent: Control) -> void:
	var text := ("Editing a stage marks everything downstream stale. This engine "
		+ "resolves all %d stages on every run -- there is no partial recompute, so "
		+ "the stale badge names where the edit landed, not where the run starts. "
		+ "Ecology (09) and Resources (10) carry no dials. GPU, LOD and render "
		+ "quality live under MORE ▸ Preferences.") % STAGES.size()
	var l := _pg_mono(text, "text_faint", 9.5)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	parent.add_child(l)

## **What a parameter write owes the rest of the sheet.**
##
## `hGenStep`/`hGenRange`/`hGenTog` all funnel through the prototype's
## `_setGenParam(key, val, stage)`, whose second act is `_markStale(stage)` --
## so an edit does not only write a value, it repaints the group numbers, the
## per-group `stale` captions, the note, and the Generate button's own range.
##
## Caught by `_genphone_probe.gd`, not by reading this back: tapping `+` on
## Plates moved `tect.plates` 14 -> 15 and left `_stale_from_stage` at **-1**,
## because the stepper closure had the key and not the stage. That is
## `MISTAKES.md`'s "covering some inputs of a thing, not all of them" -- the
## slider's release handler had it and the stepper's did not.
##
## Repaint rather than rebuild: a rebuild inside a press frees the button that
## is still delivering the press and loses the scroll position. The one case
## that genuinely needs new nodes -- the stale note appearing for the first
## time -- is deferred so it lands after the input frame.
func _pg_after_param_write(stage_index: int) -> void:
	var was_stale: bool = _stale_from_stage >= 0
	_mark_stale_from(stage_index)
	for m in _pg_stale_marks:
		var e: Dictionary = m
		var stale: bool = _stale_from_stage >= 0 and int(e["index"]) >= _stale_from_stage
		var num: Label = e["num"]
		var state: Label = e["state"]
		if is_instance_valid(num):
			num.add_theme_color_override("font_color",
				_pg_warn() if stale else DccTheme.c("text_faint"))
		if is_instance_valid(state):
			state.text = ("stale" if stale else "resolved") if bridge.has_world else "no world"
	if is_instance_valid(_pg_gen_button):
		_pg_gen_button.text = ("REGENERATE %02d → 10" % (_stale_from_stage + 1)) 			if (_stale_from_stage >= 0 and bridge.has_world) else "GENERATE WORLD"
	if is_instance_valid(_pg_stale_label):
		_pg_stale_label.text = _stale_note_text()
	elif not was_stale and _stale_from_stage >= 0:
		_pg_rebuild.call_deferred()

# -- One collapsible group ----------------------------------------------------

## `genGroups`: `border:1px solid var(--hair2); border-radius:18px`, a 50 dp
## header of `[num][name][state][chev]`, and a body separated by a `--hair2`
## rule at `padding:6px 14px 12px`.
func _pg_group(parent: Control, index: int) -> void:
	var st: Dictionary = STAGES[index]
	var stale: bool = _stale_from_stage >= 0 and index >= _stale_from_stage
	var open: bool = bool(_pg_open.get(index, false))

	var pc := PanelContainer.new()
	pc.add_theme_stylebox_override("panel",
		_pg_box(Color(0, 0, 0, 0), _pg_px(18), DccTheme.c("line_soft"), 1))
	parent.add_child(pc)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	pc.add_child(col)

	var head := Button.new()
	head.focus_mode = Control.FOCUS_NONE
	head.mouse_filter = Control.MOUSE_FILTER_PASS
	head.custom_minimum_size.y = _pg_tap(50)
	head.tooltip_text = String(st.get("produces", ""))
	for state in ["normal", "hover", "pressed", "focus"]:
		head.add_theme_stylebox_override(state, DccTheme.empty())
	head.pressed.connect(_pg_toggle_group.bind(index))
	col.add_child(head)

	## The four header elements ride inside the button rather than beside it:
	## a `Button` with four differently-coloured pieces of text cannot be one
	## string, and a row of Labels over the button would eat the press unless
	## every one of them is `MOUSE_FILTER_IGNORE`.
	var hrow := HBoxContainer.new()
	hrow.set_anchors_preset(Control.PRESET_FULL_RECT)
	hrow.add_theme_constant_override("separation", _pg_px(10))
	hrow.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hrow.offset_left = _pg_px(14)
	hrow.offset_right = -_pg_px(14)
	head.add_child(hrow)
	var num := _pg_mono("%02d" % (index + 1), "text_faint", 9.5)
	if stale:
		num.add_theme_color_override("font_color", _pg_warn())
	num.custom_minimum_size.x = _pg_px(22)
	num.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	hrow.add_child(num)
	var name_label := DccTheme.label(String(st["name"]), "text_bright", _pg_fs(12.5))
	name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	hrow.add_child(name_label)
	## Three states, not the canvas's two. `resolved` and `stale` both assert
	## that a world exists to be one or the other; before the first generate
	## neither is true, and printing `resolved` there would be exactly the
	## "encode no value as a plausible value" defect this project keeps
	## finding. `no world` is the third.
	var state_text := "stale" if stale else "resolved"
	if not bridge.has_world:
		state_text = "no world"
	var state_label := _pg_mono(state_text, "text_faint", 9)
	state_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	hrow.add_child(state_label)
	_pg_stale_marks.append({"index": index, "num": num, "state": state_label})
	var chev := _pg_mono(DccIcons.SYMBOLS["chevron"] if open else DccIcons.SYMBOLS["expand"],
		"text_faint", 11)
	chev.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	hrow.add_child(chev)

	if not open:
		return
	col.add_child(DccTheme.rule())
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pg_px(14))
	pad.add_theme_constant_override("margin_right", _pg_px(14))
	pad.add_theme_constant_override("margin_top", _pg_px(6))
	pad.add_theme_constant_override("margin_bottom", _pg_px(12))
	col.add_child(pad)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", _pg_px(2))
	pad.add_child(body)
	_pg_fill_group(body, index)

func _pg_toggle_group(index: int) -> void:
	_pg_open[index] = not bool(_pg_open.get(index, false))
	_pg_rebuild()

## Exactly `_build_group_section()`'s predicate: every `params.rs` row whose
## `group` matches one of this stage's, then this stage's loose `keys`. Ordinary
## rows first and `ADVANCED_KEYS` after, which is the desktop's order -- the
## difference is that the phone draws them inline instead of behind a
## disclosure, because the canvas has no disclosure form and dropping them
## would lose nine live controls.
func _pg_fill_group(body: VBoxContainer, index: int) -> void:
	var st: Dictionary = STAGES[index]
	var groups: Array = st.get("groups", [])
	var plain: Array = []
	var advanced: Array = []
	for key in bridge.param_keys():
		var info := bridge.param_info(key)
		if not groups.has(String(info.get("group", ""))):
			continue
		if ADVANCED_KEYS.has(key):
			advanced.append(key)
		else:
			plain.append(key)
	for key in (st.get("keys", []) as Array):
		plain.append(String(key))

	for key in plain:
		_pg_param(body, String(key), index)
	for entry in PHONE_GEN_ABSENT:
		if int(entry["stage"]) == index:
			if String(entry.get("route", "")) == "new_world":
				_pg_route_row(body, String(entry["label"]), String(entry["why"]))
			else:
				_pg_dash_row(body, String(entry["label"]), String(entry["why"]))
	if not advanced.is_empty():
		body.add_child(_pg_mono("ADVANCED", "text_dim", 9.5, 2))
		for key in advanced:
			_pg_param(body, String(key), index)
	if plain.is_empty() and advanced.is_empty():
		var gap := String(st.get("gap", ""))
		var l := _pg_mono(gap if not gap.is_empty()
			else "No parameters in cartalith-engine for this stage.", "text_faint", 9.5)
		l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		body.add_child(l)

# -- One field ----------------------------------------------------------------

## Dispatch on `param_info(key).type`, which is the same three-way `Kind` the
## engine's own table carries: `bool` -> the canvas's `f.isTog`, everything else
## -> `f.isRange`. There is no `f.isSeg` case, because no engine parameter is an
## enumeration -- the two the canvas draws as segments are creation-time and are
## handled by `PHONE_GEN_ABSENT` above.
func _pg_param(parent: Control, key: String, stage_index: int) -> void:
	var info := bridge.param_info(key)
	if info.is_empty():
		return
	if String(info.get("type", "float")) == "bool":
		_pg_toggle_field(parent, key, info, stage_index)
	else:
		_pg_range_field(parent, key, info, stage_index)

## `f.isTog`: a 40 x 22 track with an 18 x 18 knob inset 2 px, `min-height:44px`.
func _pg_toggle_field(parent: Control, key: String, info: Dictionary,
		stage_index: int) -> void:
	var on := bool(bridge.param_get(key))
	var btn := Button.new()
	btn.toggle_mode = true
	btn.button_pressed = on
	btn.focus_mode = Control.FOCUS_NONE
	btn.mouse_filter = Control.MOUSE_FILTER_PASS
	btn.custom_minimum_size.y = _pg_tap(44)
	btn.tooltip_text = String(info.get("label", key))
	for state in ["normal", "hover", "pressed", "focus"]:
		btn.add_theme_stylebox_override(state, DccTheme.empty())
	parent.add_child(btn)
	var row := HBoxContainer.new()
	row.set_anchors_preset(Control.PRESET_FULL_RECT)
	row.add_theme_constant_override("separation", _pg_px(12))
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	btn.add_child(row)
	var l := _pg_mono(String(info.get("label", key)), "text_secondary", 10)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(l)
	var track := PanelContainer.new()
	track.custom_minimum_size = Vector2(_pg_px(40), _pg_px(22))
	track.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	track.add_theme_stylebox_override("panel",
		_pg_box(DccTheme.c("accent") if on else _pg_chip(), _pg_px(11)))
	row.add_child(track)
	var knob := Control.new()
	knob.custom_minimum_size = Vector2(_pg_px(18), _pg_px(18))
	knob.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	knob.size_flags_horizontal = Control.SIZE_SHRINK_END if on else Control.SIZE_SHRINK_BEGIN
	var dot := PanelContainer.new()
	dot.set_anchors_preset(Control.PRESET_FULL_RECT)
	dot.add_theme_stylebox_override("panel",
		_pg_box(DccTheme.c("accent_ink") if on else DccTheme.c("text_secondary"), _pg_px(9)))
	knob.add_child(dot)
	var knob_pad := MarginContainer.new()
	knob_pad.add_theme_constant_override("margin_left", _pg_px(2))
	knob_pad.add_theme_constant_override("margin_right", _pg_px(2))
	knob_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	knob_pad.add_child(knob)
	track.add_child(knob_pad)
	btn.toggled.connect(func(v: bool):
		bridge.param_set(key, v)
		_pg_after_param_write(stage_index)
		## A toggle changes the knob's side and both its colours, which is new
		## geometry rather than new text -- so this one does rebuild, deferred
		## so the press finishes first.
		_pg_rebuild.call_deferred())

## **`PgSlider` moved to `dcc_widgets.gd` on 2026-09-07 and is now
## `DccWidgets.PgSlider`.**
##
## It was written here, for the two `_pg_*` sliders below, and that was the
## whole of its reach. A live census of the phone tree at 1080x2340
## (`_rangeswipe_probe.gd --census-only`) then counted **247 `Range` nodes, of
## which 245 are writable and inside a live vertical scroller and exactly 3
## arbitrated** -- these. The other **242** are 214 `DccWidgets.slider()` rows
## in the left-dock sheet, the 12 `DccWidgets.number()` spin boxes the journey
## planner's form adds to it, and 16 more across four phone-presented windows;
## every one of them had the defect this class closes.
##
## Read `DccWidgets.PgSlider` for the mechanism, the three gates it now applies
## (disabled slider, no vertical scroller, non-left button) and why `slop` is
## pinned from below as well as above. `DccShell.phone_fit()` attaches it to
## everything it walks; the two builders below still construct it directly,
## because this sheet is not one of the surfaces `phone_fit()` reaches.

## `f.isRange`: a `[label ......... value]` line, then `[-] [slider] [+]` with
## the steppers at 38 x 38 and radius 14.
##
## `f.disp` is the canvas's own formatter: two decimals below a step of 1,
## thousands-separated whole numbers at or above it, then the unit. The unit
## comes from `param_info`, so `m`, `deg C`, `x` and the rest are the engine's
## own strings and not a second table.
func _pg_range_field(parent: Control, key: String, info: Dictionary,
		stage_index: int) -> void:
	var is_int: bool = String(info.get("type", "float")) == "int"
	var lo := float(info.get("min", 0.0))
	var hi := float(info.get("max", 1.0))
	var step := float(info.get("step", 0.01))
	var unit := String(info.get("unit", ""))
	var value := float(bridge.param_get(key))

	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", _pg_px(2))
	parent.add_child(wrap)
	var head := HBoxContainer.new()
	wrap.add_child(head)
	var l := _pg_mono(String(info.get("label", key)), "text_secondary", 10)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	head.add_child(l)
	var readout := _pg_mono("", "text_bright", 11)
	head.add_child(readout)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pg_px(8))
	wrap.add_child(row)
	## `DccWidgets.PgSlider`, not `HSlider` -- see that class for the
	## vertical-swipe defect it exists to close. `_pg_px(8)` is Android's 8 dp
	## touch slop in this surface's own pixels, which is `_pscale`'s job and not `_ptap`'s: it is a
	## distance travelled, not a target to be floored at 44.
	var slider := DccWidgets.PgSlider.new()
	slider.slop = float(_pg_px(8))
	slider.min_value = lo
	slider.max_value = hi
	slider.step = step
	slider.value = value
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	DccWidgets.phone_slider(slider, DccTheme.phone_scale())
	## **Floored above `phone_slider()`'s own 32 dp, and measured before it was.**
	## `_genphone_probe.gd` walked every tappable control on this sheet at
	## 1080x2340 and found the sliders at `650x84` -- `248.0 x 32.0 dp` -- against
	## the 44 dp floor, because `DccTheme.PHONE_SLIDER_ROW` is 32. The canvas is
	## smaller still (`input[type=range]{height:22px}`) and answers the finger
	## question with the +/- pair either side, which this build already floors to
	## 44. Both are true and neither makes a 32 dp drag target acceptable, so the
	## row is raised here rather than exempted. `phone_slider()` itself is left
	## alone: it is shared with every other phone surface and re-basing it is a
	## different pass.
	slider.custom_minimum_size.y = maxf(slider.custom_minimum_size.y,
		float(_pg_tap(44)))

	## `f.disp` verbatim: `step<1 ? toFixed(2) : toLocaleString('en-US')`. The
	## thousands grouping is this file's own `_thousands()`, not a second
	## formatter -- `DccUnits` has no grouping helper (checked, 2026-09-07).
	var fmt := func(v: float) -> String:
		return (("%.2f" % v) if step < 1.0 else _thousands(int(round(v)))) + unit
	readout.text = fmt.call(value)

	## **U+2212 MINUS SIGN, not ASCII hyphen.** The canvas draws U+2212, the
	## shipped `IBMPlexMono-Regular.ttf` has it (cmap parsed 2026-09-07, the
	## same pass that found U+2684 and U+FF0B absent), so there is no
	## substitution to make here and none is recorded. This shipped as `"-"`
	## for one revision and the deviations table said U+2212 was in use, which
	## was the table asserting the opposite of the code; on glass the hyphen
	## renders short and high beside a full-height `+`. The plus IS a genuine
	## substitution -- the canvas's U+FF0B is absent from the font, so
	## `DccIcons.SYMBOLS["add"]`'s ASCII `+` stands in, and that one is
	## recorded.
	row.add_child(_pg_stepper(key, slider, -1.0, "−", stage_index))
	row.add_child(slider)
	row.add_child(_pg_stepper(key, slider, 1.0, DccIcons.SYMBOLS["add"], stage_index))

	## `input` updates the readout on every tick; `change` (here `drag_ended`,
	## Godot's one-shot release) is what writes the engine and marks the stage
	## stale. That is `tparam()`'s own split and `_build_param_row()`'s, so a
	## drag costs one generate rather than one per tick.
	slider.value_changed.connect(func(v: float):
		readout.text = fmt.call(v))
	slider.drag_ended.connect(func(_changed: bool):
		bridge.param_set(key, int(round(slider.value)) if is_int else slider.value)
		_pg_after_param_write(stage_index))

## A `-` / `+` cell. `38 x 38` in the canvas, floored to the tap minimum here.
## The canvas's glyphs are `U+2212` and `U+FF0B`; `U+2212` is in the shipped
## Plex Mono and `U+FF0B` is not (cmap checked 2026-09-07), so the plus is
## `DccIcons.SYMBOLS["add"]`, which is ASCII and cannot tofu.
func _pg_stepper(key: String, slider: HSlider, dir: float, glyph: String,
		stage_index: int) -> Button:
	var b := Button.new()
	b.text = glyph
	b.focus_mode = Control.FOCUS_NONE
	b.mouse_filter = Control.MOUSE_FILTER_PASS
	b.custom_minimum_size = Vector2(_pg_tap(38), _pg_tap(38))
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_font_size_override("font_size", _pg_fs(14))
	for c_key in ["font_color", "font_hover_color", "font_pressed_color"]:
		b.add_theme_color_override(c_key, DccTheme.c("text_secondary"))
	for state in ["normal", "hover", "pressed", "focus"]:
		b.add_theme_stylebox_override(state, _pg_box(_pg_chip(), _pg_px(14)))
	b.pressed.connect(func():
		slider.value = clampf(slider.value + dir * slider.step,
			slider.min_value, slider.max_value)
		## `HSlider.value_changed` fires for a programmatic write and repaints
		## the readout; `drag_ended` does NOT, so the engine write has to be
		## made here by hand. Same asymmetry `_build_param_row()`'s right-click
		## reset documents.
		var info := bridge.param_info(key)
		var is_int: bool = String(info.get("type", "float")) == "int"
		bridge.param_set(key, int(round(slider.value)) if is_int else slider.value)
		## The half the first cut of this function missed entirely: a stepper
		## is `hGenStep`, and `hGenStep` marks the stage stale exactly as a
		## drag release does.
		_pg_after_param_write(stage_index))
	return b

# -- The two honest non-controls ----------------------------------------------

## A field the canvas draws that this engine cannot back. Dashed, with the true
## reason on the row and in the tooltip -- never a plausible value.
func _pg_dash_row(parent: Control, label_text: String, why: String) -> void:
	var row := VBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	row.tooltip_text = why
	parent.add_child(row)
	var head := HBoxContainer.new()
	row.add_child(head)
	var l := _pg_mono(label_text, "text_faint", 10)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(l)
	head.add_child(_pg_mono("—", "text_faint", 11))
	var note := _pg_mono(why, "text_faint", 9)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	row.add_child(note)

## A field that is not missing -- it lives on another surface. Draws as a real
## row that opens File > New world.
func _pg_route_row(parent: Control, label_text: String, why: String) -> void:
	var b := Button.new()
	b.focus_mode = Control.FOCUS_NONE
	b.mouse_filter = Control.MOUSE_FILTER_PASS
	b.custom_minimum_size.y = _pg_tap(44)
	b.tooltip_text = why
	for state in ["normal", "hover", "pressed", "focus"]:
		b.add_theme_stylebox_override(state, DccTheme.empty())
	parent.add_child(b)
	var row := HBoxContainer.new()
	row.set_anchors_preset(Control.PRESET_FULL_RECT)
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", _pg_px(8))
	b.add_child(row)
	var l := _pg_mono(label_text, "text_secondary", 10)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(l)
	var go := _pg_mono("New world %s" % DccIcons.SYMBOLS["expand"], "accent", 9.5)
	go.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(go)
	b.pressed.connect(func():
		if app != null:
			app.open_new_world())
	var note := _pg_mono(why, "text_faint", 9)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	parent.add_child(note)

# -- SCULPT -------------------------------------------------------------------

## `isSculpt`. The canvas's own order: feature chips, hint, presets, brush
## globals, add-stamp, then the draft's DISCARD / COMMIT pair.
##
## Every list is read live off the engine -- `get_sculpt_features()`,
## `get_sculpt_presets()`, `sculpt_get_globals()` -- which is what
## `_build_sculpt()` already does for the desktop. Nothing is transcribed.
##
## The canvas's `+ ADD DRAFT STAMP (MOCK STROKE)` is the prototype standing in
## for a map stroke it has no map to take. Here the equivalent is arming the
## sculpt tool and getting out of the way, which is what
## `phone_menu.gd::_go_civ_tool()` already does for the CIVIL tools -- so the
## button arms and closes the sheet rather than fabricating a stamp.
func _pg_sculpt(parent: Control) -> void:
	if not bridge.has_world:
		var l := _pg_mono("Generate a world first -- the Sculpt editor is created "
			+ "fresh per generated world.", "text_faint", 9.5)
		l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		parent.add_child(l)
		return
	if bridge.sculpt_get_globals().is_empty():
		var l2 := _pg_mono("No sculpt editor for this world -- a loaded save has no "
			+ "draft session, only a freshly generated world does.", "text_faint", 9.5)
		l2.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		parent.add_child(l2)
		return

	parent.add_child(_pg_mono("GEOLOGICAL FEATURE", "text_dim", 9.5, 2))
	var current := bridge.sculpt_get_feature()
	var chips := HFlowContainer.new()
	chips.add_theme_constant_override("h_separation", _pg_px(7))
	chips.add_theme_constant_override("v_separation", _pg_px(7))
	parent.add_child(chips)
	var hint := ""
	for f in bridge.get_sculpt_features():
		var d: Dictionary = f
		var key := String(d.get("key", ""))
		if key == current:
			hint = String(d.get("hint", ""))
		chips.add_child(_pg_chip_button(String(d.get("label", key)), key == current, 42,
			func():
				bridge.sculpt_set_feature(key)
				app.arm_tool("sculpt")
				_pg_rebuild()))
	if not hint.is_empty():
		var hl := _pg_mono(hint, "text_faint", 9.5)
		hl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		parent.add_child(hl)

	parent.add_child(_pg_mono("PRESETS", "text_dim", 9.5, 2))
	var presets := HFlowContainer.new()
	presets.add_theme_constant_override("h_separation", _pg_px(7))
	presets.add_theme_constant_override("v_separation", _pg_px(7))
	parent.add_child(presets)
	var all_presets := bridge.get_sculpt_presets()
	for i in all_presets.size():
		var pd: Dictionary = all_presets[i]
		var idx := i
		presets.add_child(_pg_chip_button(String(pd.get("name", "Preset %d" % i)), false, 38,
			func():
				bridge.sculpt_apply_preset(idx)
				app.arm_tool("sculpt")
				_pg_rebuild()))

	parent.add_child(_pg_mono("BRUSH · GLOBAL", "text_dim", 9.5, 2))
	var globals_now := bridge.sculpt_get_globals()
	for spec in bridge.get_sculpt_globals_info():
		var sd: Dictionary = spec
		var gkey := String(sd.get("key", ""))
		if not globals_now.has(gkey):
			continue
		_pg_sculpt_slider(parent, sd, float(globals_now[gkey]))

	var arm := Button.new()
	arm.text = "%s ARM SCULPT · DRAW ON THE MAP" % DccIcons.SYMBOLS["add"]
	arm.focus_mode = Control.FOCUS_NONE
	arm.mouse_filter = Control.MOUSE_FILTER_PASS
	arm.custom_minimum_size.y = _pg_tap(48)
	arm.add_theme_font_override("font", DccTheme.mono(2, true))
	arm.add_theme_font_size_override("font_size", _pg_fs(10))
	for c_key in ["font_color", "font_hover_color", "font_pressed_color"]:
		arm.add_theme_color_override(c_key, DccTheme.c("accent"))
	var dashed := _pg_box(Color(0, 0, 0, 0), _pg_px(24), DccTheme.c("accent"), 1)
	for state in ["normal", "hover", "pressed", "focus"]:
		arm.add_theme_stylebox_override(state, dashed)
	arm.pressed.connect(func():
		app.arm_tool("sculpt")
		if app.has_method("_set_phone_detent"):
			app._set_phone_detent("peek"))
	parent.add_child(arm)
	## `StyleBoxFlat` has no dashed border, so the canvas's `1px dashed
	## var(--acc)` is drawn solid. Said here rather than left as a silent
	## divergence: the affordance the dash carries -- "this makes a draft, not
	## a commit" -- is carried by the caption instead.

	var stamps := bridge.sculpt_stamp_count()
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", _pg_px(10))
	parent.add_child(foot)
	var note := _pg_mono("%d stamp%s on the draft" % [stamps, "" if stamps == 1 else "s"],
		"accent" if stamps > 0 else "text_faint", 10)
	note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	note.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	foot.add_child(note)
	## `_on_sculpt_discard()` / `_on_sculpt_commit()`, not the bare bridge
	## calls: both also clear the viewport preview and re-point the right dock,
	## and `sculpt_commit` takes a reason string those two already supply.
	foot.add_child(_pg_chip_button("DISCARD", false, 44, func():
		_on_sculpt_discard()
		_pg_rebuild()))
	foot.add_child(_pg_chip_button("%s COMMIT" % DccIcons.SYMBOLS["tick"], true, 44, func():
		_on_sculpt_commit()
		_pg_rebuild()))

func _pg_sculpt_slider(parent: Control, spec: Dictionary, value: float) -> void:
	var key := String(spec.get("key", ""))
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", _pg_px(2))
	parent.add_child(wrap)
	var head := HBoxContainer.new()
	wrap.add_child(head)
	var l := _pg_mono(String(spec.get("label", key)), "text_secondary", 10)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(l)
	var readout := _pg_mono("%.2f" % value, "text_bright", 11)
	head.add_child(readout)
	var is_int: bool = bool(spec.get("int", false))
	## The SCULPT half of the same arbitration -- these sliders sit in the same
	## `ScrollContainer` and were reached by the same vertical swipe. Found by
	## enumerating the sheet's `Range`s rather than by hitting it a second
	## time: **`_pg_range_field()` and this function are the two sites in the
	## `_pg_*` block that construct a slider**, and both take
	## `DccWidgets.PgSlider`.
	##
	## That sentence replaced a pasted `grep -n "HSlider.new()"` and its
	## quoted count of two. The paste stopped reproducing the moment it was
	## written -- the substitution it documents removed both matches, so the
	## only line the grep returns today is the comment quoting its own
	## string, and a reader following it gets a self-reference rather than a
	## count. Name the symbols; a symbol survives the edit that a command
	## measuring the file cannot.
	var s := DccWidgets.PgSlider.new()
	s.slop = float(_pg_px(8))
	s.min_value = float(spec.get("min", 0.0))
	s.max_value = float(spec.get("max", 1.0))
	s.step = float(spec.get("step", 0.01))
	s.value = value
	DccWidgets.phone_slider(s, DccTheme.phone_scale())
	## Same 44 dp floor as `_pg_range_field()`; see its own note.
	s.custom_minimum_size.y = maxf(s.custom_minimum_size.y, float(_pg_tap(44)))
	wrap.add_child(s)
	s.value_changed.connect(func(v: float):
		readout.text = "%.2f" % v)
	## `sculpt_set_globals` takes a Dictionary -- there is no singular setter
	## (`grep -n "^func sculpt_set_global" engine_bridge.gd`, 2026-09-07), and
	## `_on_sculpt_global()` writes it the same way for the desktop.
	s.drag_ended.connect(func(_c: bool):
		bridge.sculpt_set_globals({key: (round(s.value) if is_int else s.value)}))

## The canvas's chip: `min-height:<h>px; padding:0 13px; border-radius:<h/2>px;
## font:10px mono`, accent-on-wash when selected.
func _pg_chip_button(text: String, on: bool, h: float, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.mouse_filter = Control.MOUSE_FILTER_PASS
	b.custom_minimum_size.y = _pg_tap(h)
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_font_size_override("font_size", _pg_fs(10))
	for c_key in ["font_color", "font_hover_color", "font_pressed_color"]:
		b.add_theme_color_override(c_key,
			DccTheme.c("accent") if on else DccTheme.c("text_secondary"))
	var box := _pg_box(DccTheme.c("accent_wash") if on else Color(0, 0, 0, 0),
		_pg_px(h / 2.0), DccTheme.c("accent") if on else DccTheme.c("line"), 1)
	box.content_margin_left = _pg_px(13)
	box.content_margin_right = _pg_px(13)
	for state in ["normal", "hover", "pressed", "focus"]:
		b.add_theme_stylebox_override(state, box)
	b.pressed.connect(on_press)
	return b
