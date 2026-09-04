extends Node
class_name RightDock

## Controller for the right dock's content (`DCC_SHELL_SPEC.md` §6):
## "contents follow the selection, not the workspace." One instance, wired
## from `app.gd`, owns `app.right_dock_body` and swaps what's in it whenever
## the viewport's selection changes -- independent of which domain rail
## button is active. §3 also says a domain switch "swaps both docks"; that
## is honoured only in the weak sense that nothing here contradicts it (no
## domain-specific default view exists to switch to besides Sample), because
## §6 -- named as this file's own specification -- is unambiguous that the
## dock tracks selection, not the rail.
##
## Ported from `main.gd`'s old Sample panel (`_refresh_sample_panel`, lines
## ~1567-1600 as last read) and its `_build_causal_chain_text` (the
## "WHY HERE?" explanation, ~1508-1551): the settlement causal-chain logic
## is real and unchanged here, only re-hosted under the new dock. Everything
## else in this file is new: the old Sample panel only ever had cursor
## position and settlement hover to show.
##
## **The per-cell field sampler this file used to say did not exist now
## does** (`sample_bridge.rs`, `bridge.sample_cell()`), so §6's Sample
## context is fully live -- see `SAMPLE_FIELDS` below for what each row
## reads and, in one case, for a correction to what the old comment claimed
## the ceiling was.

const CTX_SAMPLE := "sample"
const CTX_SETTLEMENT := "settlement"
const CTX_ROUTE := "route"
const CTX_RIVER := "river"
const CTX_FACTION := "faction"
const CTX_MEASURE := "measure"
const CTX_REGION := "region"
## `CTX_SCULPT` and `CTX_JOURNEY` were here. They are now `TOOL_STAMPS` and
## `TOOL_JOURNEY`, appended section ids -- see those constants for the
## measurements that caught each of them replacing a selected settlement. Do
## not re-add either as a context: a `CTX_` name, a `CTX_TITLES` row and a
## `_dispatch()` arm are the three things that made the dock replace the
## selection, and the ruling rejects all three.
## The Wildlife debug view's roster popup -- the reference's own
## `#wildInfo` panel (`showWildInfo`, HTML 8259), re-hosted here rather than
## rebuilt as a floating panel: §6 already says this dock's contents follow
## the selection, and a clicked ecoregion is a selection.
const CTX_WILDLIFE := "wildlife"
## `GUI_GAP_REGISTER.md` **ED-02** -- the history ledger. A right-dock context
## and not a window, following `DCC_SHELL_SPEC.md` §7.1 proposal 3: it is
## selection-adjacent (a row IS a selection) and this dock is already the
## context-driven surface.
const CTX_HISTORY := "history"

## `05-right-dock-and-bars.md` §1.8-§1.14, GUI replacement stage 5. **Seven**
## sections from `rdMode4()`'s own fall-through table (§1.2b): `tool` is
## `sculpt`/`freehand` -> stamps (rule 1, `TOOL_STAMPS`, converted from a
## context on 2026-09-03 -- see its own doc), `tool === 'biome'` -> paint,
## `tool` is `label`/`icon` -> anno, `tool === 'territory'`
## -> terr, `domain==='CARTO' && tool==='inspect'` -> stops, `tool` is
## `way`/`route` with a non-empty draft -> way (rule 7, `TOOL_WAY`), and the
## planner results -> plan (rule 8, `TOOL_JOURNEY`, converted from a context
## on 2026-09-04 -- see its own doc). All seven
## read live engine state fresh on every rebuild, the same "no private draft"
## shape the Stamp stack already used -- there is nothing here for a second
## editor to disagree with.
##
## **Rule 5 (`place`) is deliberately absent, and that is the ruling rather
## than an omission.** Its panel (§1.11 `rdPlace`) *is* the settlement
## inspector, and this dock already draws it -- as `CTX_SETTLEMENT`, driven by
## the selection, which is the half of rule 5 that reads `cv.sel >= 0`
## (`civilization_workspace.gd::_settlement_click` pins it for a fresh drop
## too). Its other half, `tool === 'settlement'`, would draw that same
## inspector for whatever is selected, so under *"selection wins"* it asks for
## nothing this dock is not already doing; and the Settlement tool's own four
## inputs (class, faction, name, snap-to-water) are in the options bar, where
## §2 puts them. Duplicating them here would be the "two pickers over one
## concept" shape `_append_layers()` records this shell having had to undo
## three times.
##
## **Rule 8 (`plan`) was the last arm that still replaced**, and the owner
## extended the ruling to it on 2026-09-04 (`LARGE_ITEM_RULINGS.md`: *"it
## becomes an appended section like every other"*). It is now `TOOL_JOURNEY`;
## see that constant, and `show_journey()`.
##
## **These are `TOOL_*`, not `CTX_*`, and that is the owner's 2026-09-03
## ruling rather than a naming preference** (`LARGE_ITEM_RULINGS.md`,
## verbatim): *"Selection wins; the tool appends a section."* They shipped as
## four `CTX_` constants with four `CTX_TITLES` rows and four `_dispatch()`
## arms, which made arming a tool **replace** whatever the dock was showing --
## measured in a booted app on 2026-09-03, selecting a settlement and then
## arming Territory: `title=Territory`, `settlement name SURVIVED=false`. That
## is the exact naive merge the ruling rejects in its own reasoning ("the dock
## flips away from a selected settlement the moment a tool arms"). They are now
## section ids, appended by `_append_tool()` **after** `_dispatch()` draws the
## selection -- `_append_layers()`'s shape, which was already the worked example
## of this ruling and is one section lower down.
##
## Do not give any of these a `CTX_` name, a `CTX_TITLES` row or a `_dispatch()`
## arm again: each of those three is what made the dock replace the selection.
##
## **They are not stored, either.** Which one is drawn is derived from
## `app.armed_tool` (plus the domain, for Stops) on every rebuild -- that IS
## `rdMode4()`'s own fall-through table, quoted above, so a remembered copy of
## it would be a second answer to a question the shell already answers. The
## `show_*` calls below carry the per-tool *data* these sections cannot
## otherwise see (`_paint_ctx_layer`, `_paint_on_pick`, `_terr_faction`) and
## nothing else. Measured reason, not taste: while it was stored, arming a
## different WORLD tool left the Paint section behind, because the only caller
## of `leave_paint_context()` is a *domain* switch -- `_rightdock5_probe.gd`
## had recorded that as expected behaviour when it was a whole-dock takeover,
## and appending would have turned it into a stale panel sitting under a live
## selection until the user changed domain. `_tool_section()` is the derivation.
##
## **One tool id genuinely differs from the design file's own markup**:
## `PaintTarget` (`paint_bridge.rs`) is `Biome`/`Terrain`/`Splat`, and this
## shell's own armed-tool id is `"paint"`, not `"biome"` -- `world_workspace
## .gd` registers `register_tool_click_handler("paint", ...)`. The constant
## below is named for what this port actually calls it.
const TOOL_PAINT := "paint"
## `rdStops` (§1.9). Triggered when the CARTO domain is active with no more
## specific tool armed -- see `show_stops()`'s own doc for the two call
## sites this needs (a domain switch fires no `tool_armed` of its own).
const TOOL_STOPS := "stops"
## `rdAnno` (§1.10) -- Label and Icon share one section, exactly as they
## share one entry in the design's own right-dock title table.
const TOOL_ANNO := "anno"
## `rdTerr` (§1.12). Named `TOOL_TERR` (not `TOOL_TERRITORY`) to match the
## design doc's own short id and the grep this stage's own brief was
## written against.
const TOOL_TERR := "territory"
## `rdStamps` -- **`rdMode4()`'s rule 1, and the last place in this dock where
## the owner's 2026-09-03 ruling was not yet applied.**
##
## The Stamp stack shipped as `CTX_SCULPT`: a context constant, a `CTX_TITLES`
## row and a `_dispatch()` arm -- the exact three things the `TOOL_*` block
## above warns never to give a tool again, because each of them is what makes
## arming a tool *replace* whatever the dock was showing. It was measured doing
## precisely that (`_rdappend_probe.gd`, 2026-09-03): with a settlement
## selected, arming Sculpt gave `title=Stamp stack` and the settlement's own
## name gone from the body. Territory's identical defect was found and fixed on
## the same day; this one was rule 1 of the same table and was left standing.
##
## **Derived from two clauses, both gated on WORLD, and the second is
## behaviour preservation rather than a new rule.** The design's own rule 1 is
## `tool is sculpt or freehand -> stamps`, which is the first clause. The
## second -- a draft that still holds stamps -- exists because the old context
## flag survived a disarm and three call sites depended on that:
##
## * `world_workspace.gd::_sculpt_escape()` disarms to Inspect after cancelling
##   a stroke. Under the tool clause alone, Escape would take the Commit and
##   Discard controls away from a draft that is still uncommitted.
## * `menus.gd`'s Edit > Select All in WORLD selects every stamp and
##   re-announces this dock, with whatever tool the user had armed.
## * `_deadwire_probe.gd::_audit_warm_sculpt()` draws two real strokes and then
##   audits the stack's four gated buttons without arming anything.
##
## The WORLD gate is `leave_sculpt_context()`'s own condition read off
## `app.gd::_on_workspace_changed` (`if id != "world"`), not a new opinion --
## the same source `TOOL_PAINT`'s gate came from.
const TOOL_STAMPS := "stamps"
## `rdWay` (§1.14) -- `rdMode4()`'s **rule 7**: `tool` is `way` or `route`
## **and the draft is non-empty**. The last rung of the ladder that had no
## surface anywhere in this shell; §1.14's four rows were never drawn, and the
## draft's own controls (type, Commit, Discard) are in the tool options bar
## (§2), which is where the design puts them and where they stay.
##
## **One section id for two tools, and two titles -- a recorded deviation.**
## `rdMode4()` collapses Way and Route into one mode, which is why §1.3's
## `ROUTE` title is a dead entry the spec itself flags (*"`rdMode4()` returns
## `'way'` for both the Way and Route tools, so `ROUTE` is unreachable"*). This
## port revives it, because the two are genuinely different things here rather
## than the reference's alias: a way is a permanent edge in the network
## (`way_bridge.rs`, committed into `get_roads()`/`get_sea_routes()`), a route
## is a planned traversal over it (`route_bridge.rs`, committed into
## `route_get()`), they carry different options and land in different lists. So
## the section header names whichever tool drew the draft.
##
## **And it says "draft", which §1.3's `WAY` does not.** `_build_route()`
## already titles the *selected* committed route's section `Route`, so under
## the ruling both can now be on screen at once -- a collision that cannot
## happen in the reference, where `rdWay` **replaces** the dock and the two
## are mutually exclusive by construction. Appending is this port's change, so
## the disambiguation is this port's to make; the word is `rdMode4()` rule 7's
## own (`wy().draft.length > 0`), not a new one.
##
## **No domain gate, unlike `TOOL_PAINT` and `TOOL_STAMPS`, and for the same
## kind of reason those two have one**: nothing in the shell clears a Way or
## Route draft on a domain switch (`app.gd`'s `_on_workspace_changed` calls no
## `leave_*` for either, and the draft itself lives in Rust), so gating on CIVIL
## would *remove* a live draft's readout that the old code left alone. Rules 3
## and 4 got no gate for exactly this reason and rule 7 gets none either.
const TOOL_WAY := "way"
## `rdPlan` (§1.13) -- `rdMode4()`'s **rule 8**, and the last arm in this dock
## that still replaced the selection. Converted 2026-09-04 on the owner's own
## extension of the 2026-09-03 ruling (`LARGE_ITEM_RULINGS.md`: *"`rdMode4()`
## (rule 8) is the last built context that replaces the selection; it becomes
## an appended section like every other"*).
##
## It shipped as `CTX_JOURNEY`: a context constant, a `CTX_TITLES` row and a
## `_dispatch()` arm -- the exact three things the `TOOL_*` block above warns
## never to give a tool, because each of them is what makes arming a tool
## *replace* whatever the dock was showing. Those three are readable in the
## conversion's own diff and are not restated here as a measurement.
##
## **It had a fourth, and that one was measured.**
## `journey_planner_view.gd::build_results()` **cleared `right_dock_body`
## itself** before drawing -- redundant while Journey replaced the dock, fatal
## the moment it appended, since the selection `_dispatch()` had just drawn sat
## in that same container. Re-introducing only that teardown under
## `_jp8append_probe.gd` (2026-09-04, mutation run, restored after) turned T1
## red exactly there: `the selected settlement's name SURVIVES arming Journey`
## FAIL, and `settlement@-1 journey@2` -- the settlement section swept out from
## under the appended plan. The teardown is gone; `_rebuild()` was already the
## only clear.
##
## **The condition is this port's own, and deliberately not §1.2b's literal
## text.** Rule 8 reads `domain==='CIVIL' && civCat==='planner' && tool in
## {inspect, pan}` -- a *category* selection in a dock this port does not
## have. This shell arms a real tool for it instead (`journey_planner_view
## .open()` calls `app.arm_tool("journey")`), so the equivalent condition is
## `armed_tool == "journey" && active_domain() == "civilization"`. That is
## `_recompute_visibility()`'s own condition, copied rather than invented: the
## section must draw exactly when the planner's own panels are showing, or the
## dock would carry results for a surface that is not on screen.
##
## **Plus `_journey_view != null`, which is a data clause, not a third rule** --
## the same shape `TOOL_WAY`'s `_way_draft.size() > 0` carries. The delegate is
## what `build_results()` is called on, and `show_journey()` is what supplies
## it; with no delegate there is nothing to draw and the section is absent
## rather than drawing an empty one.
const TOOL_JOURNEY := "journey"

## Noun phrases for `explain_settlement()`'s suitability term keys. Copied
## verbatim from `main.gd`'s own `SUIT_TERM_LABELS` -- wording belongs to the
## UI, not the engine (`ARCHITECTURE.md`), so this is the same wording, not
## a rewrite.
const SUIT_TERM_LABELS := {
	"carrying_capacity": "fertile land",
	"water_access": "fresh water",
	"gentle_slope": "gentle terrain",
	"terrain_form": "terrain form",
	"coastal_access": "coastal access",
	"river": "river access",
	"lake": "lakeside",
	"minerals": "mineral deposits",
	"route_corridor": "natural route corridor",
	"farmland": "farmland",
	"buildable_ground": "buildable ground",
	"flood_risk": "flood risk",
	"islet_penalty": "isolation",
	"water_bonus": "water",
}

## §6's Sample fields, in the spec's own order, each with the `sample_cell()`
## key it reads and the tooltip explaining where that reading comes from.
##
## **All twelve fields that used to read `—` here are live.** The block this
## replaces (`MISSING_SAMPLE_FIELDS`) listed each one with "no per-cell query"
## against a `WorldGen` that exported no field sampler; `sample_bridge.rs` is
## that sampler, and it needed no new retention anywhere to build -- every
## reading is either a raster generation already keeps (`WorldState::field`/
## `temperature`/`rainfall`/`flow_discharge`/`plate_id`/`boundary_mask`/
## `boundary_type`/`stress_field`/`age_field`/`crust_field`/`resistance_field`,
## `CivData::water_bodies`/`territory`) or is derived from those at the one
## queried cell.
##
## **One of the old comments was wrong and is corrected rather than deleted
## quietly.** The Biome row said `explain_settlement()`'s doc comment meant
## "retaining the rasters for arbitrary-cell queries would cost hundreds of
## MB". That doc comment is about the *suitability* rasters (coast SDF, river
## order, travel cost, the weighted terms), which genuinely are computed and
## dropped inside `compute_civilisation`. Biome is not one of them:
## `build_water_bodies`' classification is already retained on `CivData` for
## the Settlement tool's snap-to-water, and `classify_biome(t, m)` is a pure
## two-argument function over two rasters `WorldState` already holds. Nothing
## about the memory budget (`MEMORY_OPTIMIZATION_SCOPE.md`) had to move.
const SAMPLE_FIELDS := [
	{"label": "Slope", "key": "slope_deg",
		"tip": "Real ground angle, from the central-difference gradient of the height field at this cell (O(1), no slope raster). The parenthesised figure is slopeAt*GW, the engine's own resolution-independent unit -- the one build_settlement_suitability and buildCartTerrain threshold against."},
	{"label": "Aspect", "key": "aspect_deg",
		"tip": "Downslope bearing (the direction the ground faces), from the same gradient. New work: the reference's aspectFactor is a shading scalar, not a bearing, so no parity claim is made. Reads — on perfectly flat ground, where aspect is undefined."},
	{"label": "Plate + type", "key": "plate",
		"tip": "WorldState::plate_id, with oceanic/continental from the sign of crust_field (plateCrust() < 0 is oceanic)."},
	{"label": "Boundary + distance", "key": "boundary_type",
		"tip": "WorldState::boundary_type at this cell, plus the Euclidean distance to the nearest boundary_mask cell by expanding-ring search. The search is capped at 96 cells so a world with no tagged boundary cannot turn one mouse-move into a full-grid scan; past that it says so rather than reporting a number."},
	{"label": "Resistance", "key": "resistance",
		"tip": "WorldState::resistance_field -- the erosion-resistance input, retained since the lithology port needed it."},
	{"label": "Lithology", "key": "lithology",
		"tip": "buildLithology() evaluated at this one cell. It is strictly per-cell (its own doc comment: \"Pure, single-pass, no neighbour reads\"), so it is called on one-element slices rather than restating any of its golden-tested branches here."},
	{"label": "Temperature", "key": "temperature_c",
		"tip": "WorldState::temperature, degrees Celsius."},
	{"label": "Precipitation", "key": "precipitation",
		"tip": "WorldState::rainfall, the engine's normalised [0,1] moisture -- not millimetres, which this port's climate model never computes."},
	{"label": "Drainage", "key": "drainage",
		"tip": "WorldState::flow_discharge (upstream accumulation), with the Strahler order from stream_order when river extraction ran."},
	{"label": "Biome", "key": "biome",
		"tip": "CivData::water_bodies for ocean/lake, otherwise classifyBiome(temperature, rainfall) at this cell. Reads — on a loaded save, which carries no civilisation layer at all."},
	{"label": "Soil", "key": "soil",
		"tip": "buildSoilFertility() at this one cell, over the same one-element-slice call the Lithology row uses."},
	{"label": "Control", "key": "control",
		"tip": "CivData::territory -- assign_territory()'s owner per cell, 0 = unowned. Reads — on a loaded save."},
]

## `05-right-dock-and-bars.md` §1.4's footnote, verbatim: *"fields owned by
## stale stages read —"*. Which pipeline stage owns which row, derived from what
## `sample_cell()` actually reads (`sample_bridge::sample_cell()`, named
## rather than cited by line -- that file moves) rather than from where the
## row sits in the panel:
##
## * `height` -- `WorldState::field` and the two gradients taken from it. The
##   graph's root stage has no upstream, so `staleness()` can never report it
##   (`cartalith-spatial/src/staleness.rs:243-267 pub fn staleness`) and
##   these three never dash.
##   That is the correct answer rather than a dead entry: a sculpt writes the
##   height field in place, so elevation IS current the instant the stroke
##   commits -- it is everything downstream of it that is not. Named anyway, so
##   the ownership is stated once here instead of inferred from an absence.
## * `hydrology` -- `flow_discharge`, and `stream_order` for the order suffix.
## * `climate` -- `temperature`/`rainfall`, and the two per-cell functions
##   `sample_cell()` evaluates over them: `build_lithology` takes rainfall and
##   `build_soil_fertility` takes both, so neither row is upstream of climate
##   however geological it reads. Each field is gated on its DEEPEST input only,
##   which is sufficient because `pipeline_stage_graph()` makes every stage
##   depend on all of its upstreams -- a stale `height` is always a stale
##   `climate` too.
## * `civ` -- `CivData`'s own rasters (`water_bodies` for Biome, `territory`
##   for Control) and `get_settlements()` for `Nearest`.
##
## **Deliberately absent, and therefore never gated:** `Plate + type`,
## `Boundary + distance` and `Resistance` read `plate_id`, `crust_field`,
## `boundary_mask`, `boundary_type` and `resistance_field` -- tectonic-era
## fields that no stage in `pipeline_stage_graph()` writes, so no stage's
## staleness says anything at all about them. `Position` and `Cell` are the
## cursor, not the engine. Gating those on a stage would dash them for a reason
## that is not true of them, which this file treats as exactly as bad as
## leaving a stale value on screen.
const SAMPLE_STAGE := {
	"Elevation": "height",
	"Slope": "height",
	"Aspect": "height",
	"Drainage": "hydrology",
	"Temperature": "climate",
	"Precipitation": "climate",
	"Lithology": "climate",
	"Soil": "climate",
	"Biome": "civ",
	"Control": "civ",
	"Nearest": "civ",
}

## `Nearest`'s own tip, named because both `_build_sample()` and
## `on_cursor_sampled()` compose the staleness reason onto it.
const _NEAREST_TIP := "Computed here from get_settlements()'s x/y against the cursor cell."

var app: DccApp
var bridge: EngineBridge

var _context := CTX_SAMPLE
var _settlement_data: Variant = null
var _settlement_index := -1
var _route_entry: Dictionary = {}
var _route_kind := ""      ## "road" | "sea"
var _faction_id := -1
## The other party of the pair the reader asked for, or -1 when the faction was
## opened on its own (a Factions-list row, a map click). See `show_faction`.
var _faction_pair := -1
var _measure_result: Dictionary = {}
var _measure_mode := "distance"   ## One of `GlobalTools.MEASURE_MODES`' ids.
## The saved-measurements store -- the canvas's own "Saved measurements" list,
## and `annotations/measurements.json` on disk. Entries are
## `{mode: String, points: PackedVector2Array, value: float, unit: String}`,
## with `value`/`unit` **omitted together** when the reading had no single
## number to keep (`_measure_primary()`); the list dashes such a row rather
## than printing a zero that would read as a measurement.
##
## `value` is canonical km / km² / metres / degrees, never the display unit --
## `DccUnits`' own rule ("Canonical storage stays km; this only converts what a
## readout shows"), which is what makes the CSV an export rather than a
## screenshot of whatever Preferences ▸ Units happened to be set to.
##
## Anchored to one world, and cleared rather than carried when that world is
## replaced -- see `clear_measurements()`.
var _saved_measurements: Array = []
## The picked river -- one `river_at()` entity, whole. Empty means CTX_RIVER
## has nothing to draw, which after this file's own click wiring can only
## happen if the world is regenerated under a selection.
var _river: Dictionary = {}
var _region_result: Dictionary = {}
var _wildlife_region: Dictionary = {}
var _journey_view: JourneyPlannerView = null   ## TOOL_JOURNEY delegate -- see `show_journey()`.

## -- TOOL_PAINT. `_paint_ctx_layer` mirrors `world_workspace.gd`'s own private
## `_paint_layer` -- the caller passes it on every `show_paint()`, the same
## shape `show_measure(result, mode)` already uses, rather than this file
## reaching into another workspace's state. `_paint_on_pick` is bound by that
## same caller so a click on this dock's legend can arm a value without this
## file guessing at the other four `paint_set_brush` fields (radius/hardness/
## softness/land_only) it has no way to know -- see `show_paint()`.
var _paint_ctx_layer := "biome"
var _paint_on_pick: Callable = Callable()

## -- TOOL_STOPS. Which ramp stop (by position in `bridge.color_ramp()`, sorted)
## this dock's own "Selected stop" section edits -- local to this file, since
## the ramp itself carries no selection of its own (unlike labels/icons,
## which do: `label_get_selected()`/`icon_get_selected()`).
var _stops_selected := -1

## -- TOOL_TERR. The faction the Territory tool is currently armed for --
## passed in on every `show_territory()`, the same reason `_paint_ctx_layer`
## is: `civilization_workspace.gd` owns `_territory_faction`, not this file.
var _terr_faction := -1

## -- TOOL_WAY. The Way/Route draft, handed over by
## `infrastructure_workspace.gd` on every change (`show_way()`): which tool owns
## it, its points in grid coordinates, and the way type's display label ("" for
## Route, which has no type -- `route_begin("mixed")` is hardcoded, matching the
## reference; see that file's `_on_infra_tool_armed`).
##
## **Remembered, and structurally unable to leave a stale section on screen.**
## `_way_owner` is compared against `app.armed_tool` *live* in
## `_tool_section()`, so a draft left over from a tool that is no longer armed
## can only ever **suppress** this section -- never draw one. That inverts the
## failure the `TOOL_*` block above records (a stored section id kept Paint on
## screen after its tool disarmed): here the remembered half is the data and the
## deciding half is live.
##
## Remembered at all because the draft lives in Rust behind
## `way_append_point`/`route_append_stop`, neither of which reads it back, and
## there is no `way_draft()` getter to derive it from -- the same reason
## `infrastructure_workspace.gd` keeps `_way_points` for the canvas preview.
var _way_owner := ""
var _way_draft: PackedVector2Array = PackedVector2Array()
var _way_kind := ""

## Live-updated in place on every `cursor_sampled` rather than triggering a
## full `_rebuild()` -- the overlay emits that signal on every mouse-motion
## event over the viewport, and tearing the dock down and rebuilding it at
## that rate would be needless churn for sixteen labels.
## The cursor's coordinate, as **two rows carrying a pair each** rather than
## the four single-number rows this used to be (`X`, `Y`, and nothing in km at
## all). A latitude is not a separate fact from its longitude, and one row per
## axis both read as two unrelated readings and gave each axis its own value
## label to size itself against -- see `_field()`'s own note on why that
## mattered for the dock's width.
## `05-right-dock-and-bars.md` §1.4's footnote, held so it can be rewritten
## without a rebuild. See `_stale_footnote_text()`.
var _sample_stale_note: Label
var _sample_pos: Label     ## km from the map's north-west corner, X · Y
var _sample_cell: Label    ## the raster index every other row in this panel reads
var _sample_elev: Label
var _sample_nearest: Label
## `SAMPLE_FIELDS` label -> its value `Label`, so one `sample_cell()` dict
## per motion event fills every row. **One call, not sixteen**: the engine
## returns the whole cell in one `Dictionary` precisely so this handler
## never crosses the GDExtension boundary more than once per mouse-move.
var _sample_rows: Dictionary = {}

## `stale_stages()` is a pure read on the engine side -- every `StageGraph`
## query takes `&self` -- but it is not a free one: it walks every tile of every
## stage. `on_cursor_sampled()` fires on every mouse-motion event over the
## viewport, and this panel's whole design is ONE boundary crossing per motion
## (see `_sample_rows`), so the answer is cached and re-read on the same
## one-second cadence `app.gd`'s own staleness poll uses rather than per motion.
## Nothing makes a stage stale except a tool commit, so a reading up to a second
## old is the same reading.
var _stale_cache: Dictionary = {}
var _stale_cache_ms := -1000000

func _stale_now() -> Dictionary:
	var t := Time.get_ticks_msec()
	if t - _stale_cache_ms >= 1000:
		_stale_cache = bridge.stale_stages()
		_stale_cache_ms = t
	return _stale_cache

## Empty when this row's value is current; otherwise the reason it reads `—`.
## The stage graph reports the most-upstream unconsumed change all the way down
## the chain by design, so `reason` names the edit that caused it ("sculpt"),
## not the intermediate stage that passed it on.
func _stale_reason(label_text: String, stale: Dictionary) -> String:
	var stage := String(SAMPLE_STAGE.get(label_text, ""))
	if stage == "" or not stale.has(stage):
		return ""
	var e: Dictionary = stale[stage]
	var cause := String(e.get("reason", ""))
	if cause == "":
		cause = String(e.get("origin", ""))
	return ("Stale: the %s stage has not re-run since %s, so the engine's answer " +
		"for this cell is from before that edit. Recompute (status bar) to settle it.") % [stage, cause]

