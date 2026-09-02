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
const CTX_SCULPT := "sculpt"
const CTX_JOURNEY := "journey"
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

## `05-right-dock-and-bars.md` §1.8-§1.12, GUI replacement stage 5. Four
## contexts from `rdMode4()`'s own fall-through table (§1.2b): `tool ===
## 'biome'` -> paint, `tool` is `label`/`icon` -> anno, `tool === 'territory'`
## -> terr, `domain==='CARTO' && tool==='inspect'` -> stops. All four read
## live engine state fresh on every rebuild, the same "no private draft"
## shape `CTX_SCULPT` already uses -- there is nothing here for a second
## editor to disagree with.
##
## **One tool id genuinely differs from the design file's own markup**:
## `PaintTarget` (`paint_bridge.rs`) is `Biome`/`Terrain`/`Splat`, and this
## shell's own armed-tool id is `"paint"`, not `"biome"` -- `world_workspace
## .gd` registers `register_tool_click_handler("paint", ...)`. The context
## constant below is named for what this port actually calls it.
const CTX_PAINT := "paint"
## `rdStops` (§1.9). Triggered when the CARTO domain is active with no more
## specific tool armed -- see `show_stops()`'s own doc for the two call
## sites this needs (a domain switch fires no `tool_armed` of its own).
const CTX_STOPS := "stops"
## `rdAnno` (§1.10) -- Label and Icon share one context, exactly as they
## share one right-dock title in the design's own table.
const CTX_ANNO := "anno"
## `rdTerr` (§1.12). Named `CTX_TERR` (not `CTX_TERRITORY`) to match the
## design doc's own short id and the grep this stage's own brief was
## written against.
const CTX_TERR := "territory"

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
var _region_result: Dictionary = {}
var _wildlife_region: Dictionary = {}
var _journey_view: JourneyPlannerView = null   ## CTX_JOURNEY delegate -- see `show_journey()`.

## -- CTX_PAINT. `_paint_ctx_layer` mirrors `world_workspace.gd`'s own private
## `_paint_layer` -- the caller passes it on every `show_paint()`, the same
## shape `show_measure(result, mode)` already uses, rather than this file
## reaching into another workspace's state. `_paint_on_pick` is bound by that
## same caller so a click on this dock's legend can arm a value without this
## file guessing at the other four `paint_set_brush` fields (radius/hardness/
## softness/land_only) it has no way to know -- see `show_paint()`.
var _paint_ctx_layer := "biome"
var _paint_on_pick: Callable = Callable()

## -- CTX_STOPS. Which ramp stop (by position in `bridge.color_ramp()`, sorted)
## this dock's own "Selected stop" section edits -- local to this file, since
## the ramp itself carries no selection of its own (unlike labels/icons,
## which do: `label_get_selected()`/`icon_get_selected()`).
var _stops_selected := -1

## -- CTX_TERR. The faction the Territory tool is currently armed for --
## passed in on every `show_territory()`, the same reason `_paint_ctx_layer`
## is: `civilization_workspace.gd` owns `_territory_faction`, not this file.
var _terr_faction := -1

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
	bridge.generation_finished.connect(func(_ok: bool): _rebuild())
	bridge.world_loaded.connect(func(): _rebuild())
	_rebuild()

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

## Called by `world_workspace.gd` whenever the Sculpt panel is active or the
## "sculpt" tool is armed (arming the tool via a feature/preset button, or a
## stroke ending -- the dock's own Generation pipeline / Sculpt toggle was
## removed by the v3 menu pass, which folded Sculpt into WORLD ▸ Terrain) -- never on a bare cursor move, so Sample stays the default
## everywhere else. `_build_sculpt` below reads the stack fresh from
## `bridge.sculpt_list_stamps()` on every `_rebuild()`, so this setter carries
## no data of its own the way `show_measure`/`show_region` do.
func show_sculpt_stack() -> void:
	_context = CTX_SCULPT
	_rebuild()

