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
var _region_result: Dictionary = {}
var _journey_view: JourneyPlannerView = null   ## CTX_JOURNEY delegate -- see `show_journey()`.

## Live-updated in place on every `cursor_sampled` rather than triggering a
## full `_rebuild()` -- the overlay emits that signal on every mouse-motion
## event over the viewport, and tearing the dock down and rebuilding it at
## that rate would be needless churn for sixteen labels.
var _sample_x: Label
var _sample_y: Label
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
	if _sample_x == null:
		_rebuild()
		return
	_sample_x.text = ("%.0f" % gx) if valid else "—"
	_sample_y.text = ("%.0f" % gy) if valid else "—"
	_sample_nearest.text = _nearest_settlement_text(gx, gy, valid)
	var cell: Dictionary = bridge.sample_cell(int(round(gx)), int(round(gy))) if valid else {}
	_sample_elev.text = _elevation_text(cell)
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
## Measure chain -- `result` is `measure_result()`'s own dict, straight
## through.
func show_measure(result: Dictionary) -> void:
	_context = CTX_MEASURE
	_measure_result = result
	_rebuild()

## Called by `GlobalTools` when a Region marquee commits -- `result` is
## `region_get()`'s own dict.
func show_region(result: Dictionary) -> void:
	_context = CTX_REGION
	_region_result = result
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
}

func _rebuild() -> void:
	if app == null:
		return
	var body := app.right_dock_body
	for child in body.get_children():
		body.remove_child(child)
		child.queue_free()
	_sample_x = null
	_sample_y = null
	_sample_elev = null
	_sample_nearest = null
	_sample_rows.clear()
	_dispatch(body)
	app.set_right_dock_title(_current_title())

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
	_sample_x = _field(sec, "X", "—", "Cursor grid-cell X. Live once the cursor is over a generated map.")
	_sample_y = _field(sec, "Y", "—", "Cursor grid-cell Y. Live once the cursor is over a generated map.")

	_sample_elev = _accent_readout(sec, "Elevation", "—",
		"Metres above sea level at the cursor cell, from WorldState::field through " +
		"metersPerUnit()'s own anchoring (1 - seaLevel maps to peak altitude). " +
		"Negative below the waterline, which is the honest reading for an ocean cell.")

	for f in SAMPLE_FIELDS:
		_sample_rows[f["label"]] = _field(sec, f["label"], "—", f["tip"], false)

	_sample_nearest = _field(sec, "Nearest settlement", "—",
		"Computed here from get_settlements()'s x/y against the cursor cell.",
		valid)

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

	var actions := DccWidgets.group(sec, "Actions")
	for label_text in ["Economy", "Politics", "Logistics"]:
		var b := DccWidgets.action(actions, label_text, func(): pass)
		b.disabled = true
		b.tooltip_text = "No per-settlement %s panel exists yet -- see Data ▸ World data tables for the same fields, read-only." % label_text.to_lower()

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
	_field(sec, "Name", String(e.get("name", "—")))
	_field(sec, "Type", String(e.get("way_type", "")).capitalize() if _route_kind == "road" else "Sea lane")

	var pts: PackedVector2Array = e.get("points", PackedVector2Array())
	_field(sec, "Points", str(pts.size()))
	_field(sec, "Length", _route_length_text(pts))

	var unreachable := ["Stages", "Vessels", "Cost trace", "Per-stage overrides", "Daily stages"]
	for f in unreachable:
		_field(sec, f, "—",
			"get_roads()/get_sea_routes() carry only {points, brks, way_type, name} -- " +
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

	_field(sec, "Faction", str(_faction_id))
	_field(sec, "Provinces", str(mine.size()))

	var names: Array[String] = []
	for p in mine:
		names.append(String(p.get("name", "")))
	_field(sec, "Roster", ", ".join(names) if not names.is_empty() else "—",
		"" if not names.is_empty() else "No provinces carry this faction id.",
		not names.is_empty())

	_field(sec, "Territory", "—",
		"Territory is only exposed as a rendered overlay (build_territory_texture()) -- " +
		"no per-faction cell count or area query exists.", false)
	_field(sec, "State religion", "—",
		"cartalith-civ computes a has_religion flag internally " +
		"(civ_faction_aggregates, FactionAggregate) but get_provinces() doesn't carry " +
		"it and there is no get_faction_aggregates() binding.", false)

# -- Measure --------------------------------------------------------------

## §4.5.1's own right-dock spec: "Segment table (bearing, length), total,
## straight-line vs along-path difference." `measure_result()` carries every
## field this needs directly (`segments`, `total_km`, `straight_line_km`) --
## nothing here is derived a second time.
func _build_measure(body: Control) -> void:
	var sec := DccWidgets.section(body, "Measure")
	var segments: Array = _measure_result.get("segments", [])
	if segments.is_empty():
		DccWidgets.note(sec, "Click the map to drop points; Esc clears the chain.")
	else:
		for i in segments.size():
			var seg: Dictionary = segments[i]
			_field(sec, "Segment %d" % (i + 1),
				"%.1f km · %d°" % [float(seg.get("km", 0.0)), int(round(float(seg.get("bearing_deg", 0.0))))])
	sec.add_child(DccTheme.rule())
	_field(sec, "Total", "%.1f km" % float(_measure_result.get("total_km", 0.0)))
	var straight: float = float(_measure_result.get("straight_line_km", 0.0))
	var total: float = float(_measure_result.get("total_km", 0.0))
	var diff := total - straight
	_field(sec, "Straight line", "%.1f km" % straight,
		"Along-path exceeds straight-line by %.1f km." % diff if diff > 0.01 else "", diff <= 0.01 or straight > 0.0)
	var clear := DccWidgets.action(sec, "Clear", func(): bridge.measure_clear(); show_measure({}))
	clear.disabled = segments.is_empty()

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
	## Not wired to `app.open_world_data()`: that window is the settlement/
	## province/economy tables (`world_data_window.gd`), not §9's tile-pyramid
	## export route -- `region_export_tiles()` is bound and tested
	## (`LOD_TILING_INTEGRATION_SCOPE.md`'s M2, "Z4 is done"), but the Data
	## Manager panel that would call it doesn't exist. Honest disable rather
	## than a button that opens the wrong window.
	var actions := DccWidgets.group(sec, "Actions")
	var send := DccWidgets.action(actions, "Send to Data ▸ Export", func(): pass)
	send.disabled = true
	send.tooltip_text = "region_export_tiles() is bound and tested; the Data Manager panel to call it doesn't exist yet."

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

func _field(parent: Control, label_text: String, value_text: String,
		tooltip: String = "", reachable: bool = true) -> Label:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 22
	row.tooltip_text = tooltip
	var l := DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = _FIELD_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var v := DccTheme.label(value_text, "text" if reachable else "text_ghost", DccTheme.FS_SMALL)
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
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
	wrap.add_child(v)
	parent.add_child(wrap)
	return v

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