func _tip_with(tip: String, why: String) -> String:
	return tip if why == "" else "%s\n\n%s" % [tip, why]

## `05-right-dock-and-bars.md` §1.4's footnote for the staleness `stale_stages()`
## reports right now, or `""` when nothing is stale.
##
## The first clause is the prototype's own line, verbatim and lower-case. The
## parenthesis after it is this port's: `stale_stages()`' keys are the stage
## *names* the graph actually reports (`height`, `hydrology`, `climate`, `civ`),
## and naming them turns "some of these rows are old" into "these rows are old"
## without the reader having to hover twelve tooltips to find which. Sorted so
## the sentence does not reorder itself between two readings of one unchanged
## state -- `Dictionary.keys()` gives insertion order, and the engine builds that
## dictionary by walking a graph.
##
## **Only the stages this panel actually reads count.** `SAMPLE_STAGE`'s own doc
## comment lists which row each stage owns; a stale stage that owns no row here
## would make this footnote claim a dash the reader cannot find. Derived from
## `SAMPLE_STAGE`'s values rather than from a second hand-written list, so a row
## added to that table is covered by this sentence on the same edit.
func _stale_footnote_text(stale: Dictionary) -> String:
	if stale.is_empty():
		return ""
	var owned: Array = []
	for stage in stale:
		if SAMPLE_STAGE.values().has(String(stage)) and not owned.has(String(stage)):
			owned.append(String(stage))
	if owned.is_empty():
		return ""
	owned.sort()
	return "fields owned by stale stages read — (%s)" % ", ".join(owned)

## `_field()` hangs the tooltip on the row `HBoxContainer` it returns the value
## `Label` out of, and staleness starts and stops without a dock rebuild -- so a
## row that begins dashing mid-session has its reason rewritten here, rather
## than keeping the tip it was built with. A dashed row with no reason for the
## dash is the exact defect this pass exists to remove.
func _set_row_tip(v: Label, tip: String) -> void:
	var box := v.get_parent() as Control
	if box != null:
		box.tooltip_text = tip

func setup(a: DccApp, b: EngineBridge) -> void:
	app = a
	bridge = b
	bridge.generation_finished.connect(func(ok: bool):
		## A regenerate replaces the receiver tree, so the picked river's own
		## index and geometry are from a world that no longer exists.
		_river = {}
		## And so are every saved measurement's points, for the same reason and
		## one worse: a measure point is a grid cell, so the old chain would not
		## merely point at the wrong river, it would draw a plausible line over
		## ground it was never measured on. A heightmap import arrives here too
		## (`EngineBridge.import_heightmap()` emits this signal), which is
		## right -- an imported heightmap is a new world.
		if ok:
			clear_measurements()
		if _context == CTX_RIVER:
			_context = CTX_SAMPLE
		## And the Way/Route draft, for the measurements' own reason one line
		## up: its points are grid cells, so a draft kept across a regenerate
		## would report a length and a grade over ground it was never drawn on.
		_forget_way_draft()
		_rebuild())
	bridge.world_loaded.connect(func():
		_river = {}
		## **Only when no world remains.** This signal fires for seven different
		## reasons and only `close_world()` leaves `has_world` false; the
		## in-place ops (centre landmasses, carve fjords, apply an asset pack)
		## keep the same grid and must not throw a reading away, and a project
		## *open* is `restore_measurements_document()`'s job, called from
		## `app.gd::_restore_project_documents()`. Same split
		## `journey_planner_view.gd` already draws for its own list.
		if not bridge.has_world:
			clear_measurements()
		if _context == CTX_RIVER:
			_context = CTX_SAMPLE
		_forget_way_draft()
		_rebuild())
	## River selection (`OUTSTANDING_WORK.md` §2.2). Connected here rather than
	## in `app.gd`'s `_wire_selection()` for the same reason
	## `asset_library_window.gd` reaches `world_gen` directly: `app.gd` is a
	## concurrently-edited file this pass. The signal contract is `app.gd`'s own
	## and unchanged -- `_wire_selection` already hangs the Wildlife debug
	## view's ecoregion pick off this exact signal, so a right-dock context
	## driven by `map_clicked` is the established shape, not a new one.
	##
	## **Ordering is guaranteed, not lucky.** `map_overlay._gui_input` emits
	## `settlement_selected` and *then* `map_clicked`, and Godot delivers a
	## signal synchronously to every connection before the next `emit` begins.
	## So by the time this runs, `on_settlement_selected` has already set
	## `_context` -- `CTX_SETTLEMENT` if the click hit a pin, `CTX_SAMPLE` if it
	## missed everything. Only the second case is a river click, which is what
	## makes "settlement wins over river" fall out rather than need arbitrating.
	app.viewport.map_clicked.connect(_on_map_clicked_river)
	## RD-10. Two sources, and neither alone covers the section: the domain
	## decides whether it is drawn at all, and the stack decides what it says.
	## `workspace_changed` is `DccShell`'s own signal -- `app.gd`'s
	## `_on_workspace_changed` already reaches this dock for Sculpt, Paint and
	## Stops, but only through calls that are conditional on a context, so a
	## CARTO switch with (say) Measure armed rebuilds nothing.
	app.workspace_changed.connect(func(_id: String): _rebuild())
	## The appended tool section is derived from `app.armed_tool` on every
	## rebuild (`_tool_section()`), so every tool change has to cause one. The
	## `show_*` calls cover arming; **nothing covered disarming inside the
	## same domain** -- `app.gd` calls `leave_paint_context()` only on a domain
	## switch, so arming any other WORLD tool used to leave Paint on screen.
	## Measured: `_rdappend_probe.gd`'s "paint: disarm drops the tool's section"
	## failed on the first run of this pass, with `PAINT · BIOME` still in the
	## header list after `arm_tool("inspect")`. One connection here, rather than
	## a `leave_*` call added to each workspace's own `_on_tool_armed`, because
	## the rule being satisfied is this dock's and the miss was one of the four
	## already having no owner.
	app.tool_armed.connect(func(_id: String): _rebuild())
	bridge.layer_stack_changed.connect(_rebuild)
	_rebuild()

## §2.2's viewport river hit-testing. Gated four ways, each for its own reason:
##
## * `armed_tool != "inspect"` -- a click belongs to whatever tool is armed.
##   Inspect is the shell's own default and the only tool with no click handler
##   of its own (`app.gd`'s `_on_map_clicked` dispatches by armed tool and finds
##   nothing for it), so this claims no click any tool wanted.
## * `debug_view() == "wildlife"` -- that view already owns the map click and
##   drives this same dock (`app.gd`'s `_wire_selection`). Its connection is
##   made after this one, so without this guard both would fire and the dock
##   would flip to Ecoregion a frame after showing a river.
## * `_context != CTX_SAMPLE` -- the click hit a settlement (see `setup()`).
## * `has_method` -- the same degrade-rather-than-crash probe every `_has()`
##   guard in `engine_bridge.gd` uses; an older cdylib has no `river_at`.
##
## A click that finds no river leaves the dock in Sample, which is what it
## already showed: no rebuild, no flicker.
func _on_map_clicked_river(gx: float, gy: float) -> void:
	if app == null or bridge == null or not bridge.has_world:
		return
	if app.armed_tool != "inspect" or _context != CTX_SAMPLE:
		return
	if app.viewport.debug_view() == "wildlife":
		return
	if bridge.world_gen == null or not bridge.world_gen.has_method("river_at"):
		return
	var hit: Dictionary = bridge.world_gen.river_at(
		gx, gy, _river_pick_radius_cells(), RIVER_PICK_MIN_ORDER)
	if hit.is_empty():
		return
	show_river(hit)

## Trace every river, headwater trickles included, so nothing drawn on the map
## is unselectable. `EXPORT_MIN_RIVER_ORDER`'s 2 is the right filter for a
## file; it is the wrong one for a pointer, which should pick what the eye can
## see. Measured on a 192x144 world through this exact binding (a headless
## `get_rivers()` probe): **784 runs at min_order 1, 128 at 2** -- so a `2`
## here would make five rivers in six unclickable. (`tests/river_entities.rs`
## reports 773 / 125 for the same seed and size; it builds its world from
## `WorldParams::defaults`, where the divergence flags are `false`, while the
## shell's own `params::defaults()` sets them `true`. Two real worlds, not a
## discrepancy.)
const RIVER_PICK_MIN_ORDER := 1

## The pointer target, in screen pixels of *radius* -- 44 px across, the touch
## minimum this shell holds every hit target to.
const RIVER_PICK_PX := 22.0

## `RIVER_PICK_PX` expressed in grid cells at the current zoom, which is what
## `river_at()` takes.
##
## The raster is fitted into the panel preserving aspect and then scaled by the
## camera, so its drawn width is `min(panel_w, panel_h * gw/gh)` and one cell is
## `drawn_w * zoom / gw` screen pixels. That `min` is the whole of the fit --
## not a second copy of `map_overlay`'s projection, which also carries pan,
## plate margins and LOD tiling that a hit radius has no use for.
##
## Using the bare panel width instead would shrink the target on a portrait
## world in exact proportion to the letterboxing, which is the wrong direction
## for a touch minimum to be wrong.
func _river_pick_radius_cells() -> float:
	var gs := bridge.grid_size()
	if gs.x <= 0 or gs.y <= 0 or app == null:
		return 0.0
	var panel := app.viewport.size
	var drawn_w := minf(maxf(1.0, panel.x), maxf(1.0, panel.y) * float(gs.x) / float(gs.y))
	var px_per_cell := drawn_w * maxf(0.01, app.viewport.zoom()) / float(gs.x)
	return RIVER_PICK_PX / maxf(0.0001, px_per_cell)

# -- Selection API --------------------------------------------------------
#
# `on_settlement_selected` / `on_cursor_sampled` match the method names
# `app.gd`'s `_wire_selection` already forwards to any workspace that has
# them; this node is added to that same forwarding list (`app.gd`). The
# other two are called directly by the workspaces that list routes and
# factions, since no viewport signal exists for either selection yet.

func on_settlement_selected(data: Variant, index: int) -> void:
	if data == null:
		_context = CTX_SAMPLE
		_settlement_data = null
		_settlement_index = -1
	else:
		_context = CTX_SETTLEMENT
		_settlement_data = data
		_settlement_index = index
	_rebuild()

func on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	if _context != CTX_SAMPLE:
		return
	if _sample_pos == null:
		_rebuild()
		return
	var coord := _coord_texts(gx, gy, valid)
	_sample_pos.text = coord[0]
	_sample_cell.text = coord[1]
	## §1.4's staleness gate, read once for the whole panel -- see
	## `SAMPLE_STAGE` for which row each stage owns, and which rows no stage
	## owns. `Position` and `Cell` above are the cursor's own reading and are
	## never gated.
	var stale := _stale_now()
	## §1.4's footnote, rewritten in place for the same reason `_set_row_tip()`
	## exists: staleness starts and stops without a dock rebuild, so a footnote
	## written once at build time would go on explaining dashes that had settled --
	## a stale sentence about staleness, which is the worst version of it.
	if _sample_stale_note != null:
		_sample_stale_note.text = _stale_footnote_text(stale)
		_sample_stale_note.visible = _sample_stale_note.text != ""
	var nearest_why := _stale_reason("Nearest", stale)
	_sample_nearest.text = "—" if nearest_why != "" else _nearest_settlement_text(gx, gy, valid)
	_set_row_tip(_sample_nearest, _tip_with(_NEAREST_TIP, nearest_why))
	var cell: Dictionary = bridge.sample_cell(int(round(gx)), int(round(gy))) if valid else {}
	## `height` cannot be reported stale (see `SAMPLE_STAGE`), so this branch
	## never takes today -- written the same way as every other row so that a
	## stage added upstream of height gates the dock's one big number too,
	## instead of leaving it the single unguarded readout.
	_sample_elev.text = "—" if _stale_reason("Elevation", stale) != "" else _elevation_text(cell)
	## RD-11: live-updated in place for the same reason the rows above are --
	## a full `_rebuild()` on every mouse-motion event would be needless
	## churn, and the collapsed dock's one number is this same elevation
	## reading, not a stale one from the moment the dock last rebuilt.
	if app != null:
		app.set_dock_readout("right", _sample_elev.text)
	for f in SAMPLE_FIELDS:
		var row: Label = _sample_rows.get(f["label"])
		if row == null:
			continue
		## The owning stage is stale, so the engine's answer for this cell is
		## from before the last edit: dashed rather than printed. The same rule
		## `_sample_field_text()` already applies to an absent key -- never a
		## real-looking number for something that is not a reading of the world
		## as it stands.
		var why := _stale_reason(f["label"], stale)
		var text := "—" if why != "" else _sample_field_text(f["key"], cell)
		row.text = text
		_set_row_tip(row, _tip_with(String(f["tip"]), why))
		## `text_ghost` is this dock's own "nothing behind this row" tone
		## (`_field`'s `reachable` argument). A row goes ghost when its
		## reading is genuinely absent for this world -- no civ layer, no
		## river network, flat ground, no boundary inside the search cap --
		## not when it is merely off-map.
		row.add_theme_color_override("font_color",
			DccTheme.c("text" if text != "—" else "text_ghost"))

## Called by `infrastructure_workspace.gd` when a road or sea-route row is
## clicked. `kind` is `"road"` or `"sea"` -- the two calls this dock's Route
## context actually has (`bridge.roads()` / `bridge.sea_routes()`).
func show_route(entry: Dictionary, kind: String) -> void:
	_context = CTX_ROUTE
	_route_entry = entry
	_route_kind = kind
	_rebuild()

## Called by this file's own `_on_map_clicked_river`, with one `river_at()`
## entity. Public so a future river list (an Infrastructure-style rows panel
## over `get_rivers()`) can drive the same context the way
## `infrastructure_workspace.gd` drives Route.
func show_river(entity: Dictionary) -> void:
	_context = CTX_RIVER
	_river = entity
	_rebuild()

## Called by `civilization_workspace.gd` when a faction row is clicked.
##
## `pair_with` is `GUI_GAP_REGISTER.md` **RL-01**. CIVIL ▸ Relationships lists
## one row per *pair* (`Aurelia ↔ Korrath — wary (−22)`) and every row called
## `show_faction(a)` — so a row claiming a pair opened one side of it, and any
## two consecutive rows sharing that side were a press with **no visible effect
## at all**: measured 5 of 15 rows dead on a real six-faction world. Naming the
## other party here does both halves of the fix. The dock draws the pair the
## row actually named, and pressing a different row always changes something,
## because the marked pair is part of what is drawn.
func show_faction(faction_id: int, pair_with: int = -1) -> void:
	_context = CTX_FACTION
	_faction_id = faction_id
	_faction_pair = pair_with
	_rebuild()

## Called by `GlobalTools` on every point added to (or cleared from) the
## Measure chain. `mode` is one of `GlobalTools.MEASURE_MODES`' own ids and
## `result` is whichever engine dict that mode reads --
## `measure_result()` for Distance/Bearing, `measure_area()`,
## `measure_radius()`, `measure_vertical()` or `measure_section()` for the
## other four.
##
## **One context, not six.** All six are "the Measure tool is armed and has a
## reading"; a `CTX_MEASURE_AREA` and four siblings would each need their own
## title, their own readout branch and their own dispatch arm for what is one
## selection with six presentations.
func show_measure(result: Dictionary, mode: String = "distance") -> void:
	_context = CTX_MEASURE
	_measure_result = result
	_measure_mode = mode
	_rebuild()

## Called by `GlobalTools` when a Region marquee commits -- `result` is
## `region_get()`'s own dict.
func show_region(result: Dictionary) -> void:
	_context = CTX_REGION
	_region_result = result
	_rebuild()

## Called by `app.gd` on a map click while the Wildlife debug view is drawn
## -- `rec` is `bridge.wildlife_region_at()`'s own dict, straight through.
## An empty dict is the reference's own `hideWildInfo()`: the click missed
## every marker, so the dock falls back to Sample rather than keeping a
## stale roster on screen.
func show_wildlife(rec: Dictionary) -> void:
	_wildlife_region = rec
	_context = CTX_WILDLIFE if not rec.is_empty() else CTX_SAMPLE
	_rebuild()

## Called by `world_workspace.gd`, `tool_bar.gd` and `menus.gd` whenever
## something has changed the sculpt draft, and by this file's own stack rows --
## a stroke ending, a commit, a discard, an undo, a Select All. Never on a bare
## cursor move.
##
## **A plain rebuild, and no longer a context setter.** It used to assign
## `_context = CTX_SCULPT`, which made arming Sculpt replace whatever the dock
## was showing -- the shape the owner's 2026-09-03 ruling rejects, measured
## doing it with a settlement selected. Whether the Stamp stack draws is now
## `_tool_section()`'s answer, derived from the armed tool and the draft on
## every rebuild; see `TOOL_STAMPS` for the derivation and the measurement.
##
## Kept as a method rather than deleted, exactly as `leave_paint_context()`
## was: five files call it by name, and a redraw is what every one of them is
## actually asking for. `_build_sculpt` already read the stack fresh from the
## bridge on every `_rebuild()`, so this never carried data of its own the way
## `show_measure`/`show_region` do and nothing is lost by the change.
func show_sculpt_stack() -> void:
	_rebuild()

## Called from `app.gd`'s `_on_workspace_changed` when the new domain is not
## WORLD. Sculpt is a World-only tool (`world_workspace.gd`), and before the
## 2026-09-03 ruling this was the only thing that ever cleared the Stamp
## stack's context -- a visual sweep on 2026-08-20 found the dock stuck showing
## it in CIVIL and CARTOGRAPHY without it.
##
## **A plain rebuild now, because there is no context left to clear.**
## `_tool_section()` carries that same `id != "world"` condition as
## `TOOL_STAMPS`' own domain gate, so the section stops drawing on a domain
## switch by derivation rather than by a flag somebody has to remember to
## reset. Kept as a method for the reason `leave_paint_context()` is: `app.gd`
## calls it by name, and a redraw at a domain switch is what it is asking for.
##
## Settlement / route / faction / measure / region selections are still left
## untouched -- those stay meaningful across a domain switch (Inspect's own
## selection is wired domain-independently in `app.gd`'s `_wire_selection`).
func leave_sculpt_context() -> void:
	_rebuild()

## Called by `journey_planner_view.gd` when the JOURNEY tool arms -- hands
## this dock the delegate it draws the results panel through
## (`JOURNEY_PLANNER_SPEC.md` §8), rather than duplicating that rendering here.
## Mirrors `show_sculpt_stack()`'s own delegation shape (that one re-reads
## `bridge.sculpt_*` fresh each rebuild; this one re-reads `view`'s own cached
## compute result, since a fresh `jp_compute()` per rebuild would be a wasted
## boundary crossing on every unrelated `right_dock.gd` refresh).
##
## **Arms the appended Journey section; it does not claim `_context`.** Whatever
## the dock was showing -- a selected settlement, a route, a river -- is still
## showing after this call, with the plan below it. That is the owner's
## 2026-09-04 extension of the dock ruling; see `TOOL_JOURNEY`'s own doc for
## the measurement that caught this replacing the selection instead, and for
## the second half of that defect, which was in `build_results()`.
##
## The delegate is the only thing stored, exactly as `show_paint()` stores
## `_paint_ctx_layer`: *whether* the section draws is `_tool_section()`'s
## answer, derived from the armed tool and the domain on every rebuild.
## `Edit ▸ Undo history…` (`GUI_GAP_REGISTER.md` **ED-02**) -- claims this
## dock for the ledger. No data of its own: `_build_history` reads
## `bridge.undo_ledger()` fresh on every rebuild, the same shape
## `show_sculpt_stack()` uses, because the answer changes on every commit and
## a cached copy would be one more thing to invalidate.
func show_history() -> void:
	_context = CTX_HISTORY
	_rebuild()

## Called after any revert, so the rows and the budget both move. A no-op
## unless History is the live context.
func refresh_history() -> void:
	if _context == CTX_HISTORY:
		_rebuild()

func show_journey(view: JourneyPlannerView) -> void:
	_journey_view = view
	_rebuild()

## Called by `journey_planner_view.gd` after every recompute, while the
## appended Journey section is actually on screen -- a no-op otherwise
## (Journey disarmed, or the domain switched away under it), so a recompute
## that lands after a disarm cannot force a redraw of a section that is gone.
##
## **Gated on `_tool_section()`, not on `_context`.** Since 2026-09-04 the
## planner does not own `_context` at all, and a selection can be live
## underneath it; asking the derivation is asking the one thing that decides
## whether `_append_tool()` will draw this section, so the two cannot drift.
func refresh_journey() -> void:
	if _tool_section() == TOOL_JOURNEY:
		_rebuild()

## Called by `journey_planner_view.gd` when the JOURNEY tool disarms --
## drops the delegate so the appended results section stops drawing, and
## leaves whatever the dock was *selecting* exactly where it was.
##
## **It no longer resets `_context` to `CTX_SAMPLE`, and that is the point of
## the conversion rather than an omission.** While Journey was a context,
## disarming it had to name a context to go back to, and Sample was the only
## honest answer -- which threw away a settlement/route/river selection the
## user had made before arming the planner. Now the selection was never
## touched on the way in, so there is nothing to restore on the way out.
##
## Unconditional, because `_tool_section()` has already stopped answering
## `TOOL_JOURNEY` by the time the disarm path reaches here in the domain-switch
## case (`_hide()` runs off `workspace_changed`), so a guard reading the
## derivation would skip the one clear that matters. Clearing a null field and
## rebuilding is idempotent.
func clear_journey() -> void:
	_journey_view = null
	_rebuild()

## Called by `world_workspace.gd`'s own `_on_tool_armed` when `"paint"` arms,
## and again after every layer switch / commit / discard / stroke release --
## the same "re-announce the current picture" cadence `show_sculpt_stack()`
## already uses, since this dock keeps no draft of its own to patch in place.
## `on_pick_value`, when given, is bound to a closure that knows the other
## four `Brush` fields this file cannot see; omitted (or an invalid
## `Callable`), the legend's rows still show swatch/label/count, just not a
## click-to-arm affordance -- see `_build_paint`.
##
## **Arms the appended Paint section; it does not claim `_context`.** Whatever
## the dock was showing -- a selected settlement mid-edit, a route, a river --
## is still showing after this call, with Paint below it. That is the owner's
## 2026-09-03 ruling; see `TOOL_PAINT`'s own doc for the measurement that
## caught this replacing the selection instead.
func show_paint(layer: String, on_pick_value: Callable = Callable()) -> void:
	if layer != "":
		_paint_ctx_layer = layer
	_paint_on_pick = on_pick_value
	_rebuild()

## Mirrors `leave_sculpt_context()` exactly -- called from `app.gd`'s
## workspace-switch handler for the same reason: Biome paint is a WORLD-only
## tool. Named `..._context` because that is what every call site already says.
##
## **A plain rebuild, because whether the section draws is `_tool_section()`'s
## answer and not a flag this could clear.** Kept as a method rather than
## deleted: `app.gd` calls it by name, and a redraw at a domain switch is
## exactly what it was asking for.
func leave_paint_context() -> void:
	_rebuild()

## `rdMode4()` rule 6: `domain==='CARTO' && tool==='inspect'`. Two call sites
## need this, and neither alone covers the rule -- `cartography_workspace.gd`'s
## `_on_any_tool_armed` (armed tool changes, but a domain switch fires no
## `tool_armed` of its own) and `app.gd`'s `_on_workspace_changed` (domain
## changes, but arming Inspect while already in CARTO fires no workspace
## change). Both are wired to call this only when both halves of the rule
## already hold.
##
## **This file re-checks them anyway now, and that sentence used to say it did
## not.** `_tool_section()` reads the same `armed_tool == "inspect" && domain
## == "cartography"` rule directly, so these two calls are re-announcements --
## a redraw at the moment the answer could have changed -- and not the thing
## that decides. Trusting the callers is what left the Paint section on screen
## after its tool disarmed: the rule had a third case neither caller covered.
func show_stops() -> void:
	_rebuild()

## Drops the ramp-stop *selection* -- the one piece of state this section owns
## that outlives a rebuild -- and redraws. Whether the section itself draws is
## `_tool_section()`'s answer, which already reads the same domain-and-tool rule
## both of this method's call sites check before calling it.
func leave_stops_context() -> void:
	if _stops_selected != -1:
		_stops_selected = -1
	_rebuild()

## Called by `cartography_workspace.gd`'s `_on_any_tool_armed` for both
## `"label"` and `"icon"` -- one section for both tools, matching §1.3's own
## right-dock title table (`ANNOTATION` names neither tool by itself) and
## `_append_tool()`'s single `TOOL_ANNO` branch.
func show_anno() -> void:
	_rebuild()

func leave_anno_context() -> void:
	_rebuild()

## Called by `civilization_workspace.gd` on arming `"territory"`, on every
## faction re-pick while it stays armed, and after a commit/discard -- the
## live stats this shows (`civ_faction_territory_stats`) only change at
## commit, but re-announcing costs nothing and keeps this in step with
## whichever faction is actually armed.
func show_territory(faction_id: int) -> void:
	_terr_faction = faction_id
	_rebuild()

func leave_territory_context() -> void:
	_rebuild()

## `rdMode4()` rule 7. Called by `infrastructure_workspace.gd::_push_way_draft()`
## whenever the Way or Route draft changes -- arm, every click, a way-type
## change, commit, discard, and the disarm that hands the tool back. `owner_tool`
## is `"way"`, `"route"` or `""`; the last is how neither-armed arrives and
## clears what this dock remembers.
##
## **Arms the appended Way section; it does not claim `_context`.**
## `show_paint()`'s shape exactly -- whatever the dock was showing is still
## showing, with the draft's readings below it.
##
## `points` is duplicated rather than aliased: `_way_points` is appended to in
## place on the next click, and a dock reading a live buffer would be a second
## owner of the caller's state.
func show_way(owner_tool: String, points: PackedVector2Array, kind_label: String) -> void:
	_way_owner = owner_tool
	_way_draft = points.duplicate()
	_way_kind = kind_label
	_rebuild()

## Drops the remembered draft without a rebuild -- the two world-replacement
## handlers in `setup()` rebuild once at the end of their own work, and this is
## called from inside both.
func _forget_way_draft() -> void:
	_way_owner = ""
	_way_draft = PackedVector2Array()
	_way_kind = ""

# -- Dispatch ---------------------------------------------------------------

## §6's own per-context header title, mirroring `DccWidgets.section()`'s title
## one call below it -- kept as one table here rather than a `match` inline in
## `_rebuild()` so a new `CTX_*` can't add a body section without this table
## reminding whoever adds it that the dock chrome needs the same name.
##
## **No row here for the seven `TOOL_*` sections, by ruling.** §1.3 gives each
## of them a right-dock title (`STAMP STACK`, `PAINT · BIOME`, `RAMP · STOPS`,
## `ANNOTATION`, `TERRITORY`, `WAY`, `JOURNEY — RESULTS`), and those titles are real -- they are drawn by each section's
## own `DccWidgets.section()` header inside the body, where an appended section
## puts its name. Putting one in this table instead is how the dock came to
## rename itself the moment a tool armed, which is the replacement the owner's
## 2026-09-03 ruling rejects. The header names the selection; the section names
## the tool; neither overwrites the other, and the header no longer moves at
## all when a tool arms or disarms.
const CTX_TITLES := {
	CTX_SETTLEMENT: "Settlement", CTX_ROUTE: "Route", CTX_RIVER: "River",
	CTX_FACTION: "Faction", CTX_MEASURE: "Measure", CTX_REGION: "Region select",
	CTX_WILDLIFE: "Ecoregion", CTX_HISTORY: "History",
}