## Visual sweep (2026-08-20): switching away from WORLD while the Sculpt
## panel had claimed this dock left it stuck showing the Stamp stack in
## CIVIL/CARTOGRAPHY, contradicting `show_sculpt_stack()`'s own doc comment
## ("Sample stays the default everywhere else") -- Sculpt is a World-only
## tool (`world_workspace.gd`) with nothing else that clears the context on
## a domain switch. Called from `app.gd`'s `_on_workspace_changed`. Settlement/
## route/faction/measure/region selections are left untouched -- those stay
## meaningful across a domain switch (Inspect's own selection is wired
## domain-independently in `app.gd`'s `_wire_selection`), so only Sculpt's
## own context is domain-bound enough to reset here.
func leave_sculpt_context() -> void:
	if _context == CTX_SCULPT:
		_context = CTX_SAMPLE
		_rebuild()

## Called by `journey_planner_view.gd` when the JOURNEY tool arms -- claims
## `right_dock_body` for the results panel (`JOURNEY_PLANNER_SPEC.md` §8),
## delegating the actual content back to `view.build_results()` rather than
## duplicating its rendering here. Mirrors `show_sculpt_stack()`'s own
## delegation shape (that one re-reads `bridge.sculpt_*` fresh each rebuild;
## this one re-reads `view`'s own cached compute result, since a fresh
## `jp_compute()` per rebuild would be a wasted boundary crossing on every
## unrelated `right_dock.gd` refresh).
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
	_context = CTX_JOURNEY
	_journey_view = view
	_rebuild()

## Called by `journey_planner_view.gd` after every recompute, while Journey
## is still the active context -- cheaper than `show_journey()` re-running
## the whole dispatch when only the numbers changed, and a no-op otherwise
## (Journey no longer armed, or something else grabbed the dock meanwhile).
func refresh_journey() -> void:
	if _context == CTX_JOURNEY:
		_rebuild()

## Called by `journey_planner_view.gd` when the JOURNEY tool disarms --
## returns the dock to Sample rather than leaving a stale results panel
## behind once there is nothing live driving it.
func clear_journey() -> void:
	if _context == CTX_JOURNEY:
		_journey_view = null
		_context = CTX_SAMPLE
		_rebuild()

## Called by `world_workspace.gd`'s own `_on_tool_armed` when `"paint"` arms,
## and again after every layer switch / commit / discard / stroke release --
## the same "re-announce the current picture" cadence `show_sculpt_stack()`
## already uses, since this dock keeps no draft of its own to patch in place.
## `on_pick_value`, when given, is bound to a closure that knows the other
## four `Brush` fields this file cannot see; omitted (or an invalid
## `Callable`), the legend's rows still show swatch/label/count, just not a
## click-to-arm affordance -- see `_build_paint`.
func show_paint(layer: String, on_pick_value: Callable = Callable()) -> void:
	_context = CTX_PAINT
	if layer != "":
		_paint_ctx_layer = layer
	_paint_on_pick = on_pick_value
	_rebuild()

## Mirrors `leave_sculpt_context()` exactly -- called from `app.gd`'s
## workspace-switch handler for the same reason: Biome paint is a WORLD-only
## tool with nothing else that clears this context on a domain switch.
func leave_paint_context() -> void:
	if _context == CTX_PAINT:
		_context = CTX_SAMPLE
		_rebuild()

## `rdMode4()` rule 6: `domain==='CARTO' && tool==='inspect'`. Two call sites
## need this, and neither alone covers the rule -- `cartography_workspace.gd`'s
## `_on_any_tool_armed` (armed tool changes, but a domain switch fires no
## `tool_armed` of its own) and `app.gd`'s `_on_workspace_changed` (domain
## changes, but arming Inspect while already in CARTO fires no workspace
## change). Both are wired to call this only when both halves of the rule
## already hold, so this file does not re-check them.
func show_stops() -> void:
	_context = CTX_STOPS
	_rebuild()

