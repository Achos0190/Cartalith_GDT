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

var app: DccApp
var bridge: EngineBridge

var _context := CTX_SAMPLE
var _settlement_data: Variant = null
var _settlement_index := -1
var _route_entry: Dictionary = {}
var _route_kind := ""      ## "road" | "sea"
var _faction_id := -1
var _measure_result: Dictionary = {}
var _measure_mode := "distance"   ## One of `GlobalTools.MEASURE_MODES`' ids.
var _region_result: Dictionary = {}
var _wildlife_region: Dictionary = {}
var _journey_view: JourneyPlannerView = null   ## CTX_JOURNEY delegate -- see `show_journey()`.

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
	_sample_nearest.text = _nearest_settlement_text(gx, gy, valid)
	var cell: Dictionary = bridge.sample_cell(int(round(gx)), int(round(gy))) if valid else {}
	_sample_elev.text = _elevation_text(cell)
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
		var text := _sample_field_text(f["key"], cell)
		row.text = text
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
func show_faction(faction_id: int) -> void:
	_context = CTX_FACTION
	_faction_id = faction_id
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
## "sculpt" tool is armed (switching the dock's own Generation pipeline /
## Sculpt toggle to Sculpt, arming the tool via a feature/preset button, or a
## stroke ending) -- never on a bare cursor move, so Sample stays the default
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

# -- Dispatch ---------------------------------------------------------------