func _rebuild() -> void:
	if app == null:
		return
	var body := app.right_dock_body
	for child in body.get_children():
		body.remove_child(child)
		child.queue_free()
	_sample_pos = null
	_sample_cell = null
	_sample_elev = null
	_sample_nearest = null
	_sample_stale_note = null
	_sample_rows.clear()
	## The selection, then the armed tool below it, then Layers below that.
	## The order is the whole of the owner's ruling -- see `_append_tool()`.
	_dispatch(body)
	_append_tool(body)
	## **After** the dispatch, deliberately -- see `_append_layers()`.
	_append_layers(body)
	app.set_right_dock_title(_current_title())
	_push_dock_readout()

## `_build_settlement` falls back to `_build_sample()` when its own data is
## missing (a settlement deselected out from under the dock) -- mirrored here
## so the header never claims a context the body didn't actually draw.
##
## **`CTX_JOURNEY` had the second clause here and no longer needs one.** The
## planner is an appended section since 2026-09-04 (`TOOL_JOURNEY`), so it
## cannot name this header at all; a missing `_journey_view` now means the
## section is simply absent, and the header goes on naming the selection.
##
## Reads `_context` only. `_tool_section()` deliberately cannot reach this -- see
## `CTX_TITLES`' own note directly above.
func _current_title() -> String:
	if _context == CTX_SETTLEMENT and _settlement_data == null:
		return "Sample"
	return String(CTX_TITLES.get(_context, "Sample"))

## RD-11: §6's own last line -- "elevation for Sample, layer dots for
## Layers, stamp count for the stack" -- is the right dock's collapsed
## primary readout, and `DccShell.set_dock_readout("right", …)` already
## exists for it, kept current whether or not the dock is actually
## collapsed (`dcc_shell.gd`'s own doc comment). `world_workspace
## ._push_dock_readout()` calls the left dock's equivalent on every
## rebuild; this dock never called the right-dock one at all. So this reads
## one honest number per context.
##
## **There is still no `Layers` arm, and after RD-10 that is a decision rather
## than an omission.** The owner's 2026-09-03 ruling made Layers an *appended
## section* (`_append_layers()`), not a context: a selected settlement keeps the
## dock and the layer rows arrive under it. Reporting layer dots collapsed would
## therefore mean overwriting the readout of whatever the selection is -- which
## is the replacement the ruling rejects, one line lower down. §6's "layer dots
## for Layers" is a line about a context this dock deliberately does not have.
##
## **The armed tool obeys the same rule, and the design says nothing further.**
## §6 lists one readout per context and the five tool sections are no longer
## contexts, so there is no delivered answer for "a settlement is selected AND
## Paint is armed" -- that pairing could not occur in the shape §6 was written
## for. Stated rather than guessed: **the selection's readout wins whenever
## there is a selection.** The tool's own figure fills only the slot the
## selection was never using -- the `_:` default, which is Sample, and the two
## arms that already fall back to it -- so a tool armed with nothing selected
## keeps reporting exactly what it reported before this change (painted cells,
## ramp-stop count, label/icon counts, claimed cells, stamp count, draft
## waypoints) and a tool armed
## *over* a selection cannot overwrite it. See `_fallback_readout()`.
##
## §6's "stamp count for the stack" is therefore still delivered, and is the
## one line of §6 that changed side: it used to be a context arm, and the Stamp
## stack stopped being a context on 2026-09-03 (`TOOL_STAMPS`). The number is
## the same one; what changed is that a selected settlement now keeps the
## readout while the stack is on screen beneath it.
func _push_dock_readout() -> void:
	if app == null:
		return
	app.set_dock_readout("right", _dock_readout_text())

## The readout when `_context` is carrying no selection of its own: the armed
## tool's own figure, else Sample's elevation. Two call sites, which is the
## point -- CTX_SETTLEMENT degrades to Sample when its data is missing
## (`_current_title()` mirrors that), so both have to give an armed tool the
## same slot or the readout would change on a deselect for no reason the reader
## could see. **It was three until 2026-09-04**: `CTX_JOURNEY` had the same
## degrade-to-Sample arm, and the planner is an appended section now, so its
## figure arrives through `TOOL_JOURNEY` below instead -- which is the same
## slot, reached by the same rule (*the selection's readout wins whenever there
## is a selection*), for one fewer context.
func _fallback_readout() -> String:
	match _tool_section():
		TOOL_PAINT:
			return ("%s cells" % _thousands(float(bridge.paint_painted_counts().get("total", 0)))) if bridge.has_world else "no world"
		TOOL_STOPS:
			var n := bridge.color_ramp().size() if bridge.ramp_api else 0
			return ("%d stop%s" % [n, "" if n == 1 else "s"]) if n > 0 else "no ramp"
		TOOL_ANNO:
			return "%d labels · %d icons" % [bridge.label_list().size(), bridge.icon_list().size()]
		TOOL_TERR:
			var stats := bridge.civ_faction_territory_stats(_terr_faction) if _terr_faction >= 0 else {}
			return ("%s cells" % _thousands(float(stats.get("claimed_cells", 0)))) if not stats.is_empty() else "no claim"
		TOOL_WAY:
			## The same figure the tool options bar reports, in each tool's own
			## noun -- `_tool_options_way()` says "waypoints", `_tool_options_
			## route()` says "stops", and a collapsed dock should not rename
			## what the bar beside it is already calling the thing.
			var pts := _way_draft.size()
			var noun := "waypoint" if _way_owner == "way" else "stop"
			return "%d %s%s" % [pts, noun, "" if pts == 1 else "s"]
		TOOL_JOURNEY:
			## `_tool_section()` answers `TOOL_JOURNEY` only with a non-null
			## delegate (its own third clause), so this cannot be reached
			## without one -- the same guarantee `TOOL_WAY` above relies on for
			## `_way_draft`. The plan's own one-line figure, not a second
			## opinion about it: `journey_planner_view.gd::readout_text()` is
			## the single reader of `_last_result` for this purpose.
			return _journey_view.readout_text()
		TOOL_STAMPS:
			## §6's own "stamp count for the stack", moved here verbatim from the
			## `CTX_SCULPT` arm it used to sit in. `sculpt_stamp_count()` rather
			## than `sculpt_list_stamps().size()`: the same number without
			## marshalling every stamp dictionary to ask for it.
			var n := bridge.sculpt_stamp_count()
			return ("%d stamp%s" % [n, "" if n == 1 else "s"]) if bridge.has_world else "no world"
	return _sample_elev.text if _sample_elev != null else "—"

func _dock_readout_text() -> String:
	match _context:
		CTX_SETTLEMENT:
			if _settlement_data == null:
				return _fallback_readout()
			return String((_settlement_data as Dictionary).get("name", "—"))
		CTX_ROUTE:
			return _route_length_text(_route_entry.get("points", PackedVector2Array()))
		CTX_RIVER:
			## Length, matching CTX_ROUTE's own readout: the two contexts
			## describe the same kind of thing (a line on the map with a real
			## routed length) and a reader collapsing the dock should get the
			## same number from both.
			if _river.is_empty():
				return "no river"
			return DccUnits.format_adaptive(float(_river.get("km", 0.0)))
		CTX_FACTION:
			var culture := ""
			for f in bridge.get_factions():
				var d: Dictionary = f
				if int(d.get("id", -1)) == _faction_id:
					culture = String(d.get("culture", ""))
					break
			return ("%d · %s" % [_faction_id, culture.capitalize()]) if culture != "" else "faction %d" % _faction_id
		CTX_MEASURE:
			return _measure_readout()
		CTX_REGION:
			return ("%d cells" % int(_region_result.get("cell_count", 0))) if not _region_result.is_empty() else "no region"
		CTX_WILDLIFE:
			return ("%d species" % int(_wildlife_region.get("richness", 0))) if not _wildlife_region.is_empty() else "no ecoregion"
		CTX_HISTORY:
			var st := bridge.undo_stats()
			return "%d of %d reversible" % [int(st.get("depth", 0)), bridge.undo_ledger().size()]
		_:
			return _fallback_readout()

## Named rather than inlined in `_rebuild()` -- a `match` cannot be the tail
## statement of a lambda closed with `)` in this GDScript version, and this
## keeps `_rebuild()`'s teardown loop and this dispatch legible as two
## separate concerns anyway.
func _dispatch(body: Control) -> void:
	match _context:
		CTX_SETTLEMENT:
			_build_settlement(body)
		CTX_ROUTE:
			_build_route(body)
		CTX_RIVER:
			_build_river(body)
		CTX_FACTION:
			_build_faction(body)
		CTX_MEASURE:
			_build_measure(body)
		CTX_REGION:
			_build_region(body)
		CTX_WILDLIFE:
			_build_wildlife(body)
		CTX_HISTORY:
			_build_history(body)
		_:
			_build_sample(body)

# -- The armed tool's appended section (`LARGE_ITEM_RULINGS.md`, 2026-09-03) --
#
# The owner's ruling on this dock, verbatim: *"Selection wins; the tool appends
# a section."* An armed tool adds its own section **below** whatever the dock is
# showing rather than replacing it -- the ruling's own reasoning being that
# "merging naively makes the dock flip away from a selected settlement the
# moment a tool arms". So this runs after `_dispatch()`, never instead of it,
# and touches neither `_context` nor the dock header.
#
# **This is not a second `_dispatch()`.** `_dispatch()` answers *what is
# selected* from `_context`, and exactly one of its arms draws. This answers
# *what is armed*, from `app.armed_tool` -- an independent question with an
# independent source; both answers are on screen at once, which is the whole
# point. `_append_layers()` below is the same shape with a one-valued question,
# and was this ruling's first worked example.
#
# **No selection is needed for a section to append.** `_dispatch()`'s default
# arm draws Sample, so a tool armed with nothing selected still gets its own
# section -- under the cursor readout, which is a useful pairing while painting
# rather than a fallback being tolerated.

## Which section `_append_tool()` will draw, or `""`. **`rdMode4()`'s own
## fall-through table (§1.2b) read live, and nothing else** -- rules 3
## (`label`/`icon`) and 4 (`territory`) are unconditional on the armed tool,
## rules 1 (`sculpt`) and 6 (`inspect`) also read the domain, rule 7
## (`way`/`route`) also reads the draft, and rule 8 (`journey`) reads the
## domain and the delegate. See the `TOOL_*` block at the top of
## this file for why this is derived rather than remembered.
##
## **One section at a time, which is the table's own shape**: `rdMode4()` is
## first-match-wins and returns a single mode, so a tool that matches two rules
## cannot draw two panels. The arms below are mutually exclusive on
## `app.armed_tool` anyway; only `TOOL_STAMPS`' draft clause is not keyed on
## the tool, and it is deliberately reached last so an explicitly armed tool
## always beats a draft left lying in the WORLD domain.
##
## `_rebuild()` therefore has to run on every tool change, which is why
## `setup()` connects `app.tool_armed` -- the five `show_*` calls cover the
## arms, but nothing covered a *disarm* that stayed inside the same domain.
##
## **Paint carries a domain condition §1.2b's own table does not give it**, and
## that is behaviour preservation rather than a new rule: `app.gd`'s
## `_on_workspace_changed` called `leave_paint_context()` on every switch away
## from WORLD, "Biome paint is a WORLD-only tool", and `armed_tool` survives a
## domain switch (nothing in the shell re-arms Inspect on one -- checked
## against every `arm_tool(` call site). Without this the Paint section would
## follow the still-armed tool into CIVIL, which is one thing the old code
## demonstrably did not do. Territory and Annotation get no such condition for
## the same reason in reverse: the old code never cleared them on a domain
## switch either, and rules 3 and 4 say not to.
func _tool_section() -> String:
	if app == null:
		return ""
	match app.armed_tool:
		"paint":
			return TOOL_PAINT if app.active_domain() == "world" else ""
		"territory":
			return TOOL_TERR
		"label", "icon":
			return TOOL_ANNO
		"inspect":
			if app.active_domain() == "cartography":
				return TOOL_STOPS
		"way", "route":
			## Rule 7's own `wy().draft.length > 0`. **`_way_owner` must be the
			## tool that is armed right now**, not merely non-empty: arming Route
			## while a Way draft is live commits the way
			## (`infrastructure_workspace.gd::_on_infra_tool_armed`), and that
			## handler and this dock's own `tool_armed` rebuild are two
			## connections to one signal -- whichever runs first, this comparison
			## is what stops the committed way's points being drawn under the
			## Route tool for a rebuild.
			if _way_owner == app.armed_tool and _way_draft.size() > 0:
				return TOOL_WAY
		"journey":
			## Rule 8, in this port's own vocabulary -- see `TOOL_JOURNEY` for
			## why the condition is `armed_tool`/`active_domain()` rather than
			## §1.2b's `civCat === 'planner'`. Both halves are
			## `journey_planner_view.gd::_recompute_visibility()`'s own, so the
			## section is on screen exactly while the planner's panels are.
			##
			## **No `else` and no `return ""`, deliberately**: `"inspect"` above
			## does the same, and for the same reason. Journey armed while the
			## domain has moved to WORLD means the planner is hidden and a live
			## sculpt draft below is the honest answer, so this arm falls
			## through to the draft clause exactly as `"inspect"` does.
			if app.active_domain() == "civilization" and _journey_view != null:
				return TOOL_JOURNEY
	## Rule 1, and its draft clause -- see `TOOL_STAMPS`. Reached after the
	## `match` on purpose: the clause is not keyed on the armed tool at all, so
	## it cannot be an arm of a `match` over `app.armed_tool`. `sculpt` itself
	## is answered here too rather than in the `match`, so the tool half and the
	## draft half share one domain gate instead of stating it twice.
	if app.active_domain() != "world" or bridge == null:
		return ""
	if app.armed_tool == "sculpt" or bridge.sculpt_stamp_count() > 0:
		return TOOL_STAMPS
	return ""

func _append_tool(body: Control) -> void:
	var section := _tool_section()
	match section:
		TOOL_PAINT:
			_build_paint(body)
		TOOL_STOPS:
			_build_stops(body)
		TOOL_ANNO:
			_build_anno(body)
		TOOL_TERR:
			_build_territory(body)
		TOOL_STAMPS:
			_build_sculpt(body)
		TOOL_WAY:
			_build_way(body)
		TOOL_JOURNEY:
			_build_journey(body)
	## An uncommitted draft keeps its own controls whatever else is armed.
	##
	## `_tool_section()` answers with exactly ONE id, and its `match` reaches
	## `paint`/`territory`/`label`/`icon` (and now `way`/`route`) before the
	## draft clause -- so arming any of those took Commit, Discard, Undo and
	## Redo away from a draft
	## the user had not committed. Worse than merely hidden: Paint draws its own
	## Commit/Discard in that slot, so the user was shown a Commit belonging to
	## a different draft.
	##
	## That is the owner's 2026-09-03 ruling breaking in the one place it most
	## matters ("nothing is yanked away, so no *is editing* signal is needed"),
	## and it is what HEAD already did before rule 1 was converted -- the stack
	## came from `_dispatch()` then, so it and the tool section were both on
	## screen. The disarm path was guarded and the arm-another-tool path was not.
	## Caught by a verifier, not by the conversion.
	if section != TOOL_STAMPS and _draft_stack_live():
		_build_sculpt(body)

## True while a sculpt draft is uncommitted and reachable -- the condition the
## draft clause in `_tool_section()` carries, named once so the two cannot
## drift. Domain-gated exactly as that clause is: a draft is world-domain work.
func _draft_stack_live() -> bool:
	if app == null or bridge == null:
		return false
	if app.active_domain() != "world":
		return false
	return bridge.sculpt_stamp_count() > 0

# -- Layers (`GUI_GAP_REGISTER.md` RD-10) -----------------------------------
#
# §6's `Layers` context -- "ordered list with visibility dot, name, opacity bar,
# blend mode; nested children under Terrain" -- built as an **appended section**
# and deliberately not as a `CTX_LAYERS`.
#
# That is the owner's 2026-09-03 ruling on this dock, verbatim
# (`LARGE_ITEM_RULINGS.md`): *"Selection wins; the tool appends a section."* The
# dock keeps showing the selected entity and the layer rows arrive **below** it.
# So there is no context constant, no `CTX_TITLES` row and no `_dispatch()` arm:
# every one of those would be the replacement the ruling explicitly rejects, and
# nothing is yanked away from a user mid-edit.
#
# Drawn while the CARTO domain is active, which is `show_stops()`'s own trigger
# (`rdMode4()` rule 6 reads the domain, not a selection). A settlement inspector
# open in WORLD has no use for the terrain raster's compositing order.
#
# **The editor is the left dock, not this.** `render_workspace.gd`'s
# `_build_layer_stack()` owns the opacity slider and the blend picker; this is
# §6's ordered list, so the two continuous values are readouts here and the two
# discrete ones -- visibility and order -- are live. Two sliders over one
# `set_layer_stack` is the "two pickers over one concept" shape this shell has
# had to undo three times. Both halves read the engine fresh and both write
# through `EngineBridge.set_layer_stack`, whose `layer_stack_changed` signal
# rebuilds the other, so they cannot drift apart in either direction.

func _append_layers(body: Control) -> void:
	if app == null or bridge == null or not bridge.layer_stack_api:
		return
	if app.active_domain() != "cartography":
		return
	var rows: Array = bridge.layer_stack()
	if rows.is_empty():
		return
	var sec := DccWidgets.section(body, "Layers")
	for i in rows.size():
		_layer_row(sec, rows[i] as Dictionary, i, rows.size())
	DccWidgets.note(sec,
		"The terrain raster's three categories, top drawn last. Opacity and "
		+ "blend are set in Cartography - Layers; the dot and the order are live "
		+ "here.")
	DccWidgets.action(sec, "Layer properties...",
		func(): app.select_domain_category("cartography", "Layers"))

## One row: dot, name, opacity bar, blend name, and the two reorder buttons
## WCAG 2.2 SC 2.5.7 requires beside the left dock's drag.
func _layer_row(parent: Control, d: Dictionary, index: int, count: int) -> void:
	## Read through `has()`, never `get(k, default)`: a row that arrived without
	## `visible` must not draw as a hidden layer, and one without `opacity` must
	## not draw an empty bar. Both are indistinguishable from the real thing.
	var missing: Array = []
	for k in ["id", "label", "visible", "opacity", "blend"]:
		if not d.has(k):
			missing.append(k)
	if not missing.is_empty():
		DccWidgets.note(parent, "Layer %d is unreadable - no %s." % [index, ", ".join(missing)])
		return

	var id := String(d["id"])
	var shown := bool(d["visible"])
	var tablet := DccTheme.is_tablet()
	var fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_TINY
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 20

	var dot := Button.new()
	dot.flat = true
	dot.focus_mode = Control.FOCUS_NONE
	dot.text = DccIcons.SYMBOLS["on"] if shown else DccIcons.SYMBOLS["off"]
	dot.add_theme_font_override("font", DccTheme.mono(0))
	dot.add_theme_font_size_override("font_size", fs)
	dot.add_theme_color_override("font_color",
		DccTheme.c("text") if shown else DccTheme.c("text_ghost"))
	dot.tooltip_text = "%s %s." % ["Hide" if shown else "Show", String(d["label"])]
	if tablet:
		dot.custom_minimum_size = Vector2(DccTheme.role_px("row_min_h"), DccTheme.role_px("row_min_h"))
	dot.pressed.connect(func(): _set_layer_key(id, "visible", not shown))
	row.add_child(dot)

	var name_label := DccTheme.mono_label(String(d["label"]),
		"text" if shown else "text_ghost", fs)
	name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_label.clip_text = true
	row.add_child(name_label)

	## §6's "opacity bar" -- a readout, since the slider lives in the left dock.
	var opacity := clampf(float(d["opacity"]), 0.0, 1.0)
	var bar := ProgressBar.new()
	bar.show_percentage = false
	bar.min_value = 0.0
	bar.max_value = 1.0
	bar.value = opacity
	bar.custom_minimum_size = Vector2(44, 4)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.add_theme_stylebox_override("background", DccTheme.outline("border"))
	bar.add_theme_stylebox_override("fill",
		DccTheme.outline("accent" if shown else "text_ghost",
			"accent" if shown else "text_ghost", 0))
	bar.tooltip_text = "Opacity %.2f. Set it in Cartography - Layers." % opacity
	row.add_child(bar)

	var blend := DccTheme.mono_label(String(d["blend"]), "text_dim", fs)
	blend.tooltip_text = "Blend mode. Set it in Cartography - Layers."
	row.add_child(blend)

	_reorder(row, "Up", fs, tablet, index > 0, index, index - 1)
	_reorder(row, "Down", fs, tablet, index < count - 1, index, index + 1)
	parent.add_child(row)

func _reorder(parent: Control, text: String, fs: int, tablet: bool,
		enabled: bool, from: int, to: int) -> void:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.text = text
	b.disabled = not enabled
	b.tooltip_text = "Move this layer one place %s the stack." % text.to_lower()
	b.add_theme_font_size_override("font_size", fs)
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	if tablet:
		b.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	b.pressed.connect(func(): _move_layer(from, to))
	parent.add_child(b)

## One gesture, one key. An absent key means *unchanged* at the boundary, so the
## row carries its id and nothing else -- restating the values this dock only
## reads would let it overwrite an opacity the left dock had just set.
func _set_layer_key(id: String, key: String, value: Variant) -> void:
	var out: Array = []
	for r in bridge.layer_stack():
		var d: Dictionary = r
		var row := {"id": String(d.get("id", ""))}
		if String(d.get("id", "")) == id:
			row[key] = value
		out.append(row)
	_write_layers(out)

## The new order, sent as data -- the engine decides what the stack is and both
## docks are rebuilt from its answer. Nothing here reorders its own children.
func _move_layer(from: int, to: int) -> void:
	var ids: Array = []
	for r in bridge.layer_stack():
		ids.append(String((r as Dictionary).get("id", "")))
	if from < 0 or from >= ids.size() or to < 0 or to >= ids.size() or from == to:
		return
	var moved: String = ids[from]
	ids.remove_at(from)
	ids.insert(to, moved)
	var out: Array = []
	for id in ids:
		out.append({"id": id})
	_write_layers(out)

## 3 or 0 -- all three rows or none, nothing changed on a refusal. The repaint
## and the rebuild both hang off `layer_stack_changed`, which
## `EngineBridge.set_layer_stack` emits only on success, so a refusal costs
## exactly one warning and no redraw.
##
## Returns whether it was accepted. Nothing in this file reads that today; it is
## returned because otherwise the refusal branch's only effect is a log line and
## no probe can tell the two apart -- mutation testing scored `!= 3` SURVIVED
## while it returned `void`.
func _write_layers(rows: Array) -> bool:
	if bridge.set_layer_stack(rows) != 3:
		push_warning("Layers: the engine refused the stack; nothing changed.")
		return false
	return true

# -- Sample -------------------------------------------------------------

func _build_sample(body: Control) -> void:
	var sec := DccWidgets.section(body, "Sample")
	var valid: bool = bridge.has_world
	_sample_pos = _field(sec, "Position", "—",
		"Cursor position in km from the map's north-west corner, X then Y. " +
		"Printed to %d decimal%s for this world: a cell is %s across, and no " %
			[_coord_decimals(), "" if _coord_decimals() == 1 else "s", _cell_km_text()] +
		"reading in this port distinguishes two points inside one cell, so the " +
		"step shown is the largest power of ten that still fits inside one.",
		true, true)
	_sample_cell = _field(sec, "Cell", "—",
		"The raster index every other row in this panel is read at, X then Y. " +
		"Live once the cursor is over a generated map.", true, true)

	_sample_elev = _accent_readout(sec, "Elevation", "—",
		"Metres above sea level at the cursor cell, from WorldState::field through " +
		"metersPerUnit()'s own anchoring (1 - seaLevel maps to peak altitude). " +
		"Negative below the waterline, which is the honest reading for an ocean cell.")

	## §1.4's staleness gate. Read once for the whole panel -- `_stale_now()`
	## caches, but a dozen calls to it would still be a dozen dictionary lookups
	## per rebuild for one answer that cannot change between rows.
	var stale := _stale_now()
	for f in SAMPLE_FIELDS:
		_sample_rows[f["label"]] = _field(sec, f["label"], "—",
			_tip_with(String(f["tip"]), _stale_reason(f["label"], stale)), false)

	## "Nearest settlement" was this dock's single widest row -- and its value
	## changes on every mouse-move, which is what made the whole pane breathe
	## (see `_field()`). The label is shortened and its column narrowed so the
	## name and its distance both still fit inside the pane's own width instead
	## of pushing against it.
	var nearest_why := _stale_reason("Nearest", stale)
	_sample_nearest = _field(sec, "Nearest", "—",
		_tip_with(_NEAREST_TIP, nearest_why),
		valid and nearest_why == "", false, 60)

	## §1.4's footnote, verbatim: *"fields owned by stale stages read —"*. Until
	## now the reason a row dashed lived only in that row's tooltip, which is
	## hover-only and therefore absent on the tablet composition this same dock
	## serves -- so on a touch device a stale panel was a panel of em dashes with
	## no explanation anywhere on screen.
	##
	## **Drawn only while something actually is stale, and it names what.** The
	## prototype draws it unconditionally, and that is wrong for this panel rather
	## than a difference worth copying: the two rows immediately below it ("Route
	## cost", "E-W profile") dash permanently for reasons that have nothing to do
	## with staleness, and an always-on sentence about stale stages would be read
	## as their explanation. It also sits **above** them for the same reason --
	## directly under the rows it actually describes. See `_stale_footnote_text()`.
	_sample_stale_note = DccWidgets.note(sec, _stale_footnote_text(stale))
	_sample_stale_note.visible = _sample_stale_note.text != ""

	## §6's no-selection list has two more entries than the rows above, and
	## both were simply absent rather than disclosed (2026-08-20 menu-structure
	## audit). Drawn as permanently-dashed rows with their real reason, the
	## same shape every other unavailable field in this panel already takes.
	_field(sec, "Route cost", "—",
		"§6 lists it; nothing computes a per-cell traversal cost. cartalith-civ's route " +
		"cost lives inside the Journey Planner's own Dijkstra (jp_plan) and is per-LEG, " +
		"over a chosen party and season -- it has no meaning at one cell with no journey " +
		"around it, and no #[func] evaluates the cost surface pointwise. Plan a journey " +
		"(Data ▸ Journey planner, ⇧J) for the real figure.", false)
	_field(sec, "E–W profile", "—",
		"§6's elevation profile through the cursor's row. Every input exists -- sample_cell " +
		"reads any cell -- but drawing it means one call per column on every mouse move " +
		"(1 000-4 000 boundary crossings per frame at working resolution), and there is no " +
		"row-slice #[func] to fetch the whole scanline in one call. A real gap in the " +
		"binding surface, not in the data.", false)

	if not valid:
		DccWidgets.note(sec, "No world generated -- every field goes live once one exists.")
	elif bridge.sample_cell(0, 0).is_empty():
		## A loaded save has no `WorldSource::Generated` behind it, so
		## `sample_refs()` returns nothing and the whole panel stays dashed.
		## Said out loud rather than left looking broken.
		DccWidgets.note(sec,
			"This world was loaded from a save, which carries none of the substrate " +
			"fields (crust, boundary type, resistance) the Sample panel reads. " +
			"Generate a world to sample it.")