func leave_stops_context() -> void:
	if _context == CTX_STOPS:
		_stops_selected = -1
		_context = CTX_SAMPLE
		_rebuild()

## Called by `cartography_workspace.gd`'s `_on_any_tool_armed` for both
## `"label"` and `"icon"` -- one context for both tools, matching §1.3's own
## right-dock title table (`ANNOTATION` names neither tool by itself) and
## `_dispatch()`'s single `CTX_ANNO` branch.
func show_anno() -> void:
	_context = CTX_ANNO
	_rebuild()

func leave_anno_context() -> void:
	if _context == CTX_ANNO:
		_context = CTX_SAMPLE
		_rebuild()

## Called by `civilization_workspace.gd` on arming `"territory"`, on every
## faction re-pick while it stays armed, and after a commit/discard -- the
## live stats this shows (`civ_faction_territory_stats`) only change at
## commit, but re-announcing costs nothing and keeps this in step with
## whichever faction is actually armed.
func show_territory(faction_id: int) -> void:
	_context = CTX_TERR
	_terr_faction = faction_id
	_rebuild()

func leave_territory_context() -> void:
	if _context == CTX_TERR:
		_context = CTX_SAMPLE
		_rebuild()

# -- Dispatch ---------------------------------------------------------------

## §6's own per-context header title, mirroring `DccWidgets.section()`'s title
## one call below it -- kept as one table here rather than a `match` inline in
## `_rebuild()` so a new `CTX_*` can't add a body section without this table
## reminding whoever adds it that the dock chrome needs the same name.
const CTX_TITLES := {
	CTX_SETTLEMENT: "Settlement", CTX_ROUTE: "Route", CTX_RIVER: "River",
	CTX_FACTION: "Faction", CTX_MEASURE: "Measure", CTX_REGION: "Region select",
	CTX_SCULPT: "Stamp stack", CTX_JOURNEY: "Journey",
	CTX_WILDLIFE: "Ecoregion", CTX_HISTORY: "History",
	## `05-right-dock-and-bars.md` §1.3. `CTX_PAINT`'s title is dynamic
	## ("PAINT · BIOME"/"PAINT · TERRAIN"/"PAINT · SPLAT") and built in
	## `_current_title()` instead -- this static table has no per-instance
	## data to build it from.
	CTX_STOPS: "Ramp · stops", CTX_ANNO: "Annotation", CTX_TERR: "Territory",
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
	_sample_rows.clear()
	_dispatch(body)
	app.set_right_dock_title(_current_title())
	_push_dock_readout()

## `_build_settlement`/`_build_journey` both fall back to `_build_sample()`
## when their own data is missing (a settlement deselected out from under the
## dock, or Journey armed with no `_journey_view` yet) -- mirrored here so the
## header never claims a context the body didn't actually draw.
func _current_title() -> String:
	if _context == CTX_SETTLEMENT and _settlement_data == null:
		return "Sample"
	if _context == CTX_JOURNEY and _journey_view == null:
		return "Sample"
	## §1.3: `PAINT · ` + the target's own name, upper-cased -- the one
	## title this table can't hold statically.
	if _context == CTX_PAINT:
		return "Paint · %s" % _paint_ctx_layer.capitalize()
	return String(CTX_TITLES.get(_context, "Sample"))

## RD-11: §6's own last line -- "elevation for Sample, layer dots for
## Layers, stamp count for the stack" -- is the right dock's collapsed
## primary readout, and `DccShell.set_dock_readout("right", …)` already
## exists for it, kept current whether or not the dock is actually
## collapsed (`dcc_shell.gd`'s own doc comment). `world_workspace
## ._push_dock_readout()` calls the left dock's equivalent on every
## rebuild; this dock never called the right-dock one at all. No "Layers"
## context exists here yet (RD-10 is still an omission), so this reads one
## honest number per context that DOES exist rather than inventing the
## missing one.
func _push_dock_readout() -> void:
	if app == null:
		return
	app.set_dock_readout("right", _dock_readout_text())

func _dock_readout_text() -> String:
	match _context:
		CTX_SETTLEMENT:
			if _settlement_data == null:
				return _sample_elev.text if _sample_elev != null else "—"
			return String((_settlement_data as Dictionary).get("name", "—"))
		CTX_ROUTE:
			return _route_length_text(_route_entry.get("points", PackedVector2Array()))
		CTX_RIVER:
			return "no rivers"
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
		CTX_SCULPT:
			return ("%d stamps" % bridge.sculpt_list_stamps().size()) if bridge.has_world else "no world"
		CTX_HISTORY:
			var st := bridge.undo_stats()
			return "%d of %d reversible" % [int(st.get("depth", 0)), bridge.undo_ledger().size()]
		CTX_JOURNEY:
			if _journey_view == null:
				return _sample_elev.text if _sample_elev != null else "—"
			return _journey_view.readout_text()
		CTX_PAINT:
			return ("%s cells" % _thousands(float(bridge.paint_painted_counts().get("total", 0)))) if bridge.has_world else "no world"
		CTX_STOPS:
			var n := bridge.color_ramp().size() if bridge.ramp_api else 0
			return ("%d stop%s" % [n, "" if n == 1 else "s"]) if n > 0 else "no ramp"
		CTX_ANNO:
			return "%d labels · %d icons" % [bridge.label_list().size(), bridge.icon_list().size()]
		CTX_TERR:
			var stats := bridge.civ_faction_territory_stats(_terr_faction) if _terr_faction >= 0 else {}
			return ("%s cells" % _thousands(float(stats.get("claimed_cells", 0)))) if not stats.is_empty() else "no claim"
		_:
			return _sample_elev.text if _sample_elev != null else "—"

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
		CTX_SCULPT:
			_build_sculpt(body)
		CTX_JOURNEY:
			_build_journey(body)
		CTX_HISTORY:
			_build_history(body)
		CTX_PAINT:
			_build_paint(body)
		CTX_STOPS:
			_build_stops(body)
		CTX_ANNO:
			_build_anno(body)
		CTX_TERR:
			_build_territory(body)
		_:
			_build_sample(body)

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

## No `get_rivers()` exists and nothing in the viewport can select one --
## unlike Route, this context has no live trigger today. Implemented anyway
## so `_dispatch()` is complete and honest rather than silently dropping the
## branch, matching `§6` and `STRANDED_TOOLS.md`'s own discipline of
## recording the gap rather than hiding it.
func _build_river(body: Control) -> void:
	var sec := DccWidgets.section(body, "River")
	## The clause struck here on 2026-09-01 said "the only river-derived output
	## that crosses the GDExtension boundary is baked into
	## build_color_texture()'s rendered raster", and this same file disproves
	## it three rows up: the Sample readouts report Strahler order and
	## discharge per cell off `stream_order`/`flow_discharge`
	## (`SAMPLE_FIELDS`' own Drainage row, three above this one, cites both by
	## name), `layers_popover.gd` draws
	## `strahler` as a live layer, and `measure_section` labels a crossing
	## "River · order 3". What genuinely does not cross is the *entity*, which
	## is `measure_bridge.rs`'s own wording and is what this note now says.
	DccWidgets.note(sec,
		"No hydrological river ENTITY is exposed to Godot. cartalith-hydrology " +
		"computes river networks internally (order, discharge, catchment) for " +
		"erosion and settlement suitability, and per-CELL order and discharge do " +
		"cross the boundary -- Sample reads them, the Strahler layer draws them, " +
		"and a measured section labels its river crossings by order. What nothing " +
		"does is aggregate a channel run into one river: there is no get_rivers(), " +
		"so there is no river to name, to total a length for, or to select in the " +
		"viewport.")
	for f in ["Name", "Length", "Source elevation", "Discharge", "Catchment", "Tributaries", "Navigation"]:
		_field(sec, f, "—", "No get_rivers() binding.", false)
	var actions := DccWidgets.group(sec, "Actions")
	## All three used to share one seven-word tooltip ("No river binding to act
	## on."), which named the same gap three times and told a reader nothing
	## about what each Action would need. Each now says what is specifically
	## missing for *it* -- the three are blocked by three different absences,
	## not by one. Found by the 2026-08-31 unwired audit.
	var why := {
		"Hydrology":
			"Would report this river's Strahler order, discharge and channel width. " +
			"All three are computed (strahler_from_receivers, compute_flow's " +
			"flow_discharge, river_width_scale_k) but only ever as per-CELL rasters -- " +
			"nothing aggregates a channel run into one river with its own readings, so " +
			"there is no per-river figure to report.",
		"Edit geometry":
			"Would move the river's course. The course is not stored: it is re-traced " +
			"from the receiver tree on demand (trace_river_polylines, the way the GeoJSON " +
			"export and the urban pass both do it), so there is no polyline to edit, and " +
			"no write path from an edited one back into the flow field it came from.",
		"Analyse catchment":
			"Would report the upstream area draining into this river. That area lives " +
			"inside compute_flow's accumulation and crosses the boundary only one cell at " +
			"a time (Sample -> Drainage); no #[func] sums it over a channel, and summing " +
			"it here would mean one boundary crossing per upstream cell.",
	}
	for label_text in ["Hydrology", "Edit geometry", "Analyse catchment"]:
		var b := DccWidgets.action(actions, label_text, func(): pass)
		b.disabled = true
		b.tooltip_text = String(why[label_text])

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

## The collapsed dock's one number, per mode.
func _measure_readout() -> String:
	if _measure_result.is_empty():
		return "no reading"
	match _measure_mode:
		"area":
			return "%s km²" % _thousands(float(_measure_result.get("projected_km2", 0.0)))
		"radius":
			return "r %.0f km" % float(_measure_result.get("radius_km", 0.0))
		"vertical":
			return "%+.0f m" % float(_measure_result.get("delta_m", 0.0))
		"section":
			return "%.0f km section" % float(_measure_result.get("length_km", 0.0))
		"bearing":
			var segs: Array = _measure_result.get("segments", [])
			return ("%03d°" % int(round(float((segs[0] as Dictionary).get("bearing_deg", 0.0))))) if not segs.is_empty() else "no bearing"
		_:
			return "%.1f km" % float(_measure_result.get("total_km", 0.0))

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
	_accent_readout(sec, "Total length", "%.0f km" % float(_measure_result.get("total_km", 0.0)),
		"Summed leg by leg, each leg through cartalith_spatial::measure -- the same km scale every route length in this port uses.")
	DccWidgets.note(sec, "%d segment%s · %d points" % [
		segments.size(), "" if segments.size() == 1 else "s",
		int(_measure_result.get("point_count", 0))])

	var segs := DccWidgets.group(sec, "Segments")
	for i in segments.size():
		var seg: Dictionary = segments[i]
		var b := float(seg.get("bearing_deg", 0.0))
		_field(segs, "%d" % (i + 1), "%.0f km · %03d° · ↺ %03d°" % [
			float(seg.get("km", 0.0)), int(round(b)), int(round(fmod(b + 180.0, 360.0)))],
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
	_field(sec, "Distance", "%.1f km" % float(seg.get("km", 0.0)))
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
	_field(sec, "Straight line", "%.1f km" % straight,
		("Along-path exceeds straight-line by %.1f km." % diff) if diff > 0.01 else "")
	var ob := float(_measure_result.get("overall_bearing_deg", 0.0))
	_field(sec, "Overall bearing", "%03d° · ↺ %03d°" % [int(round(ob)), int(round(fmod(ob + 180.0, 360.0)))])
	var relief := bool(_measure_result.get("has_relief", false))
	_field(sec, "Sinuosity", ("%.2f" % float(_measure_result.get("sinuosity", 1.0))) if relief else "—",
		"Along-path over straight-line. 1.00 is a straight run.", relief)
	_field(sec, "Δ elevation", ("%+.0f m" % float(_measure_result.get("elevation_delta_m", 0.0))) if relief else "—",
		"First point to last, from the height field.", relief)
	_field(sec, "3D length", ("%.1f km" % float(_measure_result.get("total_km_3d", 0.0))) if relief else "—",
		"The chain followed over the ground rather than across the map.", relief)
	if not relief:
		DccWidgets.note(sec, "The three relief rows need a generated world: a loaded save carries no height substrate to read.")

## The canvas's foot: save · copy · CSV · plan journey.
##
## **Copy is real; save and CSV are one button, and it is Copy.** There is no
## saved-measurements store in this port and inventing one would be a
## persistence feature, not a measuring one -- what the canvas's three export
## buttons are actually for is getting the numbers out, and the clipboard does
## that with no file dialog, no format decision and no new state. Said out
## loud below rather than drawn as two disabled buttons.
func _build_measure_actions(body: Control) -> void:
	var actions := DccWidgets.group(body, "Actions")
	DccWidgets.action(actions, "Copy reading", _on_measure_copy)
	DccWidgets.action(actions, "Plan a journey", func(): app.open_journey_planner())
	DccWidgets.note(actions,
		"The canvas's Saved measurements list, Save and CSV are not built: no measurement store exists, " +
		"and Copy already puts every number above on the clipboard as tab-separated text a spreadsheet reads directly.")

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
		DccWidgets.note(cr,
			"Rivers are described by Strahler order, not by name: no river entity crosses the GDExtension boundary " +
			"(see this dock's own River context), so there is no toponym to print.")
	_build_measure_actions(body)

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
	_field(sec, "Extent",
		"%d × %d cells" % [int(_region_result.get("w", 0)), int(_region_result.get("h", 0))])
	_field(sec, "Extent (km)",
		"%.0f × %.0f km" % [float(_region_result.get("w_km", 0.0)), float(_region_result.get("h_km", 0.0))])
	_field(sec, "Cells", str(int(_region_result.get("cell_count", 0))))
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

func _build_journey(body: Control) -> void:
	if _journey_view == null:
		_build_sample(body)
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
# Shown while the "paint" tool is armed (this port's own id for the design's
# `biome` tool -- see the `CTX_PAINT` const's own doc). Reads
# `world_workspace.gd`'s live paint-editor state fresh every rebuild, the
# same "no private draft" shape `CTX_SCULPT` already uses -- Commit/Discard
# here and Commit/Discard in the left-dock Biome paint panel are the same two
# engine calls, so neither can disagree with the other about what is pending.
# (They CAN both be on screen showing the same number, which is fine -- see
# `CTX_SCULPT`'s own header comment on why that is not the two-drafts bug
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

	var sec := DccWidgets.section(body, "Painted · %s" % layer.capitalize())
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
		app.viewport.set_preview_texture(bridge.build_paint_preview_texture())
	show_paint(_paint_ctx_layer, _paint_on_pick)

# -- Ramp · stops (`rdStops`, §1.9) ------------------------------------------
#
# Shown whenever the CARTO domain is active with nothing more specific armed
# (`rdMode4()` rule 6) -- see `show_stops()`'s own doc for the two triggers
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
# Shown while the Label or Icon tool is armed (`rdMode4()` rule 3) -- one
# context for both, matching §1.3's own title table. Reads
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

	var sec := DccWidgets.section(body, "Placed · this session")
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
# Shown while the Territory tool is armed (`rdMode4()` rule 4 -- unconditional
# on the tool, so it wins over Faction/Settlement whenever Territory is
# armed). `civilization_workspace.gd` used to route a territory commit to
# this dock's own Faction context instead, with its own comment naming this
# exact gap ("right_dock.gd is explicitly not this pass's to change" --
# `_commit_territory`'s doc) -- that call is repointed to `show_territory`
# as part of wiring this context in.
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