## §6's own per-context header title, mirroring `DccWidgets.section()`'s title
## one call below it -- kept as one table here rather than a `match` inline in
## `_rebuild()` so a new `CTX_*` can't add a body section without this table
## reminding whoever adds it that the dock chrome needs the same name.
const CTX_TITLES := {
	CTX_SETTLEMENT: "Settlement", CTX_ROUTE: "Route", CTX_RIVER: "River",
	CTX_FACTION: "Faction", CTX_MEASURE: "Measure", CTX_REGION: "Region select",
	CTX_SCULPT: "Stamp stack", CTX_JOURNEY: "Journey",
	CTX_WILDLIFE: "Ecoregion",
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
		CTX_JOURNEY:
			if _journey_view == null:
				return _sample_elev.text if _sample_elev != null else "—"
			return _journey_view.readout_text()
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

	for f in SAMPLE_FIELDS:
		_sample_rows[f["label"]] = _field(sec, f["label"], "—", f["tip"], false)

	## "Nearest settlement" was this dock's single widest row -- and its value
	## changes on every mouse-move, which is what made the whole pane breathe
	## (see `_field()`). The label is shortened and its column narrowed so the
	## name and its distance both still fit inside the pane's own width instead
	## of pushing against it.
	_sample_nearest = _field(sec, "Nearest", "—",
		"Computed here from get_settlements()'s x/y against the cursor cell.",
		valid, false, 60)

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
	_field(sec, "Water access", water if water != "" else "—",
		"" if water != "" else
			"This settlement's suitability terms carry no water_access entry for this cell.",
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
	rt.add_theme_font_size_override("normal_font_size", DccTheme.FS_SMALL)
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
	DccWidgets.note(sec,
		"No hydrological river entity is exposed to Godot. cartalith-hydrology " +
		"computes river networks internally (order, discharge, catchment) for " +
		"erosion and settlement suitability, but the only river-derived output that " +
		"crosses the GDExtension boundary is baked into build_color_texture()'s " +
		"rendered raster -- there is no get_rivers() and nothing in the viewport " +
		"can select one.")
	for f in ["Name", "Length", "Source elevation", "Discharge", "Catchment", "Tributaries", "Navigation"]:
		_field(sec, f, "—", "No get_rivers() binding.", false)
	var actions := DccWidgets.group(sec, "Actions")
	for label_text in ["Hydrology", "Edit geometry", "Analyse catchment"]:
		var b := DccWidgets.action(actions, label_text, func(): pass)
		b.disabled = true
		b.tooltip_text = "No river binding to act on."

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
	## for a "roster entry", singular: `get_factions()` (lib.rs:3442) carries
	## the real per-faction culture/colour/settlement_count, so that's what
	## fills this section now.
	var roster: Dictionary = {}
	for f in bridge.get_factions():
		var d: Dictionary = f
		if int(d.get("id", -1)) == _faction_id:
			roster = d
			break

	_field(sec, "Faction", str(_faction_id))
	if roster.is_empty():
		_field(sec, "Culture", "—",
			"No get_factions() entry for faction %d -- generate a world first." % _faction_id, false)
		_field(sec, "Settlements", "—", "", false)
	else:
		_field(sec, "Culture", String(roster.get("culture", "?")).capitalize())
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
	_field(sec, "State religion", "—",
		"cartalith-civ computes a has_religion flag internally " +
		"(civ_faction_aggregates, FactionAggregate) but get_provinces() doesn't carry " +
		"it and there is no get_faction_aggregates() binding.", false)

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
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 22
	var l := DccTheme.label("Colour", "text_dim", DccTheme.FS_SMALL)
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
	trail.add_child(DccTheme.label("#%02X%02X%02X" % [r, g, b], "text", DccTheme.FS_SMALL))
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
		DccWidgets.note(sec, "No stamps yet -- arm a Sculpt feature (World ▸ Sculpt) and draw a stroke on the map.")
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

	var actions := DccWidgets.group(body, "Commit")
	var commit_btn := DccWidgets.action(actions, "%s Commit to map" % DccIcons.SYMBOLS["tick"], _on_sculpt_stack_commit, true)
	commit_btn.disabled = stamps.is_empty()
	var discard_btn := DccWidgets.action(actions, "Discard draft", _on_sculpt_stack_discard)
	discard_btn.disabled = stamps.is_empty()
	DccWidgets.note(body,
		"Commit bakes the whole stamp stack into the heightfield and marks the tiles it " +
		"touched stale -- it does not re-run erosion, hydrology or climate " +
		"(DCC_SHELL_SPEC.md header correction #1). No finalize/lock state exists in this " +
		"engine yet -- no bake/LOD pipeline exists to freeze against (world_workspace.gd's " +
		"own Finalize section has the same gap), so there is nothing real to report for §6's " +
		"own finalize-lock note.")

func _sculpt_stamp_row(parent: Control, d: Dictionary, selected: int) -> void:
	var idx := int(d.get("index", -1))
	var hidden := bool(d.get("hidden", false))
	var label_text := String(d.get("label", "?"))
	var pts := int(d.get("point_count", 0))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = 20
	var mark := DccTheme.mono_label(DccIcons.SYMBOLS["off"] if hidden else DccIcons.SYMBOLS["on"],
		"text_ghost" if hidden else "text_dim", DccTheme.FS_TINY)
	row.add_child(mark)
	var text := "#%d %s (%d pt%s)" % [idx, label_text, pts, "" if pts == 1 else "s"]
	var l := DccTheme.mono_label(text, "accent" if idx == selected else "text", DccTheme.FS_SMALL)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.clip_text = true
	row.add_child(l)
	var select_btn := Button.new()
	select_btn.flat = true
	select_btn.focus_mode = Control.FOCUS_NONE
	select_btn.text = "selected" if idx == selected else "select"
	select_btn.disabled = idx == selected
	select_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
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
func _field(parent: Control, label_text: String, value_text: String,
		tooltip: String = "", reachable: bool = true, mono: bool = false,
		label_w: int = _FIELD_LABEL_W) -> Label:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 22
	row.tooltip_text = tooltip
	var l := DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = label_w
	l.clip_text = true
	row.add_child(l)
	var token := "text" if reachable else "text_ghost"
	var v: Label
	if mono:
		v = DccTheme.mono_label(value_text, token, DccTheme.FS_SMALL)
	else:
		v = DccTheme.label(value_text, token, DccTheme.FS_SMALL)
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
	wrap.add_child(DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL))
	var v := DccTheme.label(value_text, "accent", 26)
	## Same rule as `_field()`: at 26 px a readout is the fastest row in the
	## dock to outgrow the pane, and this one is rewritten on every mouse-move.
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