## `sample_cell()` omits a key whose backing data genuinely is not there, so
## every read here is `has()`-guarded and an absent key becomes an em dash --
## never `get(key, 0.0)`, which would report a real-looking zero for
## something that was never computed.
func _sample_field_text(key: String, cell: Dictionary) -> String:
	if cell.is_empty() or not cell.has(key):
		return "—"
	match key:
		"slope_deg":
			return "%.1f° · n %.2f" % [float(cell["slope_deg"]), float(cell.get("slope_n", 0.0))]
		"aspect_deg":
			return "%s %.0f°" % [String(cell.get("aspect", "?")), float(cell["aspect_deg"])]
		"plate":
			return "%d · %s" % [int(cell["plate"]), String(cell.get("plate_type", "?"))]
		"boundary_type":
			return _boundary_text(cell)
		"resistance":
			return "%.3f" % float(cell["resistance"])
		"lithology":
			return String(cell["lithology"])
		"temperature_c":
			return "%.1f °C" % float(cell["temperature_c"])
		"precipitation":
			return "%.2f" % float(cell["precipitation"])
		"drainage":
			return _drainage_text(cell)
		"biome":
			return String(cell["biome"]).capitalize()
		"soil":
			return "%.2f" % float(cell["soil"])
		"control":
			var owner := int(cell["control"])
			return "unclaimed" if owner <= 0 else "faction %d" % owner
	return "—"

## §6 asks for "boundary + distance" as one field. On a boundary cell the
## type is the reading and the distance is zero; off one the type raster is
## 0 ("none") everywhere, so the distance carries the information. An absent
## `boundary_dist_cells` means nothing was found inside the engine's own
## search cap -- reported as such rather than as a number.
func _boundary_text(cell: Dictionary) -> String:
	var kind := String(cell.get("boundary_type", "none"))
	if bool(cell.get("boundary", false)):
		return "%s · on it" % kind
	if not cell.has("boundary_dist_cells"):
		return "none within 96 cells"
	return "%s · %.1f cells" % [kind, float(cell["boundary_dist_cells"])]

func _drainage_text(cell: Dictionary) -> String:
	var flow := "%.1f" % float(cell["drainage"])
	if not cell.has("river_order"):
		return flow
	var order := int(cell["river_order"])
	return flow if order <= 0 else "%s · order %d" % [flow, order]

func _elevation_text(cell: Dictionary) -> String:
	if cell.is_empty() or not cell.has("elevation_m"):
		return "—"
	var suffix := ""
	match String(cell.get("water", "")):
		"ocean": suffix = " · ocean"
		"lake": suffix = " · lake"
	return "%.0f m%s" % [float(cell["elevation_m"]), suffix]

# -- Settlement -----------------------------------------------------------

func _build_settlement(body: Control) -> void:
	if _settlement_data == null:
		_build_sample(body)
		return
	var s: Dictionary = _settlement_data
	var sec := DccWidgets.section(body, "Settlement")
	_field(sec, "Name", String(s.get("name", "—")))
	_field(sec, "Class", String(s.get("kind", "—")).capitalize())
	_field(sec, "Population", str(int(s.get("population", 0))))
	_field(sec, "Faction", str(int(s.get("faction", 0))))
	_field(sec, "Coastal", "yes" if s.get("coastal", false) else "no")
	_field(sec, "Capital", "yes" if s.get("capital", false) else "no")
	_build_settlement_faith(sec, s)

	var why: Dictionary = bridge.explain_settlement(_settlement_index)
	var water := _term_value(why, "water_access")
	## Two different absences, and the row used to blame the narrow one for
	## both (2026-09-01). An EMPTY `why` means this world was opened rather
	## than generated -- `project_bridge.rs` stores no explanations at all --
	## which is a whole-project fact, not "no water_access entry for this
	## cell". The per-cell sentence is still right when `why` has terms and
	## none of them is `water_access`.
	_field(sec, "Water access", water if water != "" else "—",
		"" if water != "" else
			("Suitability diagnostics are computed at generate time and the project "
				+ "format does not store them (SAVEFILE_COMPAT.md §16.2), so an opened "
				+ "project has none. Regenerate this world to get them back."
				if why.is_empty() else
			"This settlement's suitability terms carry no water_access entry for this cell."),
		water != "")
	_field(sec, "Defensibility", "—",
		"explain_settlement()'s suitability terms have no defensibility axis -- " +
		"gentle_slope/terrain_form are the closest inputs but the engine doesn't " +
		"label either one defensibility.", false)
	_field(sec, "Routes", "—",
		"Roads and sea routes carry no settlement index (get_roads()/get_sea_routes() " +
		"are plain polylines) -- nothing associates a route with this settlement. " +
		"STRANDED_TOOLS.md row 11.", false)

	## RD-03: all three destinations now exist, so these are live rather than
	## disabled placeholders. Economy opens `world_data_window`'s own Economy
	## tab, scoped by name (`WorldDataWindow.open(tab)`, mirroring
	## `DataManagerWindow.open(group)`'s "scope to X" shape) -- the uncapped
	## settlement/province/trade tables §6 itself points Economy at. Politics
	## reuses this same dock's own Faction context (`show_faction()`, already
	## wired from `civilization_workspace.gd`'s Roster and Territory rows) for
	## this settlement's faction. Logistics arms the Journey Planner tool
	## takeover (`app.open_journey_planner()` -> `journey_planner_view.open()`
	## -> `app.arm_tool("journey")`, `DCC_SHELL_SPEC.md` §4.5.4's 2026-08-19
	## addition) rather than opening a dialog.
	var actions := DccWidgets.group(sec, "Actions")
	DccWidgets.action(actions, "Economy", func(): app.open_world_data("Economy"))
	DccWidgets.action(actions, "Politics", func(): show_faction(int(s.get("faction", 0))))
	DccWidgets.action(actions, "Logistics", func(): app.open_journey_planner())
	## `GUI_GAP_REGISTER.md` UM-02's launcher. The reference puts it in the
	## place-edit popup (`peCityOpen`), which this shell does not have yet
	## (ED-03) -- this dock's own Settlement context is the same information,
	## already carrying `_settlement_index`, so the action lands here rather
	## than waiting on a popup. It stays live regardless of whether the town
	## can be laid out: the window itself explains a refusal (a settlement in
	## open water gets no town) rather than a disabled button implying the
	## feature is missing.
	DccWidgets.action(actions, "City layout", func(): app.open_city_viewer(_settlement_index))

	var why_sec := DccWidgets.section(body, "Why here?")
	var rt := RichTextLabel.new()
	rt.bbcode_enabled = true
	rt.fit_content = true
	rt.scroll_active = false
	rt.custom_minimum_size.x = 220
	rt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	rt.add_theme_font_size_override("normal_font_size",
		DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL)
	rt.add_theme_color_override("default_color", DccTheme.c("text"))
	rt.text = _build_causal_chain_text(s, _settlement_index)
	why_sec.add_child(rt)

## -- Faith -------------------------------------------------------------------
##
## `RELIGION_DIFFUSION_SCOPE.md` milestone 1's settlement-inspector half. The
## CIVIL > Religion category is the world roll-up; this is the one settlement
## the dock is already showing.
##
## **The two keys are absent until a diffusion has run**, and that absence is
## read as absence. `lib.rs`'s `get_settlements` says why in its own doc:
## *"omitted, not defaulted to `none` and an empty dictionary, because those
## are exactly what a fully secular settlement in a world that has been
## simulated looks like."* So a missing `religion` is dashed with its reason,
## and a present `"none"` is printed as the real answer it is -- most of these
## people follow no faith -- rather than as a second kind of dash.
##
## **Read fresh from the bridge, not from `_settlement_data`.** That dictionary
## is the snapshot taken when the pin was clicked, and a diffusion run
## afterwards adds the two keys to the engine's answer without being able to
## add them to a copy taken before it. Every other row in this section is a
## placement fact that does not move, which is why they still read the
## snapshot and this one does not.
##
## **Matched by `tid`, not by position.** `_settlement_index` is an index into
## `get_settlements()`; deleting a settlement makes index N a different town,
## and printing its adherence under this town's name is precisely the "never
## show a faith a settlement does not hold" rule failing. `tid` is the engine's
## own stable id and is on every entry.
##
## Labels and shares come from `CivilizationWorkspace`'s own statics rather
## than a second copy of them. CIVIL > Religion is the other surface that
## prints these numbers, and two renderings of one faith -- "Sun Cult" against
## "Sun cult", a real congregation of two people as `0.0%` against `<0.1%` --
## costs more than the coupling does. A rename over there breaks this file at
## parse time, which is loud rather than silent.
func _build_settlement_faith(sec: Control, snapshot: Dictionary) -> void:
	if not bridge.has_belief_api():
		_field(sec, "Faith", "—",
			"This build's engine has no civ_belief_run() binding -- the native library is "
			+ "older than this shell, so nothing here can report a religion. That is a build "
			+ "state, not a world with no faiths in it: rebuild and re-export.", false)
		return
	var live := _live_settlement(snapshot)
	if live.is_empty():
		_field(sec, "Faith", "—",
			"The engine's settlement list no longer carries this town under the id it was "
			+ "selected with, so there is no entry to ask about its adherence. Re-select it "
			+ "on the map.", false)
		return
	if not live.has("religion"):
		_field(sec, "Faith", "—",
			"The belief layer does not cover this settlement. Either no diffusion has been "
			+ "run in this world -- the layer is built on demand and is not saved with the "
			+ "project -- or the last run was discarded because something it was seeded from "
			+ "changed. CIVIL > Religion tells the two apart, and runs it.", false)
		return

	var key := String(live["religion"])
	_field(sec, "Faith", CivilizationWorkspace._religion_label(key),
		"The plurality faith: the one with the most adherents here, which is not necessarily "
		+ "a majority -- the shares below are the whole answer. \"No religion\" is one of the "
		+ "rows and one of the possible pluralities; it means most of these people follow no "
		+ "faith, not that the model has nothing to say.")

	var pop := int(live.get("population", 0))
	if not live.has("adherents"):
		DccWidgets.note(sec, "— no head-counts for this settlement: the engine gave a "
			+ "plurality without the adherents dictionary it is derived from. Nothing here "
			+ "can be turned into a share.")
		return
	var adherents: Dictionary = live["adherents"]
	var rows := CivilizationWorkspace._religion_sorted(adherents)
	if rows.is_empty():
		if pop <= 0:
			DccWidgets.note(sec, "— no adherents to count: this settlement's population "
				+ "is 0, so every share is real and every head-count floors to nobody.")
		else:
			DccWidgets.note(sec, ("— no adherents listed for %s people. A faith with "
				+ "nobody in it is omitted rather than written as 0, so an empty list under a "
				+ "real population is the engine disagreeing with itself.")
				% _thousands(float(pop)))
		return

	## Open at four rows or fewer for the same reason the settlement list in
	## CIVIL is: a short list is the answer, and collapsing it hides the whole
	## point of showing shares rather than one plurality label.
	var grp := DccWidgets.group(sec, "Adherence", rows.size() <= 4)
	DccWidgets.note(grp, ("Shares of this settlement's own population, %s people -- the "
		+ "denominator is this town and not the world. The engine hands the rounding "
		+ "remainder to the largest fractions rather than rounding each share alone, so "
		+ "these read as head-counts. A faith with nobody in it is not listed at all, and "
		+ "`<0.1%%` is a real congregation too small to round rather than an absent row.")
		% _thousands(float(pop)))
	for r in rows:
		var rkey: String = r[1]
		var n: int = r[0]
		DccWidgets.note(grp, "%s %s — %s people (%s)" % [
			CivilizationWorkspace._religion_swatch_glyph(rkey),
			CivilizationWorkspace._religion_label(rkey),
			_thousands(float(n)), CivilizationWorkspace._religion_pct(n, pop)])

## This settlement as the engine describes it *now*, or `{}` when the entry at
## `_settlement_index` is no longer the same town.
##
## `{}` is the honest answer for both failures and they are not distinguished
## here on purpose: an index past the end of the list and an index that now
## holds a different `tid` are the same fact for a caller -- there is no entry
## for this settlement -- and the caller says so once.
##
## Falls back to comparing names only when `tid` is missing from either side,
## which is what an older cdylib looks like; a name is a weaker id than a tid
## and a stronger one than the bare index this replaces.
func _live_settlement(snapshot: Dictionary) -> Dictionary:
	if _settlement_index < 0 or bridge == null:
		return {}
	var places: Array = bridge.settlements()
	if _settlement_index >= places.size():
		return {}
	var live: Dictionary = places[_settlement_index]
	if snapshot.has("tid") and live.has("tid"):
		return live if int(live["tid"]) == int(snapshot["tid"]) else {}
	return live if String(live.get("name", "")) == String(snapshot.get("name", "")) else {}

func _term_value(why: Dictionary, key: String) -> String:
	if why.is_empty() or not why.has("terms"):
		return ""
	for t in why["terms"]:
		var d: Dictionary = t
		if String(d.get("key", "")) == key:
			return "%.2f" % float(d["value"])
	return ""

func _term_strength(value: float) -> String:
	if value >= 0.75:
		return "strong"
	if value >= 0.45:
		return "moderate"
	if value > 0.05:
		return "weak"
	return "negligible"

func _describe_term(t: Dictionary) -> String:
	var key := String(t["key"])
	var label_text: String = SUIT_TERM_LABELS.get(key, key.replace("_", " "))
	return "%s %s (%.2f)" % [_term_strength(float(t["value"])), label_text, float(t["value"])]

## Ported verbatim from `main.gd`'s `_build_causal_chain_text` -- same
## thresholds (0.005), same "top 3 positives / top 2 negatives" cap, same
## wording. Only the surrounding dock changed.
func _build_causal_chain_text(s: Dictionary, index: int) -> String:
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := [
		"[b]%s[/b] (%s)" % [s["name"], kind_label],
		"Population: %s" % s["population"],
		"Faction: %d" % s["faction"],
		"Coastal: %s" % ("yes" if s["coastal"] else "no"),
		"Capital: %s" % ("yes" if s["capital"] else "no"),
	]

	var why: Dictionary = bridge.explain_settlement(index)
	if not why.is_empty():
		lines.append("")
		lines.append("[b]WHY HERE?[/b]")
		if why.has("excluded"):
			lines.append("Cell excluded from suitability (%s)." % why["excluded"])
		else:
			var terms: Array = why["terms"]
			var positives: Array[String] = []
			var negatives: Array[String] = []
			for t: Dictionary in terms:
				var c := float(t["contribution"])
				if c > 0.005 and positives.size() < 3:
					positives.append(_describe_term(t))
				elif c < -0.005 and negatives.size() < 2:
					negatives.append(_describe_term(t))
			if positives.is_empty():
				lines.append("No single factor stands out -- placed on broadly average ground.")
			else:
				lines.append(" → ".join(positives))
			if not negatives.is_empty():
				lines.append("Despite: %s" % ", ".join(negatives))
			lines.append("Suitability %.2f" % float(why["score"]))

		lines.append("")
		var ord_i := int(why["river_order"])
		var river_txt := ("Strahler %d" % ord_i) if ord_i > 0 else "none"
		var coast_cells := float(why["coast_dist_cells"])
		lines.append("River: %s · flow %.0f" % [river_txt, float(why["flow"])])
		lines.append("Distance to water: %.1f cells" % coast_cells)
		lines.append("Elevation: %.3f (normalised)" % float(why["elevation"]))
		lines.append("Travel cost: %.2f" % float(why["travel_cost"]))
	else:
		## **Said, not silently dropped** (2026-09-01).
		##
		## `explain_settlement()` returns an empty dictionary for every
		## settlement of a world that was *opened* rather than generated:
		## `project_bridge.rs` rebuilds `CivData` with `explanations:
		## Vec::new()` and says why in as many words -- an explanation is a
		## diagnostic over suitability rasters the archive does not store
		## (`SAVEFILE_COMPAT.md` §16.2), and synthesising one from what is
		## stored would be inventing a reason rather than recalling it.
		##
		## Until now the whole block -- the causal chain AND the six terrain
		## readouts under it -- simply was not appended, so the panel came
		## back one section shorter with nothing said about it, which reads
		## as a dock that is broken on this save rather than a diagnostic
		## the format never carried.
		lines.append("")
		lines.append("[b]WHY HERE?[/b]")
		lines.append("Not available for an opened project. The suitability "
			+ "diagnostics behind this chain -- and the river, water-distance, "
			+ "elevation and travel-cost readings under it -- are computed at "
			+ "generate time, and the project format does not store them "
			+ "(SAVEFILE_COMPAT.md §16.2). They are omitted rather than "
			+ "reconstructed from what was saved. Regenerating this world from "
			+ "its parameters brings them back.")

	return "\n".join(lines)

# -- Route ------------------------------------------------------------------

func _build_route(body: Control) -> void:
	var sec := DccWidgets.section(body, "Route")
	var e := _route_entry
	## A hand-drawn way is committed nameless (`civ_commit_way` sets
	## `name: ""`), so the em-dash fallback has to catch the empty string,
	## not just a missing key.
	var nm := String(e.get("name", ""))
	_field(sec, "Name", nm if not nm.is_empty() else "—")
	_field(sec, "Type", String(e.get("way_type", "")).capitalize() if _route_kind == "road" else "Sea lane")
	## `manual` (`GUI_GAP_REGISTER.md` IN-02): the map draws hand-drawn and
	## generated ways identically, on purpose (the reference styles by
	## `way_type` alone), so this readout is the only place the distinction
	## is visible -- and it is worth showing, because only one of the two is
	## something the user authored.
	_field(sec, "Source", "Hand-drawn (Way tool)" if e.get("manual", false) else "Generated network")

	var pts: PackedVector2Array = e.get("points", PackedVector2Array())
	_field(sec, "Points", str(pts.size()))
	## `km` is the engine's own routed length (`Way::km`/`ManualWay::km`,
	## computed in `f64` over the real grid). Preferred over
	## `_route_length_text`, which re-measures the `f32` `PackedVector2Array`
	## this getter rounds to -- that fallback stays for any caller still
	## passing a dict from before `km` was emitted.
	var km := float(e.get("km", 0.0))
	_field(sec, "Length", ("%.1f km" % km) if km > 0.0 else _route_length_text(pts))

	var unreachable := ["Stages", "Vessels", "Cost trace", "Per-stage overrides", "Daily stages"]
	for f in unreachable:
		_field(sec, f, "—",
			"get_roads()/get_sea_routes() carry only {points, brks, way_type, name, km, manual} -- " +
			"the manual-route authoring context (ManualWay/RouteContext, tools.rs) that " +
			"would supply this has no read surface. STRANDED_TOOLS.md row 11.", false)

func _route_length_text(pts: PackedVector2Array) -> String:
	if pts.size() < 2:
		return "—"
	var cells := 0.0
	for i in range(1, pts.size()):
		cells += pts[i - 1].distance_to(pts[i])
	var gw := bridge.grid_size().x
	if gw > 0 and bridge.last_width_km > 0.0:
		return "%.0f km" % (cells * bridge.last_width_km / float(gw))
	return "%.0f cells" % cells

# -- River --------------------------------------------------------------

## **This context is reachable, as of `OUTSTANDING_WORK.md` §2.2.** The note
## that stood here until now said "there is no `get_rivers()`, so there is no
## river to name, to total a length for, or to select in the viewport", and
## every clause of it is now false: `WorldGen::get_rivers(min_order)` returns
## the entities and `WorldGen::river_at(gx, gy, radius, min_order)` picks one,
## both over `cartalith_hydrology::river_entities`. `_on_map_clicked_river` in
## this file is the trigger the context never had.
##
## Of the seven rows this context used to dash, **six now carry a real
## reading**; Name is the only genuine remainder, and its reason is narrower
## than the old blanket one -- it is a missing name generator, not a missing
## binding. Length is promoted to the accent readout (Route's own shape) and
## four rows are new: Order, Fall, At the mouth, and Channel (drawn).
func _build_river(body: Control) -> void:
	var sec := DccWidgets.section(body, "River")
	if _river.is_empty():
		DccWidgets.note(sec,
			"No river selected. Arm Inspect and click a river on the map -- the " +
			"pick tests every traced channel run, headwater trickles included, " +
			"inside a 44 px-wide target centred on the pointer. A river the map does not draw " +
			"cannot be picked: a world opened from a project archive carries no " +
			"channel topology at all (SAVEFILE_COMPAT.md stores none), so its " +
			"rivers are in the baked raster and nowhere else.")
		return

	## §6's one big accent readout per context. Length, matching Route's --
	## the two contexts describe the same kind of thing and the collapsed-dock
	## readout is this same number.
	_accent_readout(sec, "Length", DccUnits.format_adaptive(float(_river.get("km", 0.0))),
		"The traced run's own length: the sum of its cell-to-cell steps, in grid " +
		"cells, times map_width_km / gw. A river here is one drawable receiver " +
		"chain -- what drawRiverWays strokes as a single river -- so a main stem " +
		"is measured from the headwater it was traced from, not from every source " +
		"that eventually feeds it.")

	var order := int(_river.get("order", 0))
	_field(sec, "Name", "—",
		"Rivers are unnamed in this engine, and that is a missing generator " +
		"rather than a missing binding. cartalith-civ's naming::FeatureKind has " +
		"Continent, Province, Bay, MountainRange and Lake -- no river form -- so " +
		"there is no toponym to print and inventing one here would put a name on " +
		"the map that nothing else in the world knows about.", false)
	_field(sec, "Order", "Strahler %d" % order,
		"strahler_from_receivers, rescanned over every cell of this run and " +
		"reported as its maximum -- drawRiverWays' own maxO, which is also what " +
		"colours the reference's stroke. A tributary's last point is its trunk's " +
		"junction cell, so a short tributary can report its trunk's order.")
	_field(sec, "Source elevation", _m_text(float(_river.get("source_m", 0.0))),
		"Metres at the headwater cell this run was traced from, through the same " +
		"metersPerUnit anchoring the Sample panel's elevation uses. The row under " +
		"it is the fall from there to the mouth.")
	_field(sec, "Fall", _m_text(float(_river.get("drop_m", 0.0))),
		"Source elevation minus mouth elevation. Negative is possible and is not " +
		"a bug: the traced chain follows build_channels' aspect-projected " +
		"receiver, and the carve pass moves the field under it afterwards.")
	_field(sec, "Discharge", "%s" % _thousands(float(_river.get("discharge", 0.0))),
		"The largest flow accumulation on the run (WorldState::flow_discharge, " +
		"compute_flow with rainfall seeding). Deliberately the maximum, not the " +
		"value at the mouth: the polyline follows one receiver tree and the " +
		"accumulation was built on another, so discharge is not monotone " +
		"downstream. Measured on a 192x144 world, 194 of 773 runs peak above " +
		"their own mouth -- the mouth reading is the row below.")
	_field(sec, "At the mouth", "%s" % _thousands(float(_river.get("mouth_discharge", 0.0))),
		"Flow accumulation at the outlet cell specifically -- what leaves this " +
		"river. See the Discharge row for why the two differ.")
	_field(sec, "Catchment", "%s km²" % _thousands(float(_river.get("catchment_km2", 0.0))),
		"The Discharge reading as an area, at this world's cell size. It is " +
		"rainfall-WEIGHTED, not a plain cell count: compute_flow seeds each cell " +
		"with its rainfall rescaled so the mean seed is exactly 1.0, so a wetter- " +
		"than-average basin reads larger than its true area and a drier one " +
		"smaller. A true unweighted area needs a second whole-grid compute_flow " +
		"pass -- the measured hottest line in generate() -- which is not " +
		"something to run on a dock rebuild.")
	## **"Channel (drawn)", not "Channel width".** The number is twice
	## `channel_disc`'s half-width -- the width of the ink `stamp_river_intensity`
	## lays down -- and `river_width_scale_k` deliberately grows it as the map's
	## real extent shrinks, so a river stays visible on a zoomed-in sheet. On an
	## 800 km / 192-cell world that is ~1.9 cells, which converts to ~8 km: true
	## of the drawn channel, absurd of a river. Both units are printed, cells
	## first, and the row is named for what it measures.
	if _river.has("width_cells"):
		var wc := float(_river["width_cells"])
		var km := _cell_km()
		var span := ("%.2f cells" % wc) if km <= 0.0 else \
			("%.2f cells · %s" % [wc, DccUnits.format_adaptive(wc * km)])
		_field(sec, "Channel (drawn)", span,
			"How wide this river is DRAWN at its mouth, in grid cells: twice " +
			"channel_disc's half-width, the same law stamp_river_intensity inks the " +
			"map with. It is a cartographic symbol, not a hydraulic measurement -- " +
			"river_width_scale_k widens it as the map's real extent shrinks, on " +
			"purpose, so a river stays legible on a 50 km sheet. The converted " +
			"figure beside it is the ground distance that symbol covers, which is " +
			"why it can read in kilometres.")
	else:
		_field(sec, "Channel (drawn)", "—",
			"The width law (channel_disc) needs positive flow at the cell it is " +
			"asked about, and this run's mouth carries none.", false)
	_field(sec, "Tributaries", str(int(_river.get("tributaries", 0))),
		"Traced runs that end on a cell of this one. Exact, not estimated: " +
		"trace_river_polylines stops a run at the first already-visited cell and " +
		"pushes that shared cell as its last point, so a tributary's mouth IS a " +
		"cell of its trunk.")
	## The engine's own barge/raft gate, not a display convention:
	## `civ_navigable_river_discount` (reference `_civNavigableRiverDiscount`,
	## ~line 20951) discounts travel cost only at Strahler >= 3, and the routing
	## layer is the one consumer of it. The discount *curve* stays private to
	## `cartalith-civ`; only the threshold is stated here, and it is stated
	## rather than silently applied.
	_field(sec, "Navigation", "Navigable" if order >= 3 else "Not navigable",
		"cartalith-civ's civ_navigable_river_discount (the reference's " +
		"_civNavigableRiverDiscount) treats Strahler order 3 and above as barge- " +
		"or raft-navigable and discounts travel cost across it; below 3 there is " +
		"no discount. This row states that threshold -- the discount itself is " +
		"private to the routing cost and is not exposed.")

	var actions := DccWidgets.group(sec, "Actions")
	## **Two actions, not three.** The "Hydrology" action that stood here was
	## defined by its own tooltip as "would report this river's Strahler order,
	## discharge and channel width"; all three are rows above now, so a button
	## that opens them would open what is already open. Removed rather than
	## re-labelled with a new pretext.
	##
	## The remaining two keep their disabled state, and both tooltips are
	## rewritten because both had gone false in the same way the audit's
	## dangerous class describes -- a control disabled for a reason that no
	## longer holds.
	var why := {
		"Edit geometry":
			"Would move the river's course. There IS a polyline now (get_rivers()' " +
			"points), which is what this tooltip used to say there was not. What is " +
			"still missing is the other half: the course is derived from the receiver " +
			"tree on every call, and nothing writes an edited polyline back into the " +
			"flow field it came from, so an edit would be discarded by the next trace.",
		"Analyse catchment":
			"Would break the catchment down -- which sub-basins feed this river, and " +
			"where. The total is on the Catchment row above; what does not exist is " +
			"the decomposition, which needs labelled drainage basins. landmark.rs " +
			"records the same absence for its own confluence rule: \"no basin entity " +
			"exists\".",
	}
	for label_text in ["Edit geometry", "Analyse catchment"]:
		var b := DccWidgets.action(actions, label_text, func(): pass)
		b.disabled = true
		b.tooltip_text = String(why[label_text])

## Metres, at the precision a metre reading in this shell can honestly carry:
## whole metres, since every elevation here is one cell's `field` value through
## `peak_m`, and no reading in this port resolves finer than a cell.
func _m_text(m: float) -> String:
	return "%s m" % _thousands(m)

# -- Faction ------------------------------------------------------------

func _build_faction(body: Control) -> void:
	var sec := DccWidgets.section(body, "Faction")
	var mine: Array[Dictionary] = []
	for p in bridge.provinces():
		var d: Dictionary = p
		if int(d.get("faction", -1)) == _faction_id:
			mine.append(d)

	## RD-08: this used to list province names under "Roster" -- a list of
	## who claims the faction, not a reading of the faction itself. §6 calls
	## for a "roster entry", singular: `WorldGen::get_factions`
	## (`cartalith-godot/src/lib.rs:6559 fn get_factions`) carries the real
	## per-faction culture/government/ag-tech/colour/settlement_count, so that's
	## what fills this section now.
	##
	## (Citation history, because it is the point: this number has been wrong
	## three times. It read `3442`, then `6225`, then `6295` -- each re-derived
	## in good faith and each stale before it was committed, because other
	## agents were editing `lib.rs` in the same window. The third value was the
	## worst of them: `6295` had drifted into `civ_faction_territory_stats`'s
	## doc comment, whose `dict!` a few lines below carries a plausible
	## `"faction"` key -- so following the citation landed a reader in a
	## different function that *looks* like the right one, and this file calls
	## that function too, at `_build_faction`'s Territory row below. Corrected
	## the third time on 2026-08-31, in a pass with nothing else editing the
	## tree, and every number here now carries the symbol name beside it so the
	## next drift is a `grep` away instead of a silent lie.)
	var roster := _faction_roster(_faction_id)

	_field(sec, "Faction", str(_faction_id))
	## `Government` and `Ag. technology` ride the same `roster` dict `Culture`
	## does -- the `"government"` and `"ag_tech"` keys of `get_factions`'s own
	## `dict!` (`cartalith-godot/src/lib.rs:6578-6579`, the `"government"` and
	## `"ag_tech"` rows, two under `"culture"` at `:6365`) -- and were shown
	## nowhere in this dock, while
	## `faction_roster_window.gd`'s `_vocab_choice`/`_ag_tech_choice` rows
	## (`:432`, `:438`) both read *and* edit them. Named
	## with that window's own labels rather than a second vocabulary for the
	## same two fields. Found by the 2026-08-31 unwired audit.
	if roster.is_empty():
		_field(sec, "Culture", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
		_field(sec, "Government", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
		_field(sec, "Ag. technology", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
		## The same reason its three siblings carry, not an empty string. A
		## ghosted row whose tooltip says nothing is indistinguishable from a
		## control that is broken, and this one is neither -- the roster is
		## simply empty until a world exists. Found by the 2026-09-01
		## integration audit, which reported the empty argument twice.
		_field(sec, "Settlements", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
	else:
		_field(sec, "Culture", String(roster.get("culture", "?")).capitalize())
		## `capitalize()` is the same formatting `Culture` above uses, and it
		## reads both vocabularies correctly: the government keys are
		## snake_case and the ag-tech keys camelCase (`traditionalAgrarian`),
		## which it splits into words either way.
		_field(sec, "Government", String(roster.get("government", "?")).capitalize())
		_field(sec, "Ag. technology", String(roster.get("ag_tech", "?")).capitalize())
		_faction_colour_row(sec, roster)
		_field(sec, "Settlements", str(int(roster.get("settlement_count", 0))))

	## RD-06: `civ_faction_territory_stats(faction)` is real and live now --
	## same call `civilization_workspace.gd`'s `_tool_options_territory()`
	## already reads for the CIVIL ▸ Territory options row. Reads — only when
	## the faction has committed no territory (an empty dict, not a zeroed
	## one, so a genuine zero-cells faction doesn't read as "not read here").
	var stats := bridge.civ_faction_territory_stats(_faction_id)
	_field(sec, "Territory",
		("%d cells · %.0f km² · %d contested" % [
			int(stats.get("claimed_cells", 0)), float(stats.get("area_km2", 0.0)),
			int(stats.get("contested_cells", 0))]) if not stats.is_empty() else "—",
		"" if not stats.is_empty() else
			"civ_faction_territory_stats() returned nothing for this faction -- no committed territory yet.",
		not stats.is_empty())
	_field(sec, "Provinces", str(mine.size()))
	## §6's Faction context asks for "state religion", and it was dashed with a
	## reason that looked at the wrong source: *"get_provinces() doesn't carry
	## it and there is no get_faction_aggregates() binding."* Both halves are
	## true and neither is relevant -- **this row is built from `roster`, not
	## from `get_provinces()`**, and `get_factions()` has carried `"religion"`
	## since the roster window shipped -- the `"religion"` key of
	## `get_factions`'s `dict!` (`cartalith-godot/src/lib.rs:6578`, the
	## `"religion"` row; the citation read `6243`, then `6313`, and both had
	## drifted -- `6313` onto `civ_faction_territory_stats`'s own `"faction"`
	## row, which reads convincingly and is the wrong function. Third
	## correction, 2026-08-31; see the note on `get_factions` above).
	##
	## `roster` is fetched at the head of this same function, by the
	## `bridge.get_factions()` loop in `_build_faction` -- deliberately named
	## rather than numbered, since a self-citation into this file drifts every
	## time this file is touched. `Culture` two rows up already reads out of it. The binding was one
	## `.get()` away for as long as the row has said it was missing.
	##
	## Found by the 2026-08-31 unwired audit. Not stale WIRING -- a stale
	## REASON, which `audit_wiring.py` structurally cannot see: every `#[func]`
	## involved is called, and it is the tooltip that lies.
	if roster.is_empty():
		_field(sec, "State religion", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
	else:
		var rel := String(roster.get("religion", "")).strip_edges()
		## `"none"` is a real answer from `cartalith-civ`'s own vocabulary, not
		## an absence -- a faction with no state religion is a fact about the
		## world, so it prints rather than dashing.
		_field(sec, "State religion", rel.capitalize() if rel != "" else "—")
	_build_faction_relations(body)

## `GUI_GAP_REGISTER.md` **RL-01**. Every relation this faction is a party to,
## with the pair the reader actually clicked marked. `civ_faction_relations()`
## is symmetric and derived per call (§40) — it is the same read
## `civilization_workspace.gd`'s own Relationships list makes, filtered here to
## one faction rather than restated, so the two cannot disagree about a value.
##
## Reads from the *other* side's point of view deliberately: this panel is
## already headed by one faction, so each row names who it is a relation *with*.
func _build_faction_relations(body: Control) -> void:
	var pairs: Array = bridge.civ_faction_relations()
	var mine: Array[Dictionary] = []
	for p in pairs:
		var d: Dictionary = p
		if int(d.get("a", -1)) == _faction_id or int(d.get("b", -1)) == _faction_id:
			mine.append(d)
	var sec := DccWidgets.section(body, "Relations")
	if mine.is_empty():
		DccWidgets.note(sec,
			"No other faction to stand with or against. A relation needs two "
			+ "parties; add one in the faction roster.")
		return
	mine.sort_custom(func(x, y): return float(x.get("value", 0.0)) > float(y.get("value", 0.0)))
	for d in mine:
		var other := int(d.get("b", -1)) if int(d.get("a", -1)) == _faction_id else int(d.get("a", -1))
		var other_name := String(d.get("b_name", "?")) if int(d.get("a", -1)) == _faction_id \
			else String(d.get("a_name", "?"))
		var marked := other == _faction_pair
		_field(sec, ("▸ %s" % other_name) if marked else other_name,
			"%s (%+d)" % [String(d.get("stance", "neutral")),
				int(round(100.0 * float(d.get("value", 0.0))))],
			"Border %d cells (%d%% of the widest on this map) · culture %+d · "
			% [int(d.get("border_cells", 0)),
				int(round(100.0 * float(d.get("border_fraction", 0.0)))),
				int(round(30.0 * float(d.get("culture_term", 0.0))))]
			+ "faith %+d · trade %+d · rivalry %d%%."
			% [int(round(20.0 * float(d.get("religion_term", 0.0)))),
				int(round(25.0 * float(d.get("trade_term", 0.0)))),
				int(round(100.0 * float(d.get("rivalry_term", 0.0))))],
			true)

## `bridge.get_factions()`'s own row for one faction id, or an empty
## Dictionary if this world has none (no world yet, or a stale id). Factored
## out of `_build_faction` on 2026-09-02 so `_build_territory` -- which needs
## the same lookup for the same reason (a swatch and a name, not a province
## roster) -- reads it rather than growing a second copy of the loop.
func _faction_roster(faction_id: int) -> Dictionary:
	for f in bridge.get_factions():
		var d: Dictionary = f
		if int(d.get("id", -1)) == faction_id:
			return d
	return {}

## Colour swatch + hex -- the same 11×11 `ColorRect` legend `layers_popover
## .gd`'s `_refresh_legend` already uses for a faction/layer colour, just
## right-aligned to match `_field()`'s own value column instead of a
## left-aligned legend list. `_field()` itself can't carry a swatch (its
## value is one `Label`), so this is a second, small row builder local to
## the one context that needs one.
func _faction_colour_row(parent: Control, roster: Dictionary) -> void:
	var r := int(roster.get("color_r", 0))
	var g := int(roster.get("color_g", 0))
	var b := int(roster.get("color_b", 0))
	var tablet := DccTheme.is_tablet()
	var fs := DccTheme.role_px("fs_prose") if tablet else DccTheme.FS_SMALL
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	var l := DccTheme.label("Colour", "text_dim", fs)
	l.custom_minimum_size.x = _FIELD_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var trail := HBoxContainer.new()
	trail.add_theme_constant_override("separation", 6)
	trail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	trail.alignment = BoxContainer.ALIGNMENT_END
	var sw := ColorRect.new()
	sw.color = Color8(r, g, b)
	sw.custom_minimum_size = Vector2(11, 11)
	sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	trail.add_child(sw)
	trail.add_child(DccTheme.label("#%02X%02X%02X" % [r, g, b], "text", fs))
	row.add_child(trail)
	parent.add_child(row)

# -- Measure --------------------------------------------------------------
#
# §4.5.1's original right-dock spec was one line -- "Segment table (bearing,
# length), total, straight-line vs along-path difference" -- and that is
# Distance mode, still built exactly that way below. The other five come from
# `design/Cartalith Measurement Toolbar.dc.html`, whose own caption puts the
# "readouts in the right dock". Every number in all six comes straight off an
# engine dict (`measure_result` / `measure_area` / `measure_radius` /
# `measure_vertical` / `measure_section`); nothing is derived a second time
# here, which is why there is no arithmetic in this section at all beyond the
# one along-path-minus-straight-line difference §4.5.1 asks for by name.

## The one number this mode's reading comes down to, in **canonical** units:
## `{value: float, unit: String}`, or an empty dictionary when there is no
## reading yet. `unit` is one of `km`, `km2`, `m`, `deg`.
##
## The single place the six modes' primary keys are named. Both the collapsed
## dock's readout and the saved-measurements store read it, so a mode whose key
## changes cannot come back right in one and wrong in the other -- which is the
## drift a second copy of this `match` would have invited.
##
## **The key is omitted, not defaulted.** A mode that has not been given enough
## points returns `{}` and every caller branches on that, rather than being
## handed a `0.0` that reads exactly like a real measurement of nothing.
func _measure_primary() -> Dictionary:
	if _measure_result.is_empty():
		return {}
	var segs: Array = _measure_result.get("segments", [])
	match _measure_mode:
		"area":
			if _measure_result.has("projected_km2"):
				return {"value": float(_measure_result["projected_km2"]), "unit": "km2"}
		"radius":
			if _measure_result.has("radius_km"):
				return {"value": float(_measure_result["radius_km"]), "unit": "km"}
		"vertical":
			if _measure_result.has("delta_m"):
				return {"value": float(_measure_result["delta_m"]), "unit": "m"}
		"section":
			if _measure_result.has("length_km"):
				return {"value": float(_measure_result["length_km"]), "unit": "km"}
		"bearing":
			if not segs.is_empty():
				return {"value": float((segs[0] as Dictionary).get("bearing_deg", 0.0)), "unit": "deg"}
		_:
			## Distance. `segments` empty means a one-point chain, which
			## `_build_measure_distance` already treats as no reading.
			if not segs.is_empty() and _measure_result.has("total_km"):
				return {"value": float(_measure_result["total_km"]), "unit": "km"}
	return {}

## The collapsed dock's one number, per mode. Values from `_measure_primary()`;
## the affixes ("r ", " section") and the per-mode precision are presentation
## and stay here.
func _measure_readout() -> String:
	if _measure_result.is_empty():
		return "no reading"
	var p := _measure_primary()
	match _measure_mode:
		"area":
			return DccUnits.format_area(float(p.get("value", 0.0)))
		"radius":
			return "r %s" % DccUnits.format(float(p.get("value", 0.0)))
		"vertical":
			## Metres, deliberately: `DccUnits` converts a *linear map distance*
			## between km/mi/nmi, and an elevation delta is neither -- nautical
			## miles of altitude is not a reading anyone wants. Left as metres
			## until a vertical-unit setting exists, which is its own question.
			return "%+.0f m" % float(p.get("value", 0.0))
		"section":
			return "%s section" % DccUnits.format(float(p.get("value", 0.0)))
		"bearing":
			return ("%03d°" % int(round(float(p["value"])))) if not p.is_empty() else "no bearing"
		_:
			return DccUnits.format(float(p.get("value", 0.0)), 1)

func _build_measure(body: Control) -> void:
	match _measure_mode:
		"area": _build_measure_area(body)
		"radius": _build_measure_radius(body)
		"vertical": _build_measure_vertical(body)
		"section": _build_measure_section(body)
		"bearing": _build_measure_bearing(body)
		_: _build_measure_distance(body)

func _measure_empty(body: Control, title: String, prompt: String) -> VBoxContainer:
	var sec := DccWidgets.section(body, title)
	DccWidgets.note(sec, prompt)
	return sec

func _build_measure_distance(body: Control) -> void:
	var segments: Array = _measure_result.get("segments", [])
	if segments.is_empty():
		_measure_empty(body, "Measure · distance",
			"Click the map to drop points. ⌫ drops the last one, Esc clears the chain. Nothing here writes to the world -- a reading persists until you clear it.")
		return
	var sec := DccWidgets.section(body, "Measure · distance")
	_accent_readout(sec, "Total length", DccUnits.format(float(_measure_result.get("total_km", 0.0))),
		"Summed leg by leg, each leg through cartalith_spatial::measure -- the same km scale every route length in this port uses.")
	DccWidgets.note(sec, "%d segment%s · %d points" % [
		segments.size(), "" if segments.size() == 1 else "s",
		int(_measure_result.get("point_count", 0))])

	var segs := DccWidgets.group(sec, "Segments")
	for i in segments.size():
		var seg: Dictionary = segments[i]
		var b := float(seg.get("bearing_deg", 0.0))
		_field(segs, "%d" % (i + 1), "%s · %03d° · ↺ %03d°" % [
			DccUnits.format(float(seg.get("km", 0.0))), int(round(b)), int(round(fmod(b + 180.0, 360.0)))],
			"Bearing is this port's own convention: 0° = north, clockwise. ↺ is its reciprocal.")

	_build_measure_derived(body)
	_build_measure_actions(body)

func _build_measure_bearing(body: Control) -> void:
	var segments: Array = _measure_result.get("segments", [])
	if segments.is_empty():
		_measure_empty(body, "Measure · bearing",
			"Click two points. The first is the observer, the second the target.")
		return
	var seg: Dictionary = segments[0]
	var b := float(seg.get("bearing_deg", 0.0))
	var sec := DccWidgets.section(body, "Measure · bearing")
	_accent_readout(sec, "Bearing", "%03d°" % int(round(b)),
		"Grid y increases southward in every raster in this port, so 0° is north (-y), 90° east (+x), compass-clockwise.")
	_field(sec, "Reciprocal", "%03d°" % int(round(fmod(b + 180.0, 360.0))))
	_field(sec, "Distance", DccUnits.format(float(seg.get("km", 0.0)), 1))
	_build_measure_derived(body)
	_build_measure_actions(body)

## The canvas's DERIVED block. The three relief rows read `—` rather than a
## zero when `has_relief` is false -- a loaded save carries none of the
## substrate the height field needs, exactly as the Sample panel already says.
func _build_measure_derived(body: Control) -> void:
	var sec := DccWidgets.section(body, "Derived")
	var straight: float = float(_measure_result.get("straight_line_km", 0.0))
	var total: float = float(_measure_result.get("total_km", 0.0))
	var diff := total - straight
	_field(sec, "Straight line", DccUnits.format(straight, 1),
		("Along-path exceeds straight-line by %s." % DccUnits.format(diff, 1)) if diff > 0.01 else "")
	var ob := float(_measure_result.get("overall_bearing_deg", 0.0))
	_field(sec, "Overall bearing", "%03d° · ↺ %03d°" % [int(round(ob)), int(round(fmod(ob + 180.0, 360.0)))])
	var relief := bool(_measure_result.get("has_relief", false))
	_field(sec, "Sinuosity", ("%.2f" % float(_measure_result.get("sinuosity", 1.0))) if relief else "—",
		"Along-path over straight-line. 1.00 is a straight run.", relief)
	_field(sec, "Δ elevation", ("%+.0f m" % float(_measure_result.get("elevation_delta_m", 0.0))) if relief else "—",
		"First point to last, from the height field.", relief)
	_field(sec, "3D length", (DccUnits.format(float(_measure_result.get("total_km_3d", 0.0)), 1)) if relief else "—",
		"The chain followed over the ground rather than across the map.", relief)
	if not relief:
		DccWidgets.note(sec, "The three relief rows need a generated world: a loaded save carries no height substrate to read.")

## The canvas's foot: save · copy · CSV · plan journey, with the canvas's
## Saved measurements list above them.
##
## **All four are real now.** This block carried Copy alone until 2026-09-03,
## with a note arguing that "there is no saved-measurements store in this port
## and inventing one would be a persistence feature, not a measuring one". The
## owner ruled the other way and named the shape that makes the objection moot:
## the store is a caller-owned save **slot** (`annotations/measurements.json`)
## riding the `project_save_with_documents` channel the other five documents
## already use -- *"deliberately not a second persistence mechanism"*. So no
## new persistence was invented; this is the fifth caller of one that existed.
func _build_measure_actions(body: Control) -> void:
	_build_saved_measurements(body)
	var actions := DccWidgets.group(body, "Actions")
	var save_btn := DccWidgets.action(actions, "Save measurement", _on_measure_save)
	save_btn.disabled = _measure_primary().is_empty()
	DccWidgets.action(actions, "Copy reading", _on_measure_copy)
	var csv_btn := DccWidgets.action(actions, "Copy saved as CSV", _on_measure_csv)
	csv_btn.disabled = _saved_measurements.is_empty()
	DccWidgets.action(actions, "Plan a journey", func(): app.open_journey_planner())
	DccWidgets.note(actions,
		"Saved measurements travel in the project file and are dropped when the world is replaced: a measure " +
		"point is a grid cell, so a reading recalled over another world would draw a plausible line across " +
		"ground it was never taken on.")
	## Named rather than left to be discovered from a spreadsheet: the CSV is a
	## different promise from the readouts above it, which do follow the Units
	## preference (`DccUnits`, and every `_field` in this panel).
	DccWidgets.note(actions,
		"Copy reading puts THIS reading on the clipboard as tab-separated text. Copy saved as CSV puts the " +
		"list there in canonical km, km², metres and degrees -- never the Units preference, so the same " +
		"measurements export the same numbers on every machine.")

## The canvas's "Saved measurements" list. Newest first, the same order (and
## for the same reason) as the Sculpt stamp stack: the entry just added is the
## one most likely to be acted on.
##
## Absent rather than empty when nothing is saved. An always-drawn "nothing
## here yet" section would sit above every reading in the dock for the price of
## a sentence that the Save button below it already implies.
func _build_saved_measurements(body: Control) -> void:
	if _saved_measurements.is_empty():
		return
	var sec := DccWidgets.section(body, "Saved measurements")
	for i in range(_saved_measurements.size() - 1, -1, -1):
		_saved_measurement_row(sec, i)
	DccWidgets.action(sec, "Clear all", _on_measure_clear_all)

func _saved_measurement_row(parent: Control, i: int) -> void:
	var e: Dictionary = _saved_measurements[i]
	var pts: PackedVector2Array = e.get("points", PackedVector2Array())
	var mode := String(e.get("mode", ""))
	var tablet := DccTheme.is_tablet()
	var readout_fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_TINY
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 20
	row.tooltip_text = "%d point%s, the first at cell %.1f · %.1f. Recall re-arms Measure in %s mode with these exact points, so the reading is taken again rather than replayed from a number." % [
		pts.size(), "" if pts.size() == 1 else "s",
		pts[0].x if not pts.is_empty() else 0.0, pts[0].y if not pts.is_empty() else 0.0,
		_measure_mode_label(mode)]
	var l := DccTheme.mono_label("#%d %s · %s" % [i + 1, _measure_mode_label(mode), _saved_value_text(e)],
		"text", DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_SMALL)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.clip_text = true
	row.add_child(l)
	## §57 tier B, the same call the stamp-stack row makes: one row of many, so
	## `chip_min_h` rather than the discrete-action `btn_min_h`.
	for spec in [["recall", _on_measure_recall], ["drop", _on_measure_drop]]:
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.text = String(spec[0])
		b.add_theme_font_size_override("font_size", readout_fs)
		if tablet:
			b.custom_minimum_size.y = DccTheme.role_px("chip_min_h")
		b.pressed.connect((spec[1] as Callable).bind(i))
		row.add_child(b)
	parent.add_child(row)

## A saved entry's reading, converted for **display only** -- the store itself
## is canonical (see `_saved_measurements`), and `measurements_csv()`
## deliberately does not call this.
##
## A unit this build does not know is printed with the producer's own unit
## string rather than being assumed to be kilometres: §14.3's unknown-member
## rule, applied to a value instead of a member.
func _saved_value_text(e: Dictionary) -> String:
	if not e.has("value"):
		return "—"
	var v := float(e["value"])
	var unit := String(e.get("unit", ""))
	match unit:
		"km": return DccUnits.format(v, 1)
		"km2": return DccUnits.format_area(v)
		"m": return "%+.0f m" % v
		"deg": return "%03d°" % int(round(v))
		_: return "%.2f %s" % [v, unit]

func _measure_mode_label(id: String) -> String:
	for m in GlobalTools.MEASURE_MODES:
		if String((m as Dictionary)["id"]) == id:
			return String((m as Dictionary)["label"])
	return id

func _on_measure_save() -> void:
	var p := _measure_primary()
	var pts := GlobalTools.measure_points()
	if p.is_empty() or pts.is_empty():
		app.set_status("hint", "nothing to save yet -- place the points this mode needs first", "warn")
		return
	var e: Dictionary = {"mode": _measure_mode, "points": pts,
		"value": float(p["value"]), "unit": String(p["unit"])}
	_saved_measurements.append(e)
	app.set_status("hint",
		"saved #%d (%s) -- it goes into the project file on the next save" % [
			_saved_measurements.size(), _saved_value_text(e)], "accent")
	_rebuild()

func _on_measure_recall(i: int) -> void:
	if i < 0 or i >= _saved_measurements.size():
		return
	var e: Dictionary = _saved_measurements[i]
	GlobalTools.recall_measurement(app, String(e.get("mode", "distance")),
		e.get("points", PackedVector2Array()))

func _on_measure_drop(i: int) -> void:
	if i < 0 or i >= _saved_measurements.size():
		return
	_saved_measurements.remove_at(i)
	_rebuild()

func _on_measure_clear_all() -> void:
	clear_measurements()
	_rebuild()

func _on_measure_csv() -> void:
	if _saved_measurements.is_empty():
		return
	DisplayServer.clipboard_set(measurements_csv(_saved_measurements))
	app.set_status("hint", "%d saved measurement%s copied as CSV, in canonical units" % [
		_saved_measurements.size(), "" if _saved_measurements.size() == 1 else "s"], "text_ghost")

func _on_measure_copy() -> void:
	var lines: Array[String] = []
	for key in _measure_result.keys():
		var v = _measure_result[key]
		if v is Array or v is Dictionary:
			continue
		lines.append("%s\t%s" % [key, str(v)])
	for i in (_measure_result.get("segments", []) as Array).size():
		var seg: Dictionary = (_measure_result["segments"] as Array)[i]
		lines.append("segment %d\t%.4f km\t%.2f deg" % [i + 1, float(seg.get("km", 0.0)), float(seg.get("bearing_deg", 0.0))])
	DisplayServer.clipboard_set("\n".join(lines))
	app.set_status("hint", "measurement copied to the clipboard", "text_ghost")

func _build_measure_area(body: Control) -> void:
	if _measure_result.is_empty():
		_measure_empty(body, "Measure · area",
			"Click at least three points. The ring closes itself -- the edge back to the first point is always part of it.")
		return
	var r := _measure_result
	var sec := DccWidgets.section(body, "Measure · area")
	_accent_readout(sec, "Area · projected", "%s km²" % _thousands(float(r.get("projected_km2", 0.0))),
		"The exact shoelace figure over the ring's own vertices (polyArea, reference line 28290) times the map's km per cell. Never an estimate.")
	DccWidgets.note(sec, "true surface %s km² · %d vertices" % [
		_thousands(float(r.get("true_surface_km2", 0.0))), int(r.get("vertices", 0))])
	_field(sec, "Perimeter", "%.0f km" % float(r.get("perimeter_km", 0.0)))
	_field(sec, "Water subtracted", "−%s km²" % _thousands(float(r.get("water_km2", 0.0))),
		"Ocean and lake cells inside the ring." if bool(r.get("water_from_civ", false)) else
			"No civilisation layer for this world, so water here means \"below sea level\" -- it counts no lake standing above the waterline.")
	_field(sec, "Land area", "%s km²" % _thousands(float(r.get("land_km2", 0.0))))
	_field(sec, "Centroid", "%.0f E · %.0f N" % [float(r.get("centroid_x", 0.0)), float(r.get("centroid_y", 0.0))],
		"polyCentroid (reference line 28291) -- area-weighted, in grid cells.")
	_field(sec, "Bounding box", "%.0f × %.0f km" % [float(r.get("bbox_w_km", 0.0)), float(r.get("bbox_h_km", 0.0))])
	_field(sec, "Mean elevation", "%.0f m" % float(r.get("mean_elev_m", 0.0)))
	var stride := int(r.get("stride", 1))
	DccWidgets.note(sec, ("%d cells tested, every one inside the ring." % int(r.get("sampled_cells", 0))) if stride <= 1 else
		("%d cells tested at a stride of %d -- the projected area above is still exact; only the water split, the true surface and the mean elevation are sampled." % [int(r.get("sampled_cells", 0)), stride]))
	_build_measure_actions(body)

func _build_measure_radius(body: Control) -> void:
	if _measure_result.is_empty():
		_measure_empty(body, "Measure · radius", "Click the centre, then a point on the rim.")
		return
	var r := _measure_result
	var sec := DccWidgets.section(body, "Measure · radius")
	_accent_readout(sec, "Radius", "%.0f km" % float(r.get("radius_km", 0.0)), "")
	_field(sec, "Diameter", "%.0f km" % float(r.get("diameter_km", 0.0)))
	_field(sec, "Circumference", "%.0f km" % float(r.get("circumference_km", 0.0)))
	_field(sec, "Enclosed area", "%s km²" % _thousands(float(r.get("area_km2", 0.0))),
		"πr² on the map plane. It is not clipped to the coastline -- use Area for a ring that follows real ground.")
	_build_measure_actions(body)

func _build_measure_vertical(body: Control) -> void:
	if _measure_result.is_empty():
		_measure_empty(body, "Measure · Δ vertical", "Click two points to read the drop between them.")
		return
	var r := _measure_result
	var sec := DccWidgets.section(body, "Measure · Δ vertical")
	_accent_readout(sec, "Vertical difference", "%+.0f m" % float(r.get("delta_m", 0.0)), "")
	_field(sec, "P1 · P2 elevation", "%.0f m · %.0f m" % [float(r.get("p1_elev_m", 0.0)), float(r.get("p2_elev_m", 0.0))])
	_field(sec, "Horizontal distance", "%.1f km" % float(r.get("horizontal_km", 0.0)))
	_field(sec, "3D distance", "%.1f km" % float(r.get("distance_3d_km", 0.0)))
	_field(sec, "Grade · angle", "%.2f %% · %.2f°" % [float(r.get("grade_pct", 0.0)), float(r.get("angle_deg", 0.0))])
	DccWidgets.note(sec,
		"The canvas gates this pair on 3D relief and disables it in 2D. This port reads the same height field either way, " +
		"so it stays live in both -- there is nothing the 3D view knows about elevation that the 2D one does not.")
	_build_measure_actions(body)

## The canvas's state 2 dock: SAMPLED FIELDS, SECTION LINE, CROSSINGS. The
## profile itself is the strip's (`section_strip.gd`); this is everything
## about the line that is a number rather than a curve.
func _build_measure_section(body: Control) -> void:
	if _measure_result.is_empty():
		_measure_empty(body, "Measure · cross-section",
			"Click A then B. The profile draws in the strip under the map; scrubbing it marks the sampled cell on the map.")
		return
	var r := _measure_result
	var stats: Dictionary = r.get("stats", {})
	var samples: Array = r.get("samples", [])

	var sec := DccWidgets.section(body, "Section line")
	_accent_readout(sec, "Length", "%.0f km" % float(r.get("length_km", 0.0)), "")
	_field(sec, "Bearing", "%03d°" % int(round(float(r.get("bearing_deg", 0.0)))))
	_field(sec, "3D length", "%.0f km" % float(r.get("length_3d_km", 0.0)),
		"Following the sampled ground rather than the map plane.")
	_field(sec, "Samples · spacing", "%d · %.0f m" % [samples.size(), float(r.get("spacing_m", 0.0))])

	var st := DccWidgets.section(body, "Profile statistics")
	_field(st, "min · max", "%.0f m · %.0f m" % [float(stats.get("min_m", 0.0)), float(stats.get("max_m", 0.0))])
	_field(st, "mean", "%.0f m" % float(stats.get("mean_m", 0.0)))
	_field(st, "ascent", "%+.0f m" % float(stats.get("ascent_m", 0.0)))
	_field(st, "descent", "%.0f m" % float(stats.get("descent_m", 0.0)))
	_field(st, "net Δ", "%+.0f m" % float(stats.get("net_m", 0.0)))
	_field(st, "mean · max slope", "%.1f° · %.1f°" % [
		float(stats.get("mean_slope_deg", 0.0)), float(stats.get("max_slope_deg", 0.0))])
	_field(st, "above 2 000 m", "%.0f km" % float(stats.get("above_2000m_km", 0.0)))
	_field(st, "river crossings", str(int(stats.get("river_crossings", 0))))
	_field(st, "ridge crossings", str(int(stats.get("ridge_crossings", 0))),
		"A local maximum standing at least 100 m above the lower of the two valleys flanking it. That prominence floor is this port's own -- nothing in the reference defines a ridge crossing.")
	_field(st, "shore crossings", str(int(stats.get("shore_crossings", 0))))

	var cr := DccWidgets.section(body, "Crossings")
	var crossings: Array = r.get("crossings", [])
	if crossings.is_empty():
		DccWidgets.note(cr, "The line crosses no river, ridge or shoreline.")
	else:
		for c in crossings:
			var cd: Dictionary = c
			_field(cr, "%.0f km" % float(cd.get("km", 0.0)), String(cd.get("label", "")),
				"%.0f m at this crossing." % float(cd.get("elev_m", 0.0)))
		## **Corrected with §2.2's binding.** This read "no river entity crosses
		## the GDExtension boundary (see this dock's own River context), so
		## there is no toponym to print" -- and the first half stopped being
		## true the day `get_rivers()`/`river_at()` landed. The conclusion is
		## unchanged and the reason is now the real one: `naming::FeatureKind`
		## has no river form, so nothing names a river in the first place.
		DccWidgets.note(cr,
			"Rivers are described by Strahler order, not by name. The entity does cross the boundary now " +
			"(get_rivers(), and this dock's own River context), but nothing in the engine NAMES a river: " +
			"naming::FeatureKind covers continents, provinces, bays, mountain ranges and lakes, and no river form. " +
			"So there is still no toponym to print here.")
	_build_measure_actions(body)

# ======================================= Saved measurements, on disk (F10) ====
#
# `annotations/measurements.json` -- registered in `cartalith-io`'s
# `DOCUMENT_SLOTS` on 2026-09-03 and **caller-owned**, so the shell writes it
# and the engine carries it without modelling it, exactly the way
# `entities/journeys.json` already works (`SAVEFILE_COMPAT.md` §6.5, §11.4).
#
# The owner's ruling was that a measurement store is a save SLOT and
# "deliberately not a second persistence mechanism", and that is what this is:
# `app.gd::_project_documents()` merges the text below into the same dictionary
# `project_save_with_documents` already takes for the other five documents, and
# `project_open` hands it back byte for byte. Nothing new was opened, written
# or scheduled -- this file is the sixth caller of one channel.
#
# The three functions that do the work are `static` so a probe can exercise the
# document and the CSV with no world, no bridge and no dock: everything below
# that is not static is one line of plumbing over them.

## This dock's half of the project file, as JSON **text**, or `""` when there
## is nothing to write.
##
## Empty string rather than an empty document, the same contract
## `project_bridge.rs::paint_document_json()` states for its own slot: an
## absent slot and an empty one read the same to a parser and differ to anyone
## diffing two saves.
func measurements_document() -> String:
	if _saved_measurements.is_empty():
		return ""
	var g: Vector2i = bridge.grid_size() if bridge != null else Vector2i.ZERO
	return measurements_document_text(_saved_measurements, g.x, g.y)

## `entries` in the `_saved_measurements` shape -> the slot's text.
##
## `gw`/`gh` are **this world's** grid, and they are the whole of the
## world-anchoring answer: a measure point is a fractional grid cell, so the
## grid the points were clicked on is what a reader needs to know whether they
## still mean anything. Same two members, same names and same reason as
## `drafts/paint.json`'s (`PaintDoc`), so a second implementation meets one
## rule rather than two.
##
## `Vector2` becomes a two-element array because JSON has no vector -- the same
## conversion `journey_planner_view.gd::journeys_document()` makes for `trim`.
static func measurements_document_text(entries: Array, gw: int, gh: int) -> String:
	var out: Array = []
	for raw in entries:
		var e: Dictionary = raw
		var d: Dictionary = {"mode": String(e.get("mode", "distance"))}
		var arr: Array = []
		for p in (e.get("points", PackedVector2Array()) as PackedVector2Array):
			arr.append([p.x, p.y])
		d["points"] = arr
		## Omitted **together**, never written as a zero: a mode that produced
		## no single number has no value, and `0.0 km` is a measurement.
		if e.has("value") and _is_num(e["value"]):
			d["value"] = float(e["value"])
			d["unit"] = String(e.get("unit", ""))
		out.append(d)
	return JSON.stringify({"gw": gw, "gh": gh, "measurements": out})

## The inverse: `{ok: bool, reason: String, entries: Array}`.
##
## **A grid mismatch refuses the whole document.** Three answers were open --
## refuse, carry the entries with a staleness mark, or clear -- and this is
## refuse-then-clear, which is the answer this port has already given twice for
## the same question. `WorldGen::absorb` clears the vault's links and snapshots
## on every path that replaces a world; `project_open` refuses a
## `drafts/paint.json` whose `gw`/`gh` are not this world's, because a layer
## decoded against another grid is a scrambled picture rather than a smaller
## one. A staleness mark is the worst of the three here specifically: the number
## on a measurement stays readable and plausible while the points under it have
## come to name different ground.
##
## The document is left in the archive either way. Refusing to *show* a reading
## is not a reason to delete it from the file, and `app.gd` says so on screen.
static func measurements_from_document(text: String, gw: int, gh: int) -> Dictionary:
	var parsed = JSON.parse_string(text)
	if not (parsed is Dictionary):
		return {"ok": false, "entries": [],
			"reason": "annotations/measurements.json is not an object"}
	var doc: Dictionary = parsed
	var dgw := int(doc.get("gw", 0))
	var dgh := int(doc.get("gh", 0))
	if dgw != gw or dgh != gh:
		return {"ok": false, "entries": [],
			"reason": "these measurements were taken on a %dx%d grid and this world is %dx%d; a measure point is a grid cell and does not carry over" % [dgw, dgh, gw, gh]}
	var arr = doc.get("measurements", [])
	if not (arr is Array):
		return {"ok": false, "entries": [],
			"reason": "annotations/measurements.json carries no measurements array"}
	var entries: Array = []
	for raw in (arr as Array):
		if not (raw is Dictionary):
			continue
		var e: Dictionary = raw
		var praw = e.get("points", [])
		if not (praw is Array):
			continue
		var pts := PackedVector2Array()
		for p in (praw as Array):
			## Type-guarded, not just size-guarded. `float(<null>)` is not a
			## conversion in GDScript, it is a runtime error ("Invalid call.
			## Nonexistent float constructor") that aborts this whole function
			## -- so one malformed coordinate silently discarded every healthy
			## measurement beside it, and the caller's `ok == false` branch then
			## cleared the in-memory list too. Measured before the fix: a
			## document with one bad entry and one good one recovered zero.
			if p is Array and (p as Array).size() == 2 \
					and _is_num((p as Array)[0]) and _is_num((p as Array)[1]):
				pts.append(Vector2(float(p[0]), float(p[1])))
		## A measurement with no points is not a measurement -- it is the one
		## thing the reading cannot be recomputed or recalled from.
		if pts.is_empty():
			continue
		var out_e: Dictionary = {"mode": String(e.get("mode", "distance")), "points": pts}
		## `has()` is not enough: **this build writes `"value":null` itself**
		## when a mode produced a NaN (Godot warns about it at the writer), and
		## `float(null)` is the same aborting runtime error as above. A null
		## value means the same thing an absent key means -- no reading -- so
		## it takes the same path: omit the pair and let the list dash the row,
		## rather than lose the measurement's points along with it.
		if e.has("value"):
			out_e["value"] = float(e["value"])
			out_e["unit"] = String(e.get("unit", ""))
		entries.append(out_e)
	return {"ok": true, "entries": entries, "reason": ""}

## Is `v` a number this build can convert without raising?
##
## `float(<null>)` and `float(<Dictionary>)` are runtime errors in GDScript,
## not conversions, and a raise inside a restore loop takes the whole document
## with it. Kept as one predicate so the point reader and the value reader
## cannot drift into two different ideas of "numeric".
static func _is_num(v: Variant) -> bool:
	return typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT

## The saved list as CSV text.
##
## **Canonical units, always.** `unit` names them per row and is
## `km`/`km2`/`m`/`deg` whatever `DccSettings.units_mode()` is set to. A CSV
## whose numbers moved with a display preference would not be an export -- it
## would be a screenshot of one session's settings, with nothing in the file
## saying which. `DccUnits`' own header draws the same line for the same
## reason, quoting the reference: *"units: display-only. Canonical storage
## stays km"*.
##
## One row per measurement over a fixed six-column header, rather than a column
## per mode's own fields. The six modes measure different things -- a bearing
## has no length, an area has no elevation drop -- so a wide table would carry
## an empty cell for every mode a row is not; `value`+`unit` is the shape a
## heterogeneous readings table actually has.
##
## An absent reading writes an **empty cell**, which is CSV's own way of saying
## "no value" and is not `0`.
##
## `points_cells` is the coordinate list in grid cells, semicolon-separated and
## quoted: it is the one part of a measurement a spreadsheet cannot recompute
## from the other columns.
static func measurements_csv(entries: Array) -> String:
	var lines: Array[String] = ["index,mode,point_count,value,unit,points_cells"]
	for i in entries.size():
		var e: Dictionary = entries[i]
		var pts: PackedVector2Array = e.get("points", PackedVector2Array())
		var coords: Array[String] = []
		for p in pts:
			coords.append("%.4f %.4f" % [p.x, p.y])
		lines.append("%d,%s,%d,%s,%s,\"%s\"" % [
			i + 1, String(e.get("mode", "")), pts.size(),
			("%.4f" % float(e["value"])) if e.has("value") else "",
			String(e.get("unit", "")), ";".join(coords)])
	return "\n".join(lines)

## `app.gd::_restore_project_documents()` calls this once per project open with
## whatever the archive's slot held -- `""` when it carried none, which is a
## genuine "this project has no measurements" and clears the list, for the same
## reason `restore_journeys_document()` clears its own: keeping them is how
## project A's readings follow the reader into project B and get written into
## B's archive on the next save.
##
## Returns the sentence the person is owed, or `""` when there is nothing to
## say. No identity guard of the kind the journey planner needs: `app.gd` calls
## this only from `_load_project()`, the one place a new set of documents
## actually arrives.
func restore_measurements_document(text: String) -> String:
	var note := ""
	if text.strip_edges() == "":
		clear_measurements()
	else:
		var g: Vector2i = bridge.grid_size() if bridge != null else Vector2i.ZERO
		var r := measurements_from_document(text, g.x, g.y)
		if bool(r.get("ok", false)):
			_saved_measurements = r.get("entries", [])
		else:
			## The outgoing project's readings go either way: they were grid
			## cells of a world that is no longer on screen.
			clear_measurements()
			note = String(r.get("reason", ""))
	if _context == CTX_MEASURE:
		_rebuild()
	return note

## Empties the store because the world its points were grid cells of is gone.
##
## Public: `setup()` connects it to `generation_finished` and to the world-less
## half of `world_loaded`, `restore_measurements_document()` calls it when a
## newly opened project carries no measurements of its own, and the list's own
## Clear all button calls it. Deliberately does not rebuild -- every caller
## either rebuilds already or is about to.
func clear_measurements() -> void:
	_saved_measurements = []

# -- Region select --------------------------------------------------------

## §4.5.1's own right-dock spec: "Extent in both units, cell count, tile
## estimate per LOD, and Send to Data > Export." `region_get()`'s
## `tile_estimates` is `infra_tools_bridge::REGION_LOD_GRIDS`' own three-tier
## ladder -- see that constant's doc comment for why this port picked one
## with no reference precedent to match.
func _build_region(body: Control) -> void:
	var sec := DccWidgets.section(body, "Region select")
	if _region_result.is_empty():
		DccWidgets.note(sec, "Drag a marquee on the map to select a region.")
		return
	## `05-right-dock-and-bars.md` §1.6 left `regionRows` `UNSPECIFIED:` because
	## the delivered prototype was truncated; the 2026-08-31 re-export supplies it
	## (`Cartalith DCC Environment.dc.html`, the `const regionRows` line in
	## `valsCore()`), and its **first** row is the origin -- `X · Y` in cells.
	## This panel had every other row and never said *where* the marquee was, so
	## two different rects of the same size read identically. `region_get()`
	## (`cartalith-godot/src/lib.rs`, `fn region_get`) has carried `x`/`y` all
	## along; nothing had to move on the engine side.
	##
	## Read through `has()` rather than `get(k, 0)`: cell `0 · 0` is the map's own
	## north-west corner and a perfectly legal marquee origin, so a defaulted zero
	## would be indistinguishable from a real one.
	if _region_result.has("x") and _region_result.has("y"):
		_field(sec, "Origin",
			"%d · %d cells" % [int(_region_result["x"]), int(_region_result["y"])])
	else:
		_field(sec, "Origin", "—",
			"region_get() answered without x/y, so the marquee's corner is not "
			+ "readable. The extent below is still exact.", false)
	_field(sec, "Extent",
		"%d × %d cells" % [int(_region_result.get("w", 0)), int(_region_result.get("h", 0))])
	_field(sec, "Extent (%s)" % DccUnits.suffix(),
		"%s × %s" % [DccUnits.format(float(_region_result.get("w_km", 0.0))),
			DccUnits.format(float(_region_result.get("h_km", 0.0)))])
	## §1.6's `CELLS` row is `toLocaleString('en-US')` -- grouped. A marquee over
	## a working-resolution map runs to seven digits, and `str()` printed them
	## unbroken.
	_field(sec, "Cells", _thousands(float(int(_region_result.get("cell_count", 0)))))
	sec.add_child(DccTheme.rule())
	var estimates: Array = _region_result.get("tile_estimates", [])
	for e in estimates:
		var d: Dictionary = e
		_field(sec, String(d.get("lod", "")).capitalize(),
			"%d tiles (%d×%d)" % [int(d.get("tiles", 0)), int(d.get("tile_w", 0)), int(d.get("tile_h", 0))])
	## RD-09, live since 2026-08-20. Still deliberately **not**
	## `app.open_world_data()`: that window is the settlement/province/economy
	## tables (`world_data_window.gd`), not §9's export route. It now opens the
	## Data manager straight onto Export ▸ Maps, which is the panel that was
	## missing -- `region_export_tiles()` was bound and tested
	## (`LOD_TILING_INTEGRATION_SCOPE.md`'s M2, "Z4 is done") but callerless
	## until that pane was built.
	var actions := DccWidgets.group(sec, "Actions")
	DccWidgets.action(actions, "Send to Data ▸ Export", func():
		app.data_manager_window.open_tile_export())
	## §1.6's footnote, verbatim and lower-case as the prototype writes it. It is
	## the one line that says why the button above is not a second, competing
	## export extent -- the marquee and Data ▸ Export ▸ Maps' bounds are one rect
	## seen twice, which is exactly the "two pickers over one concept" shape this
	## shell has had to undo three times elsewhere. Worth drawing rather than
	## leaving to a tooltip: this dock is also the tablet's, where there is no
	## hover.
	DccWidgets.note(sec, "the marquee and the export route are two views of one rect")


# -- Wildlife ecoregion (the reference's own #wildInfo popup) ---------------

## `showWildInfo` (reference HTML 8259-8269), field for field: the biome
## heading, the species/area line, the region summary sentence, the
## NPP/ruggedness/water triple, the lat + coastal + rugged meta line, and
## then the fauna list -- one heading per guild with its biomass share, and
## one row per species with the reference's own `~4.5M` population wording
## (formatted engine-side by `wild_fmt_pop`, so this file does not carry a
## second copy of that formatter).
# -- History ledger (`GUI_GAP_REGISTER.md` ED-02) ----------------------------

## One glyph per `undo_ledger()` row kind. Shape carries the state and the
## row's own text repeats it -- nothing here is distinguished by colour alone.
const HISTORY_GLYPH := {"height": "▲", "recorded": "·", "floor": "◼"}

## The ledger, drawn in the two tiers `DCC_SHELL_SPEC.md` §7.1 proposal 2
## names -- and which are this engine's own draft/commit seam rather than an
## invented one.
##
## **Open draft** first, because it is what has not happened yet: an
## uncommitted Sculpt stack is reversible in place, by its own tool, and is
## deliberately not a row below. **Committed** after it, newest first, one row
## per commit whether or not the commit can be walked back.
##
## Three glyphs, and the text says the same thing the glyph does -- a row is
## never distinguished by colour alone:
##
## | | |
## |---|---|
## | `▲` | a height snapshot is held; "revert to here" is real |
## | `·` | recorded only, and the row carries the specific reason |
## | `◼` | a generate or a load: history starts here |
##
## Reading `bridge.undo_ledger()` fresh each rebuild is deliberate: the
## reversible flag is a property of the live undo stack, which evicts on its
## own byte budget, so a cached row would go stale silently.
func _build_history(body: Control) -> void:
	var rows: Array = bridge.undo_ledger()
	var stats := bridge.undo_stats()

	## The draft tier. `sculpt_list_stamps()` is the only draft this shell has
	## that survives across a dock rebuild; Paint's and Territory's live in
	## their own tool bodies with their own Discard.
	var stamps: Array = bridge.sculpt_list_stamps() if bridge.has_world else []
	if not stamps.is_empty():
		var d := DccWidgets.section(body, "Open draft")
		DccWidgets.note(d, "◐  Sculpt · %d stamp%s, uncommitted" % [
			stamps.size(), "" if stamps.size() == 1 else "s"])
		DccWidgets.note(d,
			"A draft's steps are its own, reversible in place from the Sculpt panel, and "
			+ "not entered below -- nothing has happened to the world yet.")

	var sec := DccWidgets.section(body, "Committed")
	if rows.is_empty():
		DccWidgets.note(sec,
			"Nothing committed this session. A generate, a load, a Sculpt or Paint "
			+ "commit, a carve or a territory commit all enter here.")
	else:
		## Newest first, which is how every history panel reads and the
		## opposite of the engine's own oldest-first order.
		for i in range(rows.size() - 1, -1, -1):
			_history_row(sec, rows[i])

	var cost := DccWidgets.section(body, "Cost")
	var bytes := int(stats.get("bytes", 0))
	var budget := int(stats.get("budget_bytes", 1))
	DccWidgets.note(cost, "Reversible: %s of %s · %d of %d steps" % [
		String.humanize_size(bytes), String.humanize_size(budget),
		int(stats.get("depth", 0)), int(stats.get("max_steps", 5))])
	DccWidgets.note(cost,
		"A recorded-only row costs nothing -- it is a label and a timestamp. Only a "
		+ "height snapshot occupies the budget, which is Preferences ▸ Memory ▸ "
		+ "Undo history.")

func _history_row(parent: Control, entry: Variant) -> void:
	var d: Dictionary = entry
	var kind := String(d.get("kind", "recorded"))
	var reversible := bool(d.get("reversible", false))
	var glyph := String(HISTORY_GLYPH.get(kind, "·"))
	var seq := int(d.get("seq", 0))
	var steps := int(d.get("steps", 0))
	var label := "%s  %s" % [glyph, String(d.get("label", "?"))]
	if reversible:
		var b := DccWidgets.action(parent, label, func(): _revert_history(seq, steps))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = ("%s · %s. Reverts the height field to the state before this "
			+ "operation, discarding the %d step%s after it as well -- history here is "
			+ "linear, so there is no branch to come back to.") % [
				String(d.get("subsystem", "")), String(d.get("detail", "")),
				steps - 1, "" if steps == 2 else "s"]
	else:
		var note := DccWidgets.note(parent, label)
		note.tooltip_text = "%s · %s" % [String(d.get("subsystem", "")), String(d.get("detail", ""))]
	var sub := String(d.get("detail", ""))
	if not reversible and String(d.get("reason", "")) != "":
		sub = "%s — %s" % [sub, String(d.get("reason", ""))] if sub != "" else String(d.get("reason", ""))
	if sub != "":
		DccWidgets.note(parent, "      %s" % sub)

## Linear revert, confirmed when it discards more than the row itself --
## `DCC_SHELL_SPEC.md` §7.1's own choice of Photoshop's linear history over
## the non-linear kind, and the one place in this panel that destroys work.
func _revert_history(seq: int, steps: int) -> void:
	if steps > 1:
		_confirm_revert(seq, steps)
		return
	_do_revert(seq)

func _confirm_revert(seq: int, steps: int) -> void:
	var dlg := ConfirmationDialog.new()
	dlg.dialog_text = ("Revert to this state?

%d committed operation%s after it will be "
		+ "discarded. History here is linear -- there is no branch to come back to.") % [
			steps - 1, "" if steps == 2 else "s"]
	dlg.ok_button_text = "Revert"
	dlg.confirmed.connect(func():
		_do_revert(seq)
		dlg.queue_free())
	dlg.canceled.connect(func(): dlg.queue_free())
	app.add_child(dlg)
	dlg.popup_centered()

func _do_revert(seq: int) -> void:
	var done := bridge.undo_revert_to(seq)
	if done <= 0:
		app.set_status("hint",
			"that step is no longer available — its snapshot was dropped to stay inside the undo budget",
			"text_ghost")
		_rebuild()
		return
	## The same repaint `app.undo_last()` does, and for the same reason: write
	## `map_view.texture` directly rather than calling `ViewportHost.refresh()`,
	## which would also reset the camera. Reverting should leave you looking at
	## exactly where you were looking.
	if app.viewport != null:
		app.viewport.map_view.texture = bridge.color_texture()
		app.viewport.set_preview_texture(null)
	var stats: Dictionary = bridge.undo_stats()
	app.set_status("pass", "reverted %d step%s" % [done, "" if done == 1 else "s"], "text_dim")
	app.set_status("hint", "%d undo step%s left · flow, rivers and climate are not re-run" % [
		int(stats.get("depth", 0)), "" if int(stats.get("depth", 0)) == 1 else "s"], "text_ghost")
	_rebuild()


func _build_wildlife(body: Control) -> void:
	if _wildlife_region.is_empty():
		_build_sample(body)
		return
	var rec := _wildlife_region
	## The biome name is a row, not the section title: a section header is
	## `DccTheme.header()`'s uppercase Plex Mono tracked 2 px wide and does not
	## trim, so "TROPICAL SEASONAL FOREST ECOREGION" was a ~270 px minimum this
	## panel handed the dock for a value that varies by ecoregion -- the same
	## width-follows-text fault `_field()` documents, one level up.
	var sec := DccWidgets.section(body, "Ecoregion")
	_field(sec, "Biome", String(rec.get("biome_name", "Unknown")), "", true, false, 60)
	_accent_readout(sec, "Species", "%d" % int(rec.get("richness", 0)),
		"Species richness: species-area x energy (NPP) x ruggedness x latitude, " +
		"cut to the biome's Earth-analogue roster.")
	_field(sec, "Area", "%s km²" % _thousands(float(rec.get("area_km2", 0.0))),
		"Region area, from its cell count and the map's own km-per-cell.")
	DccWidgets.note(sec, String(rec.get("summary", "")))
	sec.add_child(DccTheme.rule())
	_field(sec, "NPP", "%d g/m²/yr" % int(round(float(rec.get("npp", 0.0)))),
		"Net primary productivity, Miami model (Lieth 1975) -- the energy the whole food web is built on.")
	_field(sec, "Ruggedness", "%.3f" % float(rec.get("tri", 0.0)),
		"Terrain Ruggedness Index (Riley 1999), averaged over the region.")
	_field(sec, "Water", "%.2f" % float(rec.get("water", 0.0)),
		"Mean water access across the region: 1 at a river or coast, falling away inland.")
	var meta := "lat %d°" % int(round(float(rec.get("lat_abs", 0.0))))
	if bool(rec.get("coastal", false)):
		meta += " · coastal"
	if bool(rec.get("rugged", false)):
		meta += " · rugged"
	DccWidgets.note(sec, meta)

	var fauna := DccWidgets.group(sec, "Fauna (population estimate)")
	var guilds: Array = rec.get("guilds", [])
	if guilds.is_empty():
		DccWidgets.note(fauna, "No fauna assigned: this biome has no roster entry that clears its terrain gates.")
		return
	for g in guilds:
		var d: Dictionary = g
		_field(fauna, String(d.get("label", "")), "%d%%" % int(float(d.get("biomass_rel", 0.0)) * 100.0),
			"Share of the region's total animal biomass.")
		for sp in (d.get("species", []) as Array):
			var s: Dictionary = sp
			_field(fauna, "    " + String(s.get("name", "")), "~" + String(s.get("population_text", "")),
				"%s kg body mass. Population from the region's energy budget (Lindeman 10%% cascade) over Kleiber metabolic demand." % String.num(float(s.get("mass_kg", 0.0)), 2))

## `Number.toLocaleString()` (reference line 8265's own km² formatting).
func _thousands(v: float) -> String:
	var s := "%d" % int(round(v))
	var neg := s.begins_with("-")
	if neg:
		s = s.substr(1)
	var out := ""
	for i in range(s.length()):
		if i > 0 and (s.length() - i) % 3 == 0:
			out += ","
		out += s[i]
	return ("-" + out) if neg else out

# -- Journey (delegate to journey_planner_view.gd) --------------------------

## **Appended** below the selection while the Journey tool is armed in CIVIL
## (`rdMode4()` rule 8 -- see `TOOL_JOURNEY`), never instead of it.
##
## The null guard is a bare `return`, not the `_build_sample()` fallback it
## carried as a context: `_dispatch()` has already drawn Sample by the time
## this runs when nothing is selected, so falling back here would draw the
## cursor readout **twice**. `_tool_section()` will not answer `TOOL_JOURNEY`
## without a delegate anyway; this is the belt to that brace.
func _build_journey(body: Control) -> void:
	if _journey_view == null:
		return
	_journey_view.build_results(body)

# -- Sculpt stamp stack -----------------------------------------------------
#
# §6's own table lists this context twice under two names: "Stamp stack
# (Sculpt)" (stamps, hide/show/move/delete, selected-stamp parameters, undo/
# redo, commit/discard, finalize-lock) and "Brush / Stamp" (the eight brush/
# noise globals, "stamp stack, commit / discard"). Built as ONE context, not
# two: both read the exact same live state (`bridge.sculpt_list_stamps()`,
# `sculpt_get_globals()`), and the brush/noise globals already have their own
# live editors in `world_workspace.gd`'s left-dock Sculpt panel -- duplicating
# eight sliders here under a second name would be two views of one state
# fighting to be the one a caller edits, not two contexts. This context shows
# what the left-dock panel doesn't: the stack itself, newest-first, with the
# per-stamp actions and the selected stamp's own frozen parameters.

func _build_sculpt(body: Control) -> void:
	var sec := DccWidgets.section(body, "Stamp stack")
	if not bridge.has_world:
		DccWidgets.note(sec, "Generate a world first.")
		return
	if bridge.sculpt_get_globals().is_empty():
		DccWidgets.note(sec, "No sculpt editor for this world -- a loaded save has no draft session, only a freshly generated world does.")
		return

	var stamps: Array = bridge.sculpt_list_stamps()   ## already newest-first
	var selected := bridge.sculpt_get_selected_stamp()
	if stamps.is_empty():
		DccWidgets.note(sec, "No stamps yet -- arm a Sculpt feature (World ▸ Terrain ▸ Sculpt) and draw a stroke on the map.")
	else:
		for s in stamps:
			var d: Dictionary = s
			_sculpt_stamp_row(sec, d, selected)

	if selected >= 0:
		_build_selected_stamp(body, selected, stamps)

	var hist := DccWidgets.group(body, "History")
	var undo_btn := DccWidgets.action(hist, "%s Undo" % DccIcons.SYMBOLS["undo"], _on_sculpt_stack_undo)
	undo_btn.disabled = not bridge.sculpt_can_undo()
	var redo_btn := DccWidgets.action(hist, "%s Redo" % DccIcons.SYMBOLS["redo"], _on_sculpt_stack_redo)
	redo_btn.disabled = not bridge.sculpt_can_redo()
	DccWidgets.note(hist, "Draft-scoped only (add/delete/hide/reorder) -- never touches the real heightfield.")

	## `GUI_GAP_REGISTER.md` RD-13: WW-01 (`948e15a`) gave `FinalizeLock` a real
	## engine, and `sculpt_commit` is one of its five guarded call sites --
	## `finalize_check("height_edit")` returns the same refusal sentence the
	## engine itself would print, so the button and the note agree with what a
	## press would actually do instead of failing silently against a locked
	## world.
	var lock_msg := String(bridge.finalize_check("height_edit"))
	var actions := DccWidgets.group(body, "Commit")
	var commit_btn := DccWidgets.action(actions, "%s Commit to map" % DccIcons.SYMBOLS["tick"], _on_sculpt_stack_commit, true)
	commit_btn.disabled = stamps.is_empty() or not lock_msg.is_empty()
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_sculpt_stack_discard)
	discard_btn.disabled = stamps.is_empty()
	DccWidgets.note(body,
		"Commit bakes the whole stamp stack into the heightfield and marks the tiles it " +
		"touched stale -- it does not re-run erosion, hydrology or climate " +
		"(DCC_SHELL_SPEC.md header correction #1).")
	if not lock_msg.is_empty():
		DccWidgets.note(body, lock_msg)

func _sculpt_stamp_row(parent: Control, d: Dictionary, selected: int) -> void:
	var idx := int(d.get("index", -1))
	var hidden := bool(d.get("hidden", false))
	var label_text := String(d.get("label", "?"))
	var pts := int(d.get("point_count", 0))
	var tablet := DccTheme.is_tablet()
	var readout_fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_TINY
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 20
	var mark := DccTheme.mono_label(DccIcons.SYMBOLS["off"] if hidden else DccIcons.SYMBOLS["on"],
		"text_ghost" if hidden else "text_dim", readout_fs)
	row.add_child(mark)
	var text := "#%d %s (%d pt%s)" % [idx, label_text, pts, "" if pts == 1 else "s"]
	var l := DccTheme.mono_label(text, "accent" if idx == selected else "text",
		DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_SMALL)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.clip_text = true
	row.add_child(l)
	## §57 tier B: one row of many, one selected -- the same "one of a set is
	## lit" shape as a mode chip, so it takes `chip_min_h`/`fs_readout` rather
	## than the discrete-action `btn_min_h`.
	var select_btn := Button.new()
	select_btn.flat = true
	select_btn.focus_mode = Control.FOCUS_NONE
	select_btn.text = "selected" if idx == selected else "select"
	select_btn.disabled = idx == selected
	select_btn.add_theme_font_size_override("font_size", readout_fs)
	if tablet:
		select_btn.custom_minimum_size.y = DccTheme.role_px("chip_min_h")
	select_btn.pressed.connect(_on_stamp_select.bind(idx))
	row.add_child(select_btn)
	parent.add_child(row)

## §6: "selected-stamp parameters (length, width, asymmetry, ridge noise,
## blend)" -- that exact axis list is main.gd's old sculpt-overlay prose, not
## this engine's own controls (`SculptStamp`'s frozen `params` are whichever
## feature it was painted with, per `sculpt_bridge::feature_param_pairs`), so
## this reads the real per-stamp dictionary instead of reproducing a list
## that doesn't match what got captured.
func _build_selected_stamp(parent: Control, selected: int, stamps: Array) -> void:
	var data: Dictionary = {}
	for s in stamps:
		var d: Dictionary = s
		if int(d.get("index", -1)) == selected:
			data = d
			break
	if data.is_empty():
		return
	var sec := DccWidgets.section(parent, "Selected stamp")
	_field(sec, "Feature", String(data.get("label", "?")))
	_field(sec, "Points", str(int(data.get("point_count", 0))))
	var params: Dictionary = data.get("params", {})
	for key in params.keys():
		var v = params[key]
		_field(sec, String(key).capitalize(), ("%.3f" % float(v)) if v is float else str(v))

	var hidden := bool(data.get("hidden", false))
	var actions := DccWidgets.group(sec, "Actions")
	DccWidgets.action(actions, "Deselect", _on_stamp_deselect)
	DccWidgets.action(actions, "Show" if hidden else "Hide", _on_stamp_toggle_hidden.bind(selected, hidden))
	DccWidgets.action(actions, "Move up", _on_stamp_move_up.bind(selected))
	DccWidgets.action(actions, "Move down", _on_stamp_move_down.bind(selected))
	DccWidgets.action(actions, "Delete", _on_stamp_delete.bind(selected))

func _on_sculpt_stack_undo() -> void:
	bridge.sculpt_undo()
	show_sculpt_stack()

func _on_sculpt_stack_redo() -> void:
	bridge.sculpt_redo()
	show_sculpt_stack()

## `map_view` is written directly (not `ViewportHost.refresh()`, which would
## also reset the camera to fit) -- the same reasoning
## `world_workspace.gd`'s own `_on_sculpt_commit` uses.
func _on_sculpt_stack_commit() -> void:
	bridge.sculpt_commit("sculpt")
	if app != null and app.viewport != null:
		app.viewport.map_view.texture = bridge.color_texture()
		app.viewport.set_preview_texture(null)
	show_sculpt_stack()

func _on_sculpt_stack_discard() -> void:
	bridge.sculpt_discard()
	if app != null and app.viewport != null:
		app.viewport.set_preview_texture(null)
	show_sculpt_stack()

func _on_stamp_select(index: int) -> void:
	bridge.sculpt_select_stamp(index)
	show_sculpt_stack()

func _on_stamp_deselect() -> void:
	bridge.sculpt_select_stamp(-1)
	show_sculpt_stack()

func _on_stamp_toggle_hidden(index: int, hidden: bool) -> void:
	bridge.sculpt_set_stamp_hidden(index, not hidden)
	show_sculpt_stack()

func _on_stamp_move_up(index: int) -> void:
	bridge.sculpt_move_stamp_up(index)
	show_sculpt_stack()

func _on_stamp_move_down(index: int) -> void:
	bridge.sculpt_move_stamp_down(index)
	show_sculpt_stack()

func _on_stamp_delete(index: int) -> void:
	bridge.sculpt_delete_stamp(index)
	show_sculpt_stack()

# -- Paint (`rdPaint`, §1.8) -------------------------------------------------
#
# **Appended** while the "paint" tool is armed (this port's own id for the
# design's `biome` tool -- see the `TOOL_PAINT` const's own doc). Reads
# `world_workspace.gd`'s live paint-editor state fresh every rebuild, the
# same "no private draft" shape `TOOL_STAMPS` already uses -- Commit/Discard
# here and Commit/Discard in the left-dock Biome paint panel are the same two
# engine calls, so neither can disagree with the other about what is pending.
# (They CAN both be on screen showing the same number, which is fine -- see
# `TOOL_STAMPS`'s own header comment on why that is not the two-drafts bug
# class `WW-13` was.)
#
# **The legend does not match §1.8's own table**, and that is a divergence in
# the engine, not a guess: that table names six biome / four soil / four
# vegetation entries (`bpLegend()`, the truncated prototype's own fixture).
# This port's real `PaintTarget` (`paint_bridge.rs`) is `Biome`/`Terrain`/
# `Splat` -- Biome alone has 13 paintable values, and neither Terrain nor
# Splat is named "soil" or "vegetation". Read from `bridge.get_paint_layers()`
# / `get_paint_palette()` instead of the design's fixed table, matching
# `world_workspace.gd::_build_paint`'s own precedent for the identical
# mismatch.

func _build_paint(body: Control) -> void:
	if not bridge.has_world:
		DccWidgets.note(DccWidgets.section(body, "Paint"), "Generate a world first.")
		return
	var layers := bridge.get_paint_layers()
	if layers.is_empty():
		DccWidgets.note(DccWidgets.section(body, "Paint"),
			"No paint editor for this world -- a loaded save has no draft session, same ceiling as Sculpt.")
		return
	var layer: String = _paint_ctx_layer if layers.has(_paint_ctx_layer) else String(layers[0])

	## §1.3's own right-dock title for this tool, `PAINT · BIOME`/`· TERRAIN`/
	## `· SPLAT`, which is drawn here now that the dock header keeps naming the
	## selection instead (`CTX_TITLES`' own note). It read `Painted · %s` while
	## the header carried `Paint · %s`; one name for one thing, and the
	## "painted" wording survives on the count row directly below.
	var sec := DccWidgets.section(body, "Paint · %s" % layer.capitalize())
	var counts: Dictionary = bridge.paint_painted_counts()
	var total := int(counts.get("total", 0))
	_accent_readout(sec, "Painted cells", _thousands(float(total)),
		"paint_painted_counts() for the active layer -- the composite of every committed dab and whatever is " +
		"still in the draft, the same figure the left-dock Biome paint panel's own Legend group totals.")
	var pending := bridge.paint_draft_count()
	DccWidgets.note(sec, "Nothing pending across any layer." if pending == 0 else
		("%d dab%s pending across every layer, not just %s -- Commit/Discard below act on all three at once " +
			"(paint_bridge.rs's own PaintEditor::commit_all).") % [pending, "" if pending == 1 else "s", layer])

	var palette := bridge.get_paint_palette(layer)
	var by_index: Dictionary = counts.get("counts", {})
	var swatches := _paint_swatch_colors(layer)
	var legend := DccWidgets.group(sec, "Legend · painted counts")
	if palette.is_empty():
		DccWidgets.note(legend, "No palette for this layer.")
	else:
		for i in palette.size():
			var pd: Dictionary = palette[i]
			var idx := int(pd.get("index", i + 1))
			_paint_legend_row(legend, String(pd.get("label", "?")), int(by_index.get(idx, 0)),
				swatches[i] if i < swatches.size() else {}, idx)

	var commit_group := DccWidgets.group(sec, "Commit")
	var commit_btn := DccWidgets.action(commit_group, "%s Commit" % DccIcons.SYMBOLS["tick"], _on_paint_commit_from_dock, true)
	commit_btn.disabled = pending == 0
	var discard_btn := DccWidgets.action(commit_group, "Discard draft", _on_paint_discard_from_dock)
	discard_btn.disabled = pending == 0
	if pending == 0:
		var why := "Nothing pending. Paint on the map to enable this." if total == 0 \
			else "Nothing pending -- the %s cells above are already committed." % _thousands(float(total))
		commit_btn.tooltip_text = why
		discard_btn.tooltip_text = why
	DccWidgets.note(sec,
		"Commit writes every layer's pending dabs into their own override arrays and refreshes the map -- which " +
		"stages that marks stale depends on the layer (paint_bridge.rs's own reason string per target); the " +
		"status bar names them once you press it.")

## Real per-value colour for the Biome and Terrain palettes, pulled from
## `bridge.debug_layers()`'s own "Biomes"/"Terrain" legend rows (`"bclass"`/
## `"cterrain"`, `sample_bridge::legend()`) rather than invented -- there is
## no `get_paint_palette` colour, so this is the one real source. Verified
## directly against `paint_bridge.rs`: `PaintTarget::palette()` slices
## `CART_BIOMES[..13]` / the whole of `CART_TERRAINS`, and `legend()` builds
## `"bclass"`/`"cterrain"` by enumerating `CART_BIOME_COLS`/`CART_TERRAIN_COLS`
## over those same arrays in the same order -- position `i` in one is
## position `i` in the other. Splat has no debug-layer legend (it forces a
## ground texture, not a colour -- `world_workspace.gd`'s own "Splat has no
## map colour of its own" note), so this returns empty for it and the legend
## row below falls back to no swatch.
func _paint_swatch_colors(layer: String) -> Array:
	var debug_id: String = {"biome": "bclass", "terrain": "cterrain"}.get(layer, "")
	if debug_id == "":
		return []
	for g in bridge.debug_layers():
		for it in (g as Dictionary).get("items", []):
			var item: Dictionary = it
			if String(item.get("id", "")) == debug_id:
				return item.get("legend", [])
	return []

## One legend row: swatch (when known) + label + painted count, and -- only
## when the caller wired one into `show_paint()` -- a click that arms this
## value for painting (§1.8: "Click arms that value"). This file cannot
## build that click itself: `paint_set_brush` takes radius/hardness/
## softness/land_only alongside the value, and only `world_workspace.gd`
## knows the brush's current values for those -- see `show_paint()`'s own
## doc. Read-only when no callback was given, which is still the swatch,
## label and count §1.8 asks for.
func _paint_legend_row(parent: Control, label_text: String, count: int, swatch: Dictionary, value_index: int) -> void:
	var tablet := DccTheme.is_tablet()
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	if not swatch.is_empty():
		var sw := ColorRect.new()
		sw.color = Color8(int(swatch.get("r", 0)), int(swatch.get("g", 0)), int(swatch.get("b", 0)))
		sw.custom_minimum_size = Vector2(10, 10)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		row.add_child(sw)
	var fs := DccTheme.role_px("fs_prose") if tablet else DccTheme.FS_SMALL
	var l := DccTheme.label(label_text, "text", fs)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.clip_text = true
	row.add_child(l)
	row.add_child(DccTheme.mono_label(str(count), "text_dim", fs))
	if _paint_on_pick.is_valid():
		var pick := Button.new()
		pick.text = "arm"
		pick.tooltip_text = "Arms %s for painting -- the same brush value the left-dock Biome paint panel's own Value picker sets." % label_text
		pick.focus_mode = Control.FOCUS_NONE
		pick.custom_minimum_size = Vector2(34, _chip_btn_h(tablet))
		pick.pressed.connect(func(): _paint_on_pick.call(value_index))
		row.add_child(pick)
	parent.add_child(row)

## Both call sites are a "select this one of several" row button --
## `_sculpt_stamp_row`'s own "select"/"selected" precedent (`right_dock.gd`
## §57 tier B: "the same 'one of a set is lit' shape as a mode chip, so it
## takes `chip_min_h`... rather than the discrete-action `btn_min_h`"), not a
## standalone Commit/Delete-class action -- `chip_min_h` on tablet accordingly,
## not `btn_min_h`.
func _chip_btn_h(tablet: bool) -> int:
	return int(DccTheme.role_px("chip_min_h")) if tablet else 20

func _on_paint_commit_from_dock() -> void:
	var summary: Dictionary = bridge.paint_commit()
	if app != null and app.viewport != null:
		app.viewport.map_view.texture = bridge.color_texture()
		app.viewport.set_preview_texture(null)
	var stale: PackedStringArray = summary.get("stale_stages", PackedStringArray())
	if app != null:
		app.set_status("hint", ("painted -- stale: %s" % ", ".join(stale)) if stale.size() > 0 else "painted", "text_ghost")
	show_paint(_paint_ctx_layer, _paint_on_pick)

func _on_paint_discard_from_dock() -> void:
	bridge.paint_discard()
	if app != null and app.viewport != null:
		## `true` for the same reason `world_workspace.gd`'s own
		## `_on_paint_discard` passes it: the committed layer that survives a
		## discard is the base the next dab's bounded window blits onto.
		app.viewport.set_preview_texture(bridge.build_paint_preview_texture(), true)
	show_paint(_paint_ctx_layer, _paint_on_pick)

# -- Ramp · stops (`rdStops`, §1.9) ------------------------------------------
#
# **Appended** whenever the CARTO domain is active with nothing more specific
# armed (`rdMode4()` rule 6) -- see `show_stops()`'s own doc for the two triggers
# this needs. Reads and writes the exact engine ramp `render_workspace.gd`'s
# own "Colour relief" panel already edits (`bridge.color_ramp()`/
# `set_color_ramp()`) -- there is no second ramp to disagree with, only a
# second view of the one the map actually draws from.
#
# **Two real divergences from §1.9's own table, adjudicated per §7's own
# instruction for a "contradiction in the delivered design" rather than
# guessed:**
#
# - **A stop's position is not an elevation in metres.** `get_color_ramp()`'s
#   own doc: "relative land elevation... 0 = the shoreline, 1 = the world's
#   highest point" -- not §1.9's literal `eMin=-410, eMax=4210`, which was
#   the truncated prototype's own placeholder range for a fixed-size demo
#   world. Metres shown here are that fraction of THIS world's own relief
#   (`peak_m`), so the same stop reads a different metre figure on a
#   different world -- correct, because the ramp is defined relative to each
#   world's own peak, not to a constant.
# - **Interpolation is the ramp's, not a stop's.** §1.9 draws `interp` inside
#   the "selected stop" editor; `render.rs`'s own `ElevationRamp::set_mode`
#   takes no stop index, and `render_workspace.gd`'s own note already says
#   so ("This belongs to the ramp rather than to a stop"). Drawn once, above
#   the stop list, not per stop.

func _build_stops(body: Control) -> void:
	if not bridge.ramp_api:
		DccWidgets.note(DccWidgets.section(body, "Ramp · stops"),
			"No colour-ramp editor: this build's engine has no set_color_ramp() binding.")
		return
	var sec := DccWidgets.section(body, "Ramp · stops")
	var modes: Array = bridge.ramp_modes()
	if not modes.is_empty():
		DccWidgets.choice(sec, "Blend", modes, maxi(0, modes.find(bridge.ramp_mode())),
			_on_stops_mode_changed.bind(modes),
			"How the colour crosses from one stop to the next -- the whole ramp's own setting, not this stop's (see this section's own header note).")

	var stops: Array = bridge.color_ramp()   ## [[position, Color], ...], already sorted by get_color_ramp()
	_build_ramp_bar(sec, stops, bridge.ramp_mode())

	if stops.is_empty():
		DccWidgets.note(sec, "This ramp has no stops -- Add one below.")
	else:
		var list := DccWidgets.group(sec, "Stops")
		for i in stops.size():
			_stops_row(list, i, stops)

	var edit_group := DccWidgets.group(sec, "Edit")
	DccWidgets.action(edit_group, "+ add", _on_stops_add)
	var reverse_btn := DccWidgets.action(edit_group, "reverse", _on_stops_reverse)
	reverse_btn.disabled = stops.size() < 2

	if _stops_selected >= 0 and _stops_selected < stops.size():
		_build_selected_stop(body, _stops_selected, stops)

	var actions := DccWidgets.group(body, "Actions")
	var compare_btn := DccWidgets.action(actions, "Compare", func(): pass)
	compare_btn.disabled = true
	compare_btn.tooltip_text = "Would hold the previous ramp for an A/B toggle -- §1.9's own \"(mock)\" caption in the design file. No such store exists here."
	DccWidgets.note(actions,
		"Every edit above is already live -- render_workspace.gd's own ramp editor, this dock and the map all " +
		"read the one engine ramp, so there is no separate Apply step to press (§1.9's own Apply is a mock in " +
		"the delivered prototype).")

## A small live gradient preview, mirroring `render_workspace.gd`'s own
## `_update_ramp_bar` -- duplicated rather than shared because that one is
## private to a panel built once and this one is rebuilt wholesale on every
## `_rebuild()`, the tradeoff every other section in this file already makes.
func _build_ramp_bar(parent: Control, stops: Array, mode: String) -> void:
	var offsets := PackedFloat32Array()
	var colors := PackedColorArray()
	var sorted := stops.duplicate()
	sorted.sort_custom(func(a, b): return float(a[0]) < float(b[0]))
	for s in sorted:
		offsets.append(clampf(float(s[0]), 0.0, 1.0))
		colors.append(s[1] as Color)
	if offsets.is_empty():
		return
	if offsets.size() == 1:
		offsets.append(1.0)
		colors.append(colors[0])
	var grad := Gradient.new()
	match mode:
		"Step": grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CONSTANT
		"Ease": grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CUBIC
		_: grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_LINEAR
	grad.offsets = offsets
	grad.colors = colors
	var tex := GradientTexture1D.new()
	tex.gradient = grad
	tex.width = 256
	var bar := TextureRect.new()
	bar.texture = tex
	bar.stretch_mode = TextureRect.STRETCH_SCALE
	bar.custom_minimum_size.y = 16
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(bar)

func _stops_row(parent: Control, idx: int, stops: Array) -> void:
	var tablet := DccTheme.is_tablet()
	var pair: Array = stops[idx]
	var col := pair[1] as Color
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	var sw := ColorRect.new()
	sw.color = Color(col.r, col.g, col.b, 1.0)
	sw.custom_minimum_size = Vector2(14, 14)
	sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(sw)
	var fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_SMALL
	var l := DccTheme.mono_label("#%s" % col.to_html(false).to_upper(), "accent" if idx == _stops_selected else "text_dim", fs)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(l)
	row.add_child(DccTheme.mono_label("%d m" % int(round(_ramp_metres(float(pair[0])))), "text_dim", fs))
	var sel := Button.new()
	sel.text = "selected" if idx == _stops_selected else "select"
	sel.disabled = idx == _stops_selected
	sel.focus_mode = Control.FOCUS_NONE
	sel.custom_minimum_size = Vector2(58, _chip_btn_h(tablet))
	sel.pressed.connect(func(): _stops_selected = idx; _rebuild())
	row.add_child(sel)
	parent.add_child(row)

func _build_selected_stop(body: Control, idx: int, stops: Array) -> void:
	var pair: Array = stops[idx]
	var col := pair[1] as Color
	var sec := DccWidgets.section(body, "Selected stop")

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	var picker := ColorPickerButton.new()
	## Opaque on purpose, the same reasoning `render_workspace.gd`'s own
	## swatch carries: this control owns hue, the stop's own alpha rides
	## along untouched by it -- **and it replaces §1.9's own hue slider
	## rather than reproducing it**: §7 item 26 flags that slider's thumb as
	## permanently pinned at 50%, never tracking the real hue, a documented
	## bug in the delivered prototype. A real colour picker is strictly
	## better and is what `render_workspace.gd`'s own editor already uses.
	picker.color = Color(col.r, col.g, col.b, 1.0)
	picker.custom_minimum_size = Vector2(34, 22)
	picker.edit_alpha = false
	picker.color_changed.connect(_on_stop_color_changed.bind(idx))
	head.add_child(picker)
	head.add_child(DccTheme.mono_label("#%s" % col.to_html(false).to_upper(), "text", DccTheme.FS_SMALL))
	head.add_child(DccTheme.spacer())
	head.add_child(DccTheme.mono_label("%d m" % int(round(_ramp_metres(float(pair[0])))), "accent",
		DccTheme.role_px("fs_readout") if DccTheme.is_tablet() else DccTheme.FS_SMALL))
	sec.add_child(head)

	## Hand-built rather than `DccWidgets.slider()`: the readout is a
	## DERIVED metre figure (position × peak_m), not the raw 0..1 value that
	## helper's own unit-suffix formatter would print -- the same reason
	## `render_workspace.gd`'s own position slider bypasses it too.
	var pos_row := HBoxContainer.new()
	pos_row.add_theme_constant_override("separation", 8)
	pos_row.custom_minimum_size.y = 24
	var pos_l := DccTheme.label("Elevation", "text_dim", DccTheme.FS_SMALL)
	pos_l.custom_minimum_size.x = _FIELD_LABEL_W
	pos_row.add_child(pos_l)
	var pos_slider := HSlider.new()
	pos_slider.min_value = 0.0
	pos_slider.max_value = 1.0
	pos_slider.step = 0.005
	pos_slider.value = float(pair[0])
	pos_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pos_slider.custom_minimum_size.y = 14
	pos_slider.focus_mode = Control.FOCUS_NONE
	pos_row.add_child(pos_slider)
	var pos_readout := DccTheme.mono_label("%d m" % int(round(_ramp_metres(float(pair[0])))), "text", DccTheme.FS_SMALL)
	pos_readout.custom_minimum_size.x = 56
	pos_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	pos_row.add_child(pos_readout)
	pos_slider.value_changed.connect(func(v: float): pos_readout.text = "%d m" % int(round(_ramp_metres(v))))
	pos_slider.drag_ended.connect(func(_c: bool): _on_stop_position_committed(idx, pos_slider.value))
	sec.add_child(pos_row)

	var del := DccWidgets.action(sec, "Delete", _on_stop_delete.bind(idx))
	del.disabled = stops.size() <= 1
	if stops.size() <= 1:
		del.tooltip_text = "A ramp needs at least two stops (§1.9's own refusal toast)."

## Metres above sea level a relative ramp position stands for on THIS world --
## the same formula `render_workspace.gd::_ramp_metres` uses, duplicated for
## the same "no shared home for a two-line helper" reason `_build_ramp_bar`
## gives. `0` before parameters are read, which reads as `0 m` honestly
## rather than a metre figure derived from a peak nobody has measured yet.
func _ramp_metres(at: float) -> float:
	if not bridge.params_available():
		return 0.0
	var peak = bridge.param_get("peak_m")
	return at * (float(peak) if peak != null else 0.0)

## Re-finds a stop by the position it should now have, after any edit that
## can change `get_color_ramp()`'s own sort order (a moved stop crossing a
## neighbour, an add, a reverse). `_stops_selected` is a plain index into
## that sorted list, so it cannot simply be kept from before such an edit --
## this re-derives it from the one thing that is still true: the position the
## edit itself just gave the stop we care about.
func _find_stop_near(stops: Array, position: float) -> int:
	var best := -1
	var best_d := INF
	for i in stops.size():
		var d: float = absf(float(stops[i][0]) - position)
		if d < best_d:
			best_d = d
			best = i
	return best

func _on_stops_mode_changed(i: int, modes: Array) -> void:
	bridge.set_ramp_mode(String(modes[i]))
	_rebuild()

func _on_stop_color_changed(c: Color, idx: int) -> void:
	var stops: Array = bridge.color_ramp()
	if idx < 0 or idx >= stops.size():
		return
	var old := stops[idx][1] as Color
	stops[idx][1] = Color(c.r, c.g, c.b, old.a)
	if bridge.set_color_ramp(stops) <= 0:
		return
	_rebuild()

func _on_stop_position_committed(idx: int, target: float) -> void:
	var stops: Array = bridge.color_ramp()
	if idx < 0 or idx >= stops.size():
		return
	stops[idx][0] = target
	if bridge.set_color_ramp(stops) <= 0:
		return
	_stops_selected = _find_stop_near(bridge.color_ramp(), target)
	_rebuild()

func _on_stop_delete(idx: int) -> void:
	var stops: Array = bridge.color_ramp()
	if stops.size() <= 1 or idx < 0 or idx >= stops.size():
		return
	stops.remove_at(idx)
	if bridge.set_color_ramp(stops) <= 0:
		return
	_stops_selected = -1
	_rebuild()

## New stop in the widest gap, coloured with what the ramp already shows
## there, and selected -- `render_workspace.gd::_on_add_stop`'s own
## algorithm, reused rather than reinvented; the "and selects it" half is
## §1.9's own spec, which that panel (no stop-selection concept) doesn't need.
func _on_stops_add() -> void:
	var stops: Array = bridge.color_ramp()
	var sorted := stops.duplicate()
	sorted.sort_custom(func(a, b): return float(a[0]) < float(b[0]))
	var at := 0.5
	var col := Color(0.5, 0.5, 0.5)
	if sorted.size() >= 2:
		var best := -1.0
		for i in sorted.size() - 1:
			var gap: float = float(sorted[i + 1][0]) - float(sorted[i][0])
			if gap > best:
				best = gap
				at = (float(sorted[i][0]) + float(sorted[i + 1][0])) * 0.5
				col = (sorted[i][1] as Color).lerp(sorted[i + 1][1] as Color, 0.5)
	elif sorted.size() == 1:
		at = clampf(float(sorted[0][0]) + 0.25, 0.0, 1.0)
		col = sorted[0][1] as Color
	stops.append([at, col])
	if bridge.set_color_ramp(stops) <= 0:
		return
	_stops_selected = _find_stop_near(bridge.color_ramp(), at)
	_rebuild()

## The design's own Reverse: the same colours, top to bottom.
func _on_stops_reverse() -> void:
	var stops: Array = bridge.color_ramp()
	var target := -1.0
	if _stops_selected >= 0 and _stops_selected < stops.size():
		target = 1.0 - float(stops[_stops_selected][0])
	for s in stops:
		s[0] = 1.0 - float(s[0])
	if bridge.set_color_ramp(stops) <= 0:
		return
	if target >= 0.0:
		_stops_selected = _find_stop_near(bridge.color_ramp(), target)
	_rebuild()

# -- Annotation (`rdAnno`, §1.10) --------------------------------------------
#
# **Appended** while the Label or Icon tool is armed (`rdMode4()` rule 3) --
# one section for both, matching §1.3's own title table. Reads
# `bridge.label_get_selected()`/`label_list()`/`icon_list()` fresh every
# rebuild, so there is no draft of its own to fall out of step.
#
# **The selected label's own text field commits on submit/defocus, not on
# every keystroke.** `cartography_workspace.gd`'s own left-dock Labels panel
# already carries a fuller seven-field form for the same selection (size,
# size mode, angle, font, colour, plus Confirm/Cancel over the same
# `label_bridge` edit session `label_confirm_edit`/`label_cancel_edit` gates)
# -- its own doc comment names §1.10's own shape as what it was standing in
# for. Both this section and that form write through the identical
# `label_set()`, so neither can hold a value the other disagrees with once
# either one loses focus; committing on submit/defocus here (matching that
# form's own Font field, not its Text field's more eager live-apply) keeps
# the two from visibly disagreeing while one is mid-keystroke.

func _build_anno(body: Control) -> void:
	var sel := bridge.label_get_selected()
	if sel >= 0:
		var lb := bridge.label_get(sel)
		if not lb.is_empty():
			_build_anno_selected(body, sel, lb)

	## §1.3's `ANNOTATION` -- the tool's own name, which is this section's job
	## to carry now that the dock header keeps naming the selection instead
	## (`CTX_TITLES`' own note). The old header said only "Placed · this
	## session", which was a fine subtitle under a dock titled `Annotation` and
	## names nothing on its own once that title is the selection's.
	var sec := DccWidgets.section(body, "Annotation · placed this session")
	var label_count := bridge.label_list().size()
	var icon_count := bridge.icon_list().size()
	_field(sec, "Labels", str(label_count))
	_field(sec, "Icons", str(icon_count))
	var clear := DccWidgets.action(sec, "Clear all", _on_anno_clear_all)
	clear.disabled = label_count == 0 and icon_count == 0
	if clear.disabled:
		clear.tooltip_text = "Nothing placed yet."
	DccWidgets.note(sec,
		"Labels and icons are presentation -- they add nothing to and take nothing from the world model.")

func _build_anno_selected(body: Control, idx: int, lb: Dictionary) -> void:
	var sec := DccWidgets.section(body, "Selected label")

	var text_row := HBoxContainer.new()
	text_row.add_theme_constant_override("separation", 8)
	text_row.custom_minimum_size.y = 24
	var text_edit := LineEdit.new()
	text_edit.text = String(lb.get("text", ""))
	text_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text_edit.text_submitted.connect(func(v: String): _on_anno_text_committed(idx, v))
	text_edit.focus_exited.connect(func(): _on_anno_text_committed(idx, text_edit.text))
	text_row.add_child(text_edit)
	sec.add_child(text_row)

	## `arc` is clamped `[-1,1]` in the engine (`label_bridge.rs`'s own
	## `set_arc`) -- §1.10 draws it `-50…+50`. `/50` is an exact round-trip of
	## that range, not a fabricated one: the reference's own arc control is a
	## percentage, and this port renormalised it the same way Hardness/
	## Softness were renormalised to `0..1` rather than kept as a second,
	## differently-scaled percentage field (`DECISIONS.md` §7k).
	var arc_pct := int(round(float(lb.get("arc", 0.0)) * 50.0))
	var arc_row := HBoxContainer.new()
	arc_row.add_theme_constant_override("separation", 8)
	arc_row.custom_minimum_size.y = 24
	var arc_l := DccTheme.label("Arc", "text_dim", DccTheme.FS_SMALL)
	arc_l.custom_minimum_size.x = _FIELD_LABEL_W
	arc_row.add_child(arc_l)
	var arc_slider := HSlider.new()
	arc_slider.min_value = -50
	arc_slider.max_value = 50
	arc_slider.step = 1
	arc_slider.value = arc_pct
	arc_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	arc_slider.custom_minimum_size.y = 14
	arc_slider.focus_mode = Control.FOCUS_NONE
	arc_row.add_child(arc_slider)
	var arc_readout := DccTheme.mono_label(str(arc_pct), "text", DccTheme.FS_SMALL)
	arc_readout.custom_minimum_size.x = 34
	arc_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	arc_row.add_child(arc_readout)
	arc_slider.value_changed.connect(func(v: float): arc_readout.text = str(int(v)))
	arc_slider.drag_ended.connect(func(_c: bool): _on_anno_arc_committed(idx, arc_slider.value))
	sec.add_child(arc_row)

	var del := DccWidgets.action(sec, "✕ delete label", _on_anno_delete.bind(idx))
	del.add_theme_color_override("font_color", DccTheme.c("block"))
	DccWidgets.note(sec,
		"Size, size mode, angle, font and colour, plus Confirm/Cancel over this same edit -- Cartography ▸ Labels.")

func _on_anno_text_committed(idx: int, text: String) -> void:
	bridge.label_set(idx, {"text": text})
	if app != null and app.viewport != null:
		app.viewport.refresh_annotations()
	_rebuild()

func _on_anno_arc_committed(idx: int, pct: float) -> void:
	bridge.label_set(idx, {"arc": clampf(pct / 50.0, -1.0, 1.0)})
	if app != null and app.viewport != null:
		app.viewport.refresh_annotations()
	_rebuild()

func _on_anno_delete(idx: int) -> void:
	bridge.label_delete(idx)
	if app != null and app.viewport != null:
		app.viewport.refresh_annotations()
	if app != null:
		app.set_status("hint", "label deleted", "text_ghost")
	_rebuild()

func _on_anno_clear_all() -> void:
	bridge.label_clear_all()
	bridge.icon_clear_all()
	if app != null and app.viewport != null:
		app.viewport.refresh_annotations()
	if app != null:
		app.set_status("hint", "labels and icons cleared", "text_ghost")
	_rebuild()

# -- Territory (`rdTerr`, §1.12) ---------------------------------------------
#
# **Appended** while the Territory tool is armed (`rdMode4()` rule 4 --
# unconditional on the tool). This comment used to end that sentence "so it
# wins over Faction/Settlement whenever Territory is armed", which was true of
# the code and is now the thing the owner's 2026-09-03 ruling overturned:
# **it wins over nothing.** A selected settlement or faction keeps the dock and
# this section arrives under it. That is the single case the ruling names --
# "the dock flips away from a selected settlement the moment a tool arms" --
# and it was the case measured failing.
#
# `civilization_workspace.gd` used to route a territory commit to this dock's
# own Faction context instead, with its own comment naming that gap
# ("right_dock.gd is explicitly not this pass's to change" --
# `_commit_territory`'s doc) -- that call is repointed to `show_territory`
# as part of wiring this section in.
#
# **Stats are commit-only, not live per dab.** `civ_faction_territory_stats`
# reads the COMMITTED `civ.territory`, never the in-progress draft
# (`CivTools::paint_at` only touches `territory_draft`, baked in on
# `commit()`) -- the same reason `civilization_workspace.gd`'s own
# tool-options row only refreshes this figure at arm/commit/discard, not per
# drag sample. `show_territory()` is called at exactly those same three
# events, never per dab.

func _build_territory(body: Control) -> void:
	if _terr_faction < 0:
		DccWidgets.note(DccWidgets.section(body, "Territory"), "No faction armed.")
		return
	var roster := _faction_roster(_terr_faction)
	var sec := DccWidgets.section(body, "Territory")

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	if not roster.is_empty():
		var sw := ColorRect.new()
		sw.color = Color8(int(roster.get("color_r", 0)), int(roster.get("color_g", 0)), int(roster.get("color_b", 0)))
		sw.custom_minimum_size = Vector2(12, 12)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		head.add_child(sw)
	var name_text := ("%d · %s" % [_terr_faction, String(roster.get("culture", "?")).capitalize()]) \
		if not roster.is_empty() else "faction %d" % _terr_faction
	head.add_child(DccTheme.mono_label(name_text, "text", DccTheme.FS_SMALL))
	sec.add_child(head)

	var stats := bridge.civ_faction_territory_stats(_terr_faction)
	if stats.is_empty():
		_accent_readout(sec, "Claimed cells", "0",
			"civ_faction_territory_stats() returned nothing for this faction -- no committed territory yet.")
		_field(sec, "Area", "—", "No committed territory to measure.", false)
		_field(sec, "Contested", "—", "No committed territory to measure.", false)
	else:
		_accent_readout(sec, "Claimed cells", _thousands(float(stats.get("claimed_cells", 0))),
			"civ_faction_territory_stats() over the committed territory raster -- redrawn at arm, commit and " +
			"discard, not per paint dab (see this section's own header note).")
		_field(sec, "Area", "%s km²" % _thousands(float(stats.get("area_km2", 0.0))))
		_field(sec, "Contested", str(int(stats.get("contested_cells", 0))), "Cells more than one faction has claimed.")

	DccWidgets.note(sec,
		"A claim dab is an ungated circle -- civ_tools_bridge::CivTools::paint_at pushes no coastline mask, so " +
		"painting into open water claims it too (civilization_workspace.gd's own disclosed gap, not new here).")

# -- Way / Route draft (`rdMode4()` rule 7, §1.14) --------------------------
#
# **Appended** while Way or Route is armed over a non-empty draft. Every
# reading is computed here from the draft this dock was handed plus the live
# world -- nothing in this section is a number somebody else measured earlier.
#
# **Two of §1.14's four rows are deliberately not the reference's.**
#
# * `LENGTH` is `fmtKm(Σ|Δ| × 2.5)` there -- 2.5 km per cell, hardcoded in a
#   prototype with one map size. This port has a real one, so it goes through
#   `_route_length_text()`, the same call the Route context and the collapsed
#   readout already use.
# * `GRADE · MAX` is the literal **`4.2%`** in the reference's own markup, which
#   §1.14 flags as hardcoded. A placeholder is not a reading, so it is computed
#   from `sample_cell()`'s `elevation_m` at each waypoint and **dashed with its
#   reason** when the elevation is not there to read -- never defaulted to a
#   plausible-looking zero (`MISTAKES.md`, "no value as a plausible value").
#
# **These are the draft's lengths, not the committed way's, and the note says
# so.** `way_commit()`/`route_commit()` Dijkstra-join the waypoints, so the
# path that actually lands is longer than the straight chain measured here and
# does not exist until the commit runs. The reference's own footnote -- *"Esc
# commits the way · hovering shows the live snap preview"* -- is half false in
# this port (`tool_overlay.set_path_preview()` draws the placed points, not a
# hover preview), so the true half is kept and the false half replaced by the
# disclosure above rather than copied.

func _build_way(body: Control) -> void:
	var is_way := _way_owner == "way"
	var sec := DccWidgets.section(body, "Way draft" if is_way else "Route draft")
	var pts := _way_draft.size()
	_accent_readout(sec, "Waypoints" if is_way else "Stops", str(pts),
		"Points placed in this draft. The draft itself lives in Rust " +
		"(way_append_point / route_append_stop, which snap each point before " +
		"returning); this is the copy infrastructure_workspace.gd keeps for the " +
		"canvas preview, handed over on every change.")
	_field(sec, "Length", _route_length_text(_way_draft),
		"Straight-line total over the placed points, converted with this world's own " +
		"map width -- not the reference's hardcoded 2.5 km cell. Not the committed " +
		"distance either: the commit Dijkstra-joins the waypoints, so what lands is " +
		"a cell-by-cell path and not this straight chain.", true, true)
	var grade := _way_max_grade()
	if grade.is_empty():
		_field(sec, "Grade · max", "—",
			"Needs two points on distinct cells over a generated world -- sample_cell() " +
			"omits elevation_m when there is no height field to read, and this row " +
			"dashes rather than printing a 0 that would read as flat ground.", false, true)
	else:
		_field(sec, "Grade · max", "%.1f%%" % float(grade[0]),
			"Steepest segment of the draft: |Δ elevation_m| over the ground distance " +
			"between two consecutive points. The reference hardcodes 4.2% here; this row " +
			"is computed from the live height field instead. The committed way is routed " +
			"cell by cell, so its own maximum will differ from this one.", true, true)
	if is_way and _way_kind != "":
		_field(sec, "Surface", _way_kind,
			"The way type this draft was begun with. Changing it restarts the draft: " +
			"WayDraft::way_type is fixed for a draft's whole lifetime, unlike the " +
			"reference, which re-read civWayType at commit time.", true, true)
	DccWidgets.note(sec,
		("Esc commits the way" if is_way else "Esc commits the route") +
		" and leaves the tool armed; arming any other tool commits it too. Commit, " +
		"Discard and the way type are in the tool options bar, where the design puts them.")

## `GRADE · MAX`'s reading, or `[]` when there is nothing honest to report --
## the caller dashes the row with the reason rather than printing a number.
##
## The steepest straight-line segment of the draft: |Δ elevation| over the
## ground distance between two consecutive points, both in metres. The same
## segments `Length` totals, for the same reason -- the routed path the commit
## produces does not exist yet.
##
## Bails on the **first** point whose `elevation_m` is absent rather than
## skipping it: a maximum computed over some of the segments is not the
## maximum, and reporting it as one would be worse than the dash.
func _way_max_grade() -> Array:
	if bridge == null or not bridge.has_world or _way_draft.size() < 2:
		return []
	var gw := bridge.grid_size().x
	if gw <= 0 or bridge.last_width_km <= 0.0:
		return []
	var m_per_cell := bridge.last_width_km * 1000.0 / float(gw)
	var elev := PackedFloat64Array()
	for p in _way_draft:
		var cell := bridge.sample_cell(roundi(p.x), roundi(p.y))
		if not cell.has("elevation_m"):
			return []
		elev.append(float(cell["elevation_m"]))
	var worst := -1.0
	for i in range(1, _way_draft.size()):
		var run := _way_draft[i - 1].distance_to(_way_draft[i]) * m_per_cell
		if run <= 0.0:
			continue
		worst = maxf(worst, absf(elev[i] - elev[i - 1]) / run * 100.0)
	return [] if worst < 0.0 else [worst]

# -- Shared row/field vocabulary ------------------------------------------
#
# `DccWidgets` has category/section/group/advanced plus slider/toggle/
# choice/number/action/note -- every one of those either edits a value or
# explains a rule. A dock built entirely from live readouts needs a plain
# label:value row that reports rather than edits, which `DccWidgets`
# doesn't have; adding one there would be adding a control that edits
# nothing to a file whose whole job is drawing editable rows, so it stays
# local to the file that actually needs read-only inspection.

const _FIELD_LABEL_W := 116

## **The pane's width is an input, never an output.** A Godot `Label` with no
## trimming reports its own text width as its *minimum* width, and that number
## travels up through the row, the section, the `ScrollContainer` (whose
## horizontal scrolling is disabled, so it forwards its child's minimum width
## whole) and into the right dock's `PanelContainer`, whose `custom_minimum_size
## .x` is a floor and not a ceiling. So a value wider than the dock made the
## dock wider, and since the viewport is the one `SIZE_EXPAND_FILL` child of
## the same `HBoxContainer`, the map lost exactly those pixels -- on every
## mouse-move that changed a value's length. Measured before this line existed:
## the Sample panel's "Nearest settlement" row forced a 286 px minimum against
## a 300 px dock, so the dock breathed 300 <-> 319 px and the viewport 440 <->
## 421 px as the cursor crossed from one settlement's neighbourhood to another.
##
## `text_overrun_behavior` is the fix rather than `clip_text`: both collapse the
## reported minimum width to 1 px, but the ellipsis says the value was trimmed
## instead of amputating it silently. Every row in this dock reports rather than
## edits, so there is nothing here that a trimmed value can break.
##
## `label_w` narrows the label column for the handful of rows whose *value* is
## the content (a settlement name) rather than a short reading.
## `GUI_GAP_REGISTER.md` §57 / `UNWIRED_FUNCTIONS.md` "the tablet interior
## walk": this dock's single most-repeated row builder, resolved at
## construction the same way `DccWidgets._row()` now is -- `role_px("row_min_h")`
## for the row, `"fs_prose"` for the label (prose, `DccTheme.label()`) and
## `"fs_readout"` for a mono value (Plex, `DccTheme.mono_label()`). A non-mono
## value stays prose-sized, matching the label beside it.
func _field(parent: Control, label_text: String, value_text: String,
		tooltip: String = "", reachable: bool = true, mono: bool = false,
		label_w: int = _FIELD_LABEL_W) -> Label:
	var tablet := DccTheme.is_tablet()
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	row.tooltip_text = tooltip
	var label_fs := DccTheme.role_px("fs_prose") if tablet else DccTheme.FS_SMALL
	var l := DccTheme.label(label_text, "text_dim", label_fs)
	l.custom_minimum_size.x = label_w
	l.clip_text = true
	row.add_child(l)
	var token := "text" if reachable else "text_ghost"
	var v: Label
	if mono:
		var mono_fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_SMALL
		v = DccTheme.mono_label(value_text, token, mono_fs)
	else:
		v = DccTheme.label(value_text, token, label_fs)
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	v.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	row.add_child(v)
	parent.add_child(row)
	return v

## §6: "elevation (large accent readout)". The only such readout the dock
## has. Returns the value `Label` so `on_cursor_sampled` can write metres
## into it in place, the same way every other Sample row is updated.
func _accent_readout(parent: Control, label_text: String, value_text: String, tooltip: String) -> Label:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	wrap.tooltip_text = tooltip
	var caption_fs := DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL
	wrap.add_child(DccTheme.label(label_text, "text_dim", caption_fs))
	## This used to hard-code `26` and argue that §6's "one big accent readout
	## per context" was deliberately pinned. `BUILD_ANSWERS.md` §2.4 reverses
	## that on 2026-08-31 -- *"The three unscaled values -- all three now scale.
	## They were oversights."* -- so the size comes from `fs_hero`'s own
	## desktop/tablet pair instead.
	##
	## Through `DccTheme.hero()`, not a second `role_px("fs_hero")` read here:
	## that helper's own doc names this exact readout ("§6's elevation") and had
	## no caller, so this row is what it was written for. It also draws in Plex,
	## which is what `mono_label()` is for -- every numeric readout in this
	## shell -- where `DccTheme.label()` left the dock's one big number in prose.
	var v := DccTheme.hero(value_text)
	## Same rule as `_field()`: this is the fastest row in the dock to outgrow
	## the pane, and it is rewritten on every mouse-move.
	v.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	wrap.add_child(v)
	parent.add_child(wrap)
	return v

# -- Coordinate readout ----------------------------------------------------
#
# Two rules, one shared reason: the coordinate is the one reading in this dock
# that changes on literally every mouse-move, so it is both the row most able
# to destabilise the pane's width and the row most able to lie about how much
# the map actually knows.

## One cell's real size in km -- `map_width_km / gw`, the single quotient
## `GENERATION_PARAMETERS.md` says every resolution-dependent figure in this
## port is derived from (`terrain_detail_k`, `river_flow_thresh`,
## `civ_catchment_radius_cells`, `suppression_radius_cells` all take it).
## `0.0` when there is no world, or when a loaded save carried no extent.
func _cell_km() -> float:
	var gw := bridge.grid_size().x
	if gw <= 0 or bridge.last_width_km <= 0.0:
		return 0.0
	return bridge.last_width_km / float(gw)

## How many decimals a km coordinate may honestly carry **for this world**.
##
## Every raster in this port is per-cell; nothing it can be asked -- elevation,
## biome, slope, territory -- distinguishes two points inside one cell. So the
## finest meaningful step in a coordinate is one cell, and one cell is not a
## fixed size: 2 400 km over 384 cells is 6.25 km per cell, 1 000 km over 2 048
## is 0.49 km, 200 km over 2 048 is 0.098 km. A fixed decimal count would print
## false precision on the first and throw real precision away on the last.
##
## The rule: the displayed step is the **largest power of ten no larger than
## one cell** -- `ceil(-log10(cell_km))`, clamped to 0..3. That gives 0
## decimals at 6.25 km/cell (1 km steps, one step ≈ a sixth of a cell), 1 at
## 0.49 km/cell, 2 at 0.098 km/cell. A decimal count can only move in factors
## of ten, so "no finer than a cell, and within one factor of ten of it" is the
## tightest honest rule available; the clamp at 3 stops a pathological
## metre-scale world asking for a column of digits nobody reads.
func _coord_decimals() -> int:
	var km := _cell_km()
	if km <= 0.0:
		return 0
	return clampi(int(ceil(-log(km) / log(10.0))), 0, 3)

## The cell size as the `Position` row's tooltip states it, at whatever
## precision the number itself needs to be legible.
func _cell_km_text() -> String:
	var km := _cell_km()
	if km <= 0.0:
		return "of unknown size"
	if km >= 1.0:
		return "%.2f km" % km
	return "%d m" % int(round(km * 1000.0))

## Pad to a fixed character count so the pair keeps its columns in a mono
## label. Cosmetic only -- the row's *width* is already nailed down by
## `_field()`'s ellipsis; this is what stops the two numbers sliding past each
## other as digits come and go.
func _coord_pad(s: String, chars: int) -> String:
	return " ".repeat(maxi(0, chars - s.length())) + s

## `[position, cell]` for the two Sample coordinate rows. Both are pairs on one
## line: X and Y are one reading, and one row per axis was both a worse read
## and a second label free to size itself to its own digit count.
func _coord_texts(gx: float, gy: float, valid: bool) -> Array:
	if not valid or not bridge.has_world:
		return ["—", "—"]
	var gs := bridge.grid_size()
	var cw := str(maxi(gs.x, gs.y) - 1).length()
	var cell := "%s · %s" % [
		_coord_pad(str(int(round(gx))), cw), _coord_pad(str(int(round(gy))), cw)]
	var km := _cell_km()
	if km <= 0.0:
		## A loaded save with no recorded extent: the cell index is still real,
		## the km figure would be invented.
		return ["—", cell]
	var fmt := "%%.%df" % _coord_decimals()
	var kw := (fmt % maxf(bridge.last_width_km, bridge.last_height_km)).length()
	return ["%s · %s km" % [
		_coord_pad(fmt % (gx * km), kw), _coord_pad(fmt % (gy * km), kw)], cell]

func _nearest_settlement_text(gx: float, gy: float, valid: bool) -> String:
	if not valid:
		return "—"
	var list := bridge.settlements()
	if list.is_empty():
		return "—"
	var best_name := ""
	var best_d2 := INF
	for s in list:
		var d: Dictionary = s
		var dx := float(d["x"]) - gx
		var dy := float(d["y"]) - gy
		var d2 := dx * dx + dy * dy
		if d2 < best_d2:
			best_d2 = d2
			best_name = String(d["name"])
	return "%s (%.0f cells)" % [best_name, sqrt(best_d2)]
