extends Workspace
class_name InfrastructureWorkspace

## Roads, rivers, ports, trade, logistics, and the Way/Route drawing tools
## (§4.5.4) -- formerly the standalone INFRA rail domain (§3).
##
## **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
## name and can be absorbed by civil").** INFRA no longer has its own rail
## button. This class is unchanged in what it does -- it still owns its own
## category builders below and its own Way/Route tool click/drag/escape
## handlers -- but it is now composed *into* `CivilizationWorkspace`
## (`civilization_workspace.gd`'s own `_infra` field) as a nested
## `VBoxContainer` appended after CIVIL's own five categories, instead of
## getting an `app.register_workspace()` call and a rail button of its own.
## `_nested` (set true by `civilization_workspace.gd` before calling
## `setup()`) is the only behavioural difference this composition needs:
## CIVIL's own `_build_tools()` already draws ONE combined TOOLS block
## carrying Settlement/Territory *and* Way/Route in a single row, so this
## file's own `_build_tools()` must not draw a second, duplicate one -- it
## still registers the Way/Route click/drag/escape handlers either way, since
## those are independent of which file drew the buttons.
##
## Roads and sea routes are read from the engine today (`get_roads`,
## `get_sea_routes`) -- those two calls are this whole file's read-only data
## source, per the task that built it. Rivers are not read *here*, but they do
## have bindings: `get_rivers(min_order)` and `river_at()`, which
## `right_dock.gd`'s River context uses. This line claimed "no binding at all"
## until 2026-09-03; see `rivers_note()` below for what is and is not there.
## Ports here means coastal settlements, derived from `get_settlements()`'s
## real `coastal` field, not a separate concept the engine models. Logistics
## (the journey planner) is engine-complete per `JOURNEY_PLANNER_SCOPE.md`
## **and crosses the boundary**: `jp_options`, `jp_default_plan`, `jp_compute`,
## `jp_plan_for_route`, `jp_pack_range`, `jp_vessel_matrix` and the
## `route_count`/`route_get` pair, all wrapped in `engine_bridge.gd` and all
## driven from `journey_planner_view.gd`. This sentence read "exports nothing
## past that crate boundary, like culture" until 2026-09-01, and was wrong
## twice over -- the Logistics section below had already recorded the
## correction for itself in 2026-08-19, and culture exports too
## (`get_cultures()`, CV-02).
##
## Drawing a new way or route (`infra_tools_bridge.rs`'s `InfraTools`,
## `STRANDED_TOOLS.md` row 11) has an engine, a surface, and -- since
## `GUI_GAP_REGISTER.md` IN-02 was closed -- somewhere for the result to
## land. The TOOLS block below arms Way/Route, click-chains a draft, and
## commits or discards it; `get_roads()`/`get_sea_routes()` now append
## `InfraTools::ways` to the generated network they return, each entry
## tagged `manual: true`, so `bridge.roads()`/`bridge.sea_routes()` DO
## include a hand-drawn way once committed. `_commit_way` therefore repaints
## the map (`_refresh_map_ways`) and refills the Roads ▸ Hand-drawn list
## (`_refresh_manual_ways`) instead of only printing a status line.
##
## That the two sources share one getter is the reference's arrangement
## rather than a shortcut: `_civCommitWay` (reference line 26077) pushes a
## hand-drawn way straight onto the same flat `civWays` array as the
## generated network, and the draw pass branches on `type` alone -- a manual
## `road` and a generated `road` are drawn identically. `manual` exists to
## be *listed* and to survive a network rebuild, never to be styled apart,
## so nothing here gives hand-drawn ways their own colour.
##
## The Route tool (`route_commit`) took the same treatment on 2026-08-24,
## `GUI_GAP_REGISTER.md` IN-09. A committed route was never in
## `get_roads()`/`get_sea_routes()` at all -- it lives in `InfraTools::routes`,
## readable only through `route_count()`/`route_get(i)`, which nothing on the
## GDScript side called. So a route committed correctly (a live check solved
## a 578 km, 516-point path with zero unreachable legs) and then appeared
## nowhere at all -- no map line, no list row. `_commit_route`
## now repaints the map's own route layer (`_refresh_map_routes` ->
## `map_overlay.gd`'s `_manual_routes`) and refills a Routes list beside the
## ways one (`_refresh_manual_routes`).
##
## The Routes list stopped being read-only later the same day, with IN-09's
## second half: `route_delete`/`route_set_name` are now real `#[func]`s, so
## each row carries the reference journey list's own three affordances --
## select, rename, delete (`_civRenderJourneyList`, reference line 17235) --
## and selecting one drives `map_overlay.gd`'s block-2b `sel` stroke.
##
## Still genuinely missing: rename/retype/delete of an existing *way* (the
## reference's way-properties editor -- there is no `way_set_name`/
## `way_delete` `#[func]`; only routes got theirs), and manual sea lanes route
## through the same navy/dashed style as generated ones with no per-way
## condition field.
##
## Rows here (Roads/Ports' network lists) read only; clicking a road or sea
## route pins it into the right dock (`right_dock.gd`'s Route context), and
## that now works for a hand-drawn way too, since it is the same dictionary
## shape.
##
## **`04-left-dock.md` §6c and §6d closed 2026-09-01.** §6c's `ROUTES` block
## is real: a compact, click-to-plan row per committed route under the Ways
## & routes category's own Network list (`_build_routes_teaser`), plus its
## section footnote quoted verbatim. §6d's full TRAVELER/SEASON/CARRIAGE/
## ROUTE/STOPS accordion is deliberately NOT embedded in the Travel category
## -- `_fill_logistics()`'s own doc comment carries the reasoning, which
## `_refresh_manual_routes()`'s doc comment (below) already laid the
## groundwork for. Both reuse the one real open path, `app.open_journey_
## planner()` -- never a second one.

const WAY_TYPE_ORDER := ["highway", "regional", "road", "track"]

## `way_begin`'s real vocabulary (`infra_tools_bridge::parse_way_type`),
## confirmed by reading the Rust source rather than trusting either of the
## two similarly-named-but-different lists nearby: **not** `WAY_TYPE_ORDER`
## above (that's `get_roads()`'s auto-generated-network tiers -- highway/
## regional/road/track, a different field entirely) and **not**
## `DCC_SHELL_SPEC.md` §4.5.4's own "road / track / trail / bridge" (the
## engine has no `trail` or `bridge`; `parse_way_type`'s own doc comment
## calls that spec list wrong against the tested four-entry enum). These
## four are what `ManualWayType` actually has.
const WAY_DRAW_TYPES := ["road", "track", "sea_lane", "ancient"]
const WAY_DRAW_TYPE_LABELS := ["Road", "Track", "Sea lane", "Ancient"]

## Which of our two tools (if either) is currently armed -- tracked locally
## because `app.tool_armed` fires with the *new* id already written into
## `app.armed_tool` (`app.gd`'s `arm_tool`), so there is no other way to
## learn what is being armed away from. `""` when neither is armed.
var _active_infra_tool := ""

## The in-progress click chains, kept in parallel with the engine's own
## draft for the on-canvas preview -- same reason `GlobalTools._measure_points`
## exists: `way_commit`/`route_commit` return only an index, never the
## waypoint list back. **Not necessarily pixel-identical to the committed
## way**: `way_append_point`/`route_append_stop` snap each point server-side
## before this file ever sees the result (§4.5.4's "snap to places, on by
## default"), so a click near a place or existing way may land the engine's
## copy slightly off of what is drawn here. Good enough for a live sketch of
## the chain; not claimed to be more than that.
var _way_points: PackedVector2Array = PackedVector2Array()
var _route_points: PackedVector2Array = PackedVector2Array()
var _way_type := "road"

## Set true by `civilization_workspace.gd` before `setup()`, when this
## instance is composed into CIVIL's own dock rather than standing alone --
## see this file's own class doc for why.
var _nested := false

## The Hand-drawn group's row host (`_build_manual_ways`), kept so a commit
## can refill it without a panel rebuild. `null` until `_build()` has run.
var _manual_list: Control = null

## The Route tool's own committed-list host, the same shape as `_manual_list`
## above (`GUI_GAP_REGISTER.md` IN-09). `null` until `_build()` has run.
var _manual_routes_list: Control = null

## `04-left-dock.md` §6c's own `ROUTES` block's row host -- a compact,
## click-to-plan list over the same committed routes `_manual_routes_list`
## above edits; see `_build_routes_teaser()`'s own doc for why it is a
## second, simpler view rather than a rename of that one. `null` until
## `_build_routes_teaser()` has run.
var _routes_teaser_section: Control = null

## The reference's `_civSelectedJourneyIdx`; `-1` for none. Lives here rather
## than in `map_overlay.gd` because the list is what changes it -- the overlay
## only draws it (`set_selected_manual_route`).
var _selected_route := -1

## The four data-backed categories' own body nodes, held so `rebuild_readouts()`
## can clear and refill exactly those and nothing else -- the same discipline
## `civilization_workspace.gd` uses for its own. Rivers has no field because it
## is no longer a category of this class at all: it moved to WORLD ▸ Hydrology
## with v3, and `rivers_note()` below is the one owner of its disclosure.
var _roads_body: Control
## The civ-authoring ruling's Network row (`LARGE_ITEM_RULINGS.md`,
## 2026-08-31). Held as fields because `_generate_roads_now` is a coroutine
## that has to find its own button again after the blocking engine call --
## the same reason `civilization_workspace.gd`'s `_recompute_btn` is a field.
var _gen_roads_btn: Button
var _clear_ways_btn: Button
var _roads_note: Label
var _ports_body: Control
var _trade_body: Control
var _logistics_body: Control
## `GUI_GAP_REGISTER.md` **IN-13**'s own body, deliberately NOT in
## `rebuild_readouts()` above: a match costs a real computation, so a
## generate clears it rather than silently re-running it. `TradeStore.clear()`
## in `app.gd` is what empties it.
var _flows_body: Control
## The Match button, held because `_build_flows()` runs once -- from CIVIL's
## `setup()`, before any world exists -- so nothing else would ever re-enable
## it. This is `GUI_GAP_REGISTER.md` RF-01's exact shape, found again by the
## probe pressing the real control rather than reading the source.
var _flows_run: Button

## **Where this workspace's categories attach** (2026-08-24, `design/Cartalith
## Menu Structure v3.dc.html`).
##
## v3 keeps every one of these subjects in CIVIL -- *"Roads are built by
## people, so the network is civilizational data; there is no separate
## logistics domain to move it to"* -- but names them the way a person would
## look for them: **Routes & ways**, **Travel** and **Trade**. Ports fold into
## Routes & ways (a port is where a sea lane meets a settlement, not a fifth
## subject) and Rivers leaves for WORLD ▸ Hydrology, which is where v3 puts
## the river network.
##
## So this class no longer draws five categories of its own. It holds the
## state and the Way/Route tool handlers, and CIVIL calls the three
## `build_*_into()` entry points below with its own category bodies. The fills
## (`_fill_roads` etc.) and `rebuild_readouts()` are unchanged -- they are
## keyed to the body nodes, which is what made re-parenting safe.
var _dock_hosted := false

func _build() -> void:
	_build_tools()
	## **This branch is unreachable today, and is kept deliberately.**
	## `civilization_workspace.gd` is the only thing that constructs this class
	## and it sets `_dock_hosted = true` before calling `setup()`, so the four
	## builders below never run; the live path is the `build_*_into()` entry
	## points further down, which CIVIL calls with its own category bodies.
	## Retained because that is the whole cost of keeping this class able to
	## stand on its own again -- what `_dock_hosted` exists for.
	##
	## `_build_rivers()` was a fifth line here until 2026-09-01, and it was the
	## one that could not be kept: Rivers left this dock for WORLD ▸ Hydrology
	## in the v3 re-parenting, and the category it drew held a second, drifting
	## copy of `rivers_note()`'s text -- the same disclosure with one owner and
	## two spellings, in a function nothing called.
	if not _dock_hosted:
		_build_roads()
		_build_ports()
		_build_trade()
		_build_logistics()

	## `GUI_GAP_REGISTER.md` RF-01. Everything above runs ONCE -- from
	## `civilization_workspace.gd`'s `_infra.setup()` at launch, before any
	## world exists -- so each category drew its "generate a world first" empty
	## state against an empty engine and, until this pair existed, nothing ever
	## re-ran it. Connected here rather than driven by the parent workspace so
	## this class keeps working unchanged if it is ever un-nested again (its
	## own `_nested` field is the only concession this composition needed, and
	## this is deliberately not a second one).
	bridge.generation_finished.connect(func(ok: bool): if ok: rebuild_readouts())
	bridge.world_loaded.connect(rebuild_readouts)

## Clear-and-refill for the four categories a generate or a loaded save
## invalidates. Public because `civilization_workspace.gd` composes this class
## and may want to drive it directly (its own recompute path, for instance);
## the two signals above are what actually call it today.
##
## Same cost profile as CIVIL's own `_rebuild_readouts()`: `get_roads`/
## `get_sea_routes`/`get_settlements`/`get_trade_balances`/`route_count` are
## pure reads of stored `Vec`s -- the widest of them copies a few hundred
## polyline points -- with no engine recompute behind any of them.
func rebuild_readouts() -> void:
	if _roads_body != null and is_instance_valid(_roads_body):
		_clear_body(_roads_body)
		_fill_roads(_roads_body)
	if _ports_body != null and is_instance_valid(_ports_body):
		_clear_body(_ports_body)
		_fill_ports(_ports_body)
	if _trade_body != null and is_instance_valid(_trade_body):
		_clear_body(_trade_body)
		_fill_trade(_trade_body)
	if _logistics_body != null and is_instance_valid(_logistics_body):
		_clear_body(_logistics_body)
		_fill_logistics(_logistics_body)
	## IN-13's Flows body refills from whatever `TradeStore` holds, which
	## `app.gd` has just cleared on this same world change -- so this puts the
	## "not matched yet" note back rather than showing the previous world's
	## numbers under the new world's name.
	if _flows_run != null and is_instance_valid(_flows_run):
		_flows_run.disabled = not bridge.has_world
	if _flows_body != null and is_instance_valid(_flows_body):
		_clear_body(_flows_body)
		_fill_flows()
	## A regenerate empties both hand-drawn stores, so both lists must go back
	## to their "None yet" notes rather than keep the previous world's rows.
	_refresh_manual_ways()
	_refresh_manual_routes()

## `remove_child` before `queue_free`: `queue_free` defers to the end of the
## frame, so a child left parented is still in `get_children()` while the
## refill runs and would draw twice for one frame.
static func _clear_body(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()

# -- Tools (§4.5.4: Way, Route) ------------------------------------------

## §4.5's TOOLS block: the three global tools (Inspect/Measure/Region,
## `GlobalTools.install`) plus this domain's own two. Skipped when `_nested`
## -- `civilization_workspace.gd`'s own combined TOOLS block already drew
## these same two buttons (plus Settlement/Territory) in one row, so drawing
## a second row here would duplicate them. The handler registration below
## still runs unconditionally: it is what makes the *buttons CIVIL drew*
## actually do something when clicked, regardless of which file drew them.
func _build_tools() -> void:
	if not _nested:
		DccWidgets.tools_block(self, app, app.tool_group, [
			{"id": "way", "glyph": "tool_way", "label": "Way (W)"},
			{"id": "route", "glyph": "tool_route", "label": "Route (⇧R)"},
		])
	app.register_tool_click_handler("way", func(gx, gy): _way_click(gx, gy))
	app.register_tool_click_handler("route", func(gx, gy): _route_click(gx, gy))
	## §4.5.6 lists Way, Route and Measure together as needing special Escape
	## handling. Verified against both `DCC_SHELL_SPEC.md` §4.5.4's own table
	## ("Esc commits (`_civCommitWay`)"/"(`_civCommitRoute`)") and the real
	## reference (`Cartalith Gen1 v2.10.html`, the `keydown` listener around
	## its `_civCommitRoute`/`_civCommitWay` calls, plus `_civSetTool`'s
	## "clicking the already-active tool button again turns it off (commits/
	## clears any in-progress way or route)"): Escape **commits** the draft
	## (attempting real Dijkstra routing, exactly what the Commit button
	## below does), it does not discard it, and neither `_civCommitWay` nor
	## `_civCommitRoute` ever reassigns `_civTool` -- the tool stays armed
	## afterward, ready to draw the next one. That is also `GlobalTools.
	## _measure_escape`'s own precedent ("leaves Measure armed... does not
	## fall through to the default tool-manager disarm"), so Way/Route match
	## Measure's "stays armed" half while differing on the other: Measure has
	## nothing to commit, Way/Route do.
	app.register_tool_escape_handler("way", _commit_way)
	app.register_tool_escape_handler("route", _commit_route)
	app.tool_armed.connect(_on_infra_tool_armed)

## Reacts to ANY tool arming anywhere in the app (`app.tool_armed` is one
## shared signal across every domain), not just ours -- see `_active_infra_
## tool`'s own doc for why a local flag, not `app.armed_tool`, is what tells
## us whether *we* were the one just disarmed.
func _on_infra_tool_armed(id: String) -> void:
	## Matches `_civSetTool`'s own "switching to any other tool commits
	## whichever of route/draw_way was active" -- not a discard. Runs before
	## the new tool arms so a stray in-progress draft from the last one never
	## quietly leaks into whatever comes next.
	if _active_infra_tool == "way" and id != "way":
		_commit_way()
	elif _active_infra_tool == "route" and id != "route":
		_commit_route()
	match id:
		"way":
			_active_infra_tool = "way"
			_way_points = PackedVector2Array()
			if not bridge.way_begin(_way_type) and not bridge.has_world:
				app.set_status("hint", "Generate a world first -- Way needs a generated map to route over.", "text_ghost")
			app.viewport.tool_overlay.set_path_preview(_way_points)
			_tool_options_way()
			_push_way_draft()
		"route":
			_active_infra_tool = "route"
			_route_points = PackedVector2Array()
			## The reference's Route tool has no mode control at all --
			## `_civCommitRoute` always calls `_civJoinDijkstraSegs(_civWaypoints,
			## 'mixed')`, hardcoded. `route_begin` genuinely accepts land/water
			## too (`infra_tools_bridge::parse_route_mode`'s own doc: "genuine,
			## tested cost domains... not UI-only labels"), but exposing that
			## choice here would be new UI with no reference precedent behind
			## it, so this matches the reference exactly instead.
			if not bridge.route_begin("mixed") and not bridge.has_world:
				app.set_status("hint", "Generate a world first -- Route needs a generated map to route over.", "text_ghost")
			app.viewport.tool_overlay.set_path_preview(_route_points)
			_tool_options_route()
			_push_way_draft()
		_:
			if _active_infra_tool != "":
				_active_infra_tool = ""
				app.viewport.tool_overlay.set_path_preview(PackedVector2Array())
				_push_way_draft()
				## Only reclaim the options bar for the plain "back to
				## Inspect" case. Measure/Region (`global_tools.gd`) set no
				## options-bar content of their own today, so guessing at
				## something to show while one of those is what just armed
				## would just be a different flavour of stale.
				if id == "inspect":
					_tool_options_infra_idle()

## Hands the live draft to the right dock's `rdMode4()` rule 7 section
## (`right_dock.gd::show_way()`, §1.14: waypoints, length, max grade, surface).
##
## Called from every point this file changes the draft, which is every place
## that already rebuilds the tool options row -- the same figure is in both, and
## the two would disagree the moment one of them was left out. Derives
## everything from `_active_infra_tool`, so the neither-armed case falls out of
## the default arm rather than needing its own call site.
##
## The right dock decides for itself whether to draw: it compares the owner id
## sent here against `app.armed_tool` live, so a draft pushed for a tool that is
## no longer armed can only suppress that section, never leave a stale one up.
func _push_way_draft() -> void:
	if app == null or app.right_dock_ctrl == null:
		return
	match _active_infra_tool:
		"way":
			app.right_dock_ctrl.show_way("way", _way_points,
				WAY_DRAW_TYPE_LABELS[maxi(0, WAY_DRAW_TYPES.find(_way_type))])
		"route":
			## No type label: the Route tool has no type control at all, here or
			## in the reference (`route_begin("mixed")` is hardcoded -- see
			## `_on_infra_tool_armed`), so the dock omits the row rather than
			## inventing a value for it.
			app.right_dock_ctrl.show_way("route", _route_points, "")
		_:
			app.right_dock_ctrl.show_way("", PackedVector2Array(), "")

func _way_click(gx: float, gy: float) -> void:
	if not bridge.way_append_point(gx, gy):
		return
	_way_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_path_preview(_way_points)
	_tool_options_way()
	_push_way_draft()

func _route_click(gx: float, gy: float) -> void:
	if not bridge.route_append_stop(gx, gy):
		return
	_route_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_path_preview(_route_points)
	_tool_options_route()
	_push_way_draft()

## Changing the way type mid-draft is an honest restart, not a silent
## reinterpretation of already-placed points: `way_begin` (Rust) always
## replaces the draft with a fresh, empty one -- unlike the reference, which
## read `civWayType` fresh only at commit time and so could change type
## freely mid-draw. This port's `WayDraft.way_type` is fixed for the
## draft's whole lifetime, so there is nothing to do here but begin again.
func _set_way_type(new_type: String) -> void:
	_way_type = new_type
	if _active_infra_tool == "way":
		bridge.way_begin(_way_type)
		_way_points = PackedVector2Array()
		app.viewport.tool_overlay.set_path_preview(_way_points)
	_tool_options_way()
	_push_way_draft()

## Shared by the Commit button, Escape, and switching away to another tool.
## `way_commit()` itself already matches `_civCommitWay`'s own "no-op under
## two waypoints, but the draft is gone either way" -- so this never needs
## to branch on point count before calling it.
func _commit_way() -> void:
	var idx := bridge.way_commit()
	_way_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_way_points)
	if idx >= 0:
		## Both of these are new with `GUI_GAP_REGISTER.md` IN-02's fix:
		## `get_roads()`/`get_sea_routes()` now include committed manual
		## ways, so re-reading them really does show the way just drawn.
		_refresh_map_ways()
		_refresh_manual_ways()
		app.set_status("hint",
			"Way #%d committed -- drawn on the map and listed under Civilization ▸ Routes & ways ▸ Hand-drawn." % idx,
			"text_ghost")
	if _active_infra_tool == "way":
		_tool_options_way()
	_push_way_draft()


## Repaints the map's civ layer so a way committed a moment ago appears
## without waiting for the next regenerate.
##
## Delegates to `CivilizationWorkspace._refresh_civ_data()` -- the shared,
## camera-preserving repaint whose own doc explains why this must NOT be
## `ViewportHost.refresh()` (that one calls `reset_view()`, which would snap
## the camera every time the user committed a way). This class is always
## composed as that workspace's child since the 2026-08-20 domain merge
## (`civilization_workspace.gd`'s `add_child(_infra)`), so `get_parent()` is
## it; the cast is guarded rather than asserted so a future recomposition
## costs a missing repaint, never a crash mid-commit.
func _refresh_map_ways() -> void:
	var civ := get_parent() as CivilizationWorkspace
	if civ != null:
		civ._refresh_civ_data()

func _discard_way() -> void:
	bridge.way_discard()
	_way_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_way_points)
	if _active_infra_tool == "way":
		_tool_options_way()
	_push_way_draft()

func _commit_route() -> void:
	var idx := bridge.route_commit()
	_route_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_route_points)
	if idx >= 0:
		## Both of these are new with `GUI_GAP_REGISTER.md` IN-09's fix. The
		## route really was committed before it (`route_count` incremented,
		## `route_get` returned a full solved path) -- nothing on the GDScript
		## side ever read either call back, so a 410 km route landed nowhere
		## the user could see. Exactly IN-02's failure mode one list over.
		_refresh_map_routes()
		_refresh_manual_routes()
		var r := bridge.route_get(idx)
		app.set_status("hint",
			"Route #%d committed -- %.0f km, drawn on the map and listed under Civilization ▸ Routes & ways ▸ Hand-drawn." % [idx, float(r.get("km", 0.0))],
			"text_ghost")
	if _active_infra_tool == "route":
		_tool_options_route()
	_push_way_draft()


## The route layer's own repaint. Unlike `_refresh_map_ways` this does NOT go
## through `CivilizationWorkspace._refresh_civ_data()`: routes are not part of
## `get_roads()`/`get_sea_routes()`, so `set_civ_data` neither carries nor
## clears them (see `map_overlay.gd`'s `_manual_routes` doc). `ViewportHost`
## owns the `route_count`/`route_get` loop, for the same reason it owns
## `refresh_annotations`' own two list calls.
func _refresh_map_routes() -> void:
	app.viewport.overlay.set_manual_routes(app.viewport.manual_routes())

func _discard_route() -> void:
	bridge.route_discard()
	_route_points = PackedVector2Array()
	app.viewport.tool_overlay.set_path_preview(_route_points)
	if _active_infra_tool == "route":
		_tool_options_route()
	_push_way_draft()

func _tool_options_label(row: HBoxContainer, text: String) -> void:
	row.add_child(DccTheme.mono_label(text, "accent", DccTheme.FS_SMALL, 2, true))

## §4.5.4's Way options row: `INFRA · WAY` · way type · (no routing-mode
## dropdown -- `infra_tools_bridge`'s own module doc: "nothing to build a
## 'freehand' or distinct 'snap' routing mode out of"; snap-to-place/way is
## real but applied automatically, not a toggle here) · a live point count ·
## ✓ Commit · Discard. No ↶ ↷: there is no per-waypoint undo in the engine
## (`InfraTools` only ever discards the whole draft), and building one here
## via discard-and-replay would be new, untested interaction invented for
## this task alone -- left undone and disclosed, not faked.
func _tool_options_way() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "INFRA · WAY")
		DccWidgets.choice(row, "Type", WAY_DRAW_TYPE_LABELS, WAY_DRAW_TYPES.find(_way_type),
			func(i: int): _set_way_type(WAY_DRAW_TYPES[i]))
		row.add_child(DccTheme.mono_label(
			"%d waypoint%s" % [_way_points.size(), "" if _way_points.size() == 1 else "s"],
			"text_ghost", DccTheme.FS_SMALL))
		row.add_child(DccTheme.spacer())
		var commit_btn := DccWidgets.action(row, "✓ Commit", _commit_way, true)
		commit_btn.disabled = _way_points.size() < 2
		var discard_btn := DccWidgets.action(row, "Discard", _discard_way)
		discard_btn.disabled = _way_points.is_empty()
	)

## §4.5.4's Route options row, minus the vessel/party reference (Logistics'
## own stub in this file explains why: the journey planner exports nothing
## past the Rust crate boundary, so there is no vessel/party list to read
## here) and minus a mode dropdown (see `_on_infra_tool_armed`'s "route"
## branch -- the reference never exposed one either).
func _tool_options_route() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "INFRA · ROUTE")
		row.add_child(DccTheme.mono_label(
			"%d stop%s" % [_route_points.size(), "" if _route_points.size() == 1 else "s"],
			"text_ghost", DccTheme.FS_SMALL))
		row.add_child(DccTheme.spacer())
		var commit_btn := DccWidgets.action(row, "✓ Commit", _commit_route, true)
		commit_btn.disabled = _route_points.size() < 2
		var discard_btn := DccWidgets.action(row, "Discard", _discard_route)
		discard_btn.disabled = _route_points.is_empty()
	)

## Reclaims the options bar once Way/Route hands it back to plain Inspect.
## `app.gd`'s own domain-switch-driven default (`_on_workspace_changed`)
## still says "the §4.5 tool palette to arm them is not built yet" -- stale
## the moment this file ships, but out of scope here (`app.gd` isn't ours to
## touch this pass); this is the in-session correction for as long as the
## user stays on this domain without switching away and back.
func _tool_options_infra_idle() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "INFRA · INSPECT")
		row.add_child(DccTheme.label("Way and Route tools are armed from the TOOLS block above.", "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
	)

# -- Roads --------------------------------------------------------------

## Split build/fill: `_build_*` runs once and claims the category body,
## `_fill_*` is what `rebuild_readouts()` can re-run against a new world.
func _build_roads() -> void:
	_roads_body = DccWidgets.category(self, "Roads", categories, true)
	_fill_roads(_roads_body)

# -- v3 entry points ----------------------------------------------------------

## v3 CIVIL ▸ ROUTES & WAYS: `§ Ways` (the permanent network, generated and
## hand-drawn) then the ports and sea lanes that terminate it. `§ Routes` --
## a planned traversal *over* that network -- is the Route tool in the TOOLS
## block plus the "Routes committed this session" list `_build_manual_ways`
## already draws, so both sit in this one category exactly as v3 draws them.
func build_ways_into(parent: Control) -> void:
	_dock_hosted = true
	_roads_body = VBoxContainer.new()
	_roads_body.add_theme_constant_override("separation", 0)
	_roads_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_roads_body)
	_fill_roads(_roads_body)

	_ports_body = VBoxContainer.new()
	_ports_body.add_theme_constant_override("separation", 0)
	_ports_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_ports_body)
	_fill_ports(_ports_body)

	DccWidgets.note(DccWidgets.section(parent, "Where the style lives"),
		"Line width, colour, casing and dashes per way class are Cartography ▸ "
		+ "Roads & routes -- v3's own rule: \"a way exists in the world; a route "
		+ "is an intention over it. Both are CIVIL. Their colour and line width "
		+ "are CARTO.\" Nothing in that dock changes where a road runs.")

## v3 CIVIL ▸ TRAVEL: journeys, the planner, and the travel library.
func build_travel_into(parent: Control) -> void:
	_dock_hosted = true
	_logistics_body = VBoxContainer.new()
	_logistics_body.add_theme_constant_override("separation", 0)
	_logistics_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_logistics_body)
	_fill_logistics(_logistics_body)

	var lib := DccWidgets.section(parent, "Travel library")
	var b := DccWidgets.action(lib, "Travel library… (⇧L)", func(): app.open_travel_library())
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Animals, mounts, vehicles, vessels and saved party set-ups -- the reference tables the planner draws its speeds and loads from."

## v3 CIVIL ▸ TRADE. Four rows v3 asks for, all four now backed
## (`GUI_GAP_REGISTER.md` **IN-13**, built 2026-08-25).
##
## The order is the disclosure ladder the design settled on: the world, then
## the good, then the pair, then the place. `§ Balance` is the surplus/deficit
## verdict that has always been here and says *what*; everything below it is
## the match, and says *who*.
func build_trade_into(parent: Control) -> void:
	_dock_hosted = true
	_trade_body = VBoxContainer.new()
	_trade_body.add_theme_constant_override("separation", 0)
	_trade_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_trade_body)
	_fill_trade(_trade_body)

	_build_flows(parent)

	DccWidgets.note(DccWidgets.section(parent, "Not built"),
		"Prices · tariffs · caravans as entities · change over time  ·  needs a decision\n"
		+ "None of the four is derivable from anything the civ layer holds, and "
		+ "each needs a decision about what a currency is here before it could be "
		+ "anything but a fabricated number (GUI_GAP_REGISTER.md IN-13, narrowed "
		+ "to exactly these).\n"
		+ "The flows above are a reading of the world as it stands, and stop "
		+ "there.")

## `GUI_GAP_REGISTER.md` **IN-13** -- trade flows as a routed quantity.
##
## **Behind a button, and that is the design.** The match walks every
## settlement pair against the way network and the coastline; refilling it on
## every dock rebuild would re-run it each time somebody renames a place. Same
## shape as CIVIL ▸ Territories ▸ Borders & influence, and for the same
## reason.
##
## `TradeStore` holds the result for the place editor and the map overlay, so
## one press answers all three surfaces; `app.gd` drops it on any world
## change.
func _build_flows(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Flows")
	_flows_run = DccWidgets.action(sec, "Match trade flows", _match_trade_flows)
	var run := _flows_run
	run.disabled = not bridge.has_world
	run.tooltip_text = ("Matches every settlement's surplus against every deficit it can "
		+ "actually reach -- the reference's own food-shed machinery (_civFoodShed's supplier "
		+ "enumeration, _civRoadConnected's union-find over the way network, _civGoodReach's "
		+ "bulk-needs-water rule) run over all fifteen resources instead of one. Nothing is "
		+ "retained in the engine: the match is built, read and dropped, and the reading "
		+ "reports what it cost.")
	_flows_body = VBoxContainer.new()
	_flows_body.add_theme_constant_override("separation", 4)
	_flows_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(_flows_body)
	_fill_flows()

func _fill_flows() -> void:
	if _flows_body == null or not is_instance_valid(_flows_body):
		return
	var d := TradeStore.last()
	if d.is_empty():
		DccWidgets.note(_flows_body,
			"Not matched yet. The match reads the settlement list, the way network and the "
			+ "coastline; on a few hundred settlements it takes a fraction of a second and "
			+ "keeps nothing afterwards. A settlement's own ledger (place editor ▸ Trade) and "
			+ "the map's way-load overlay read the same match.")
		return

	var flows := int(d.get("flow_count", 0))
	var importing := int(d.get("importing", 0))
	var supplied := int(d.get("supplied", 0))
	DccWidgets.note(_flows_body,
		("%s flows over %d of 15 goods, in %d ms. %d of %d settlements that need something "
		+ "get it; %d have a need nothing in reach can fill.") % [
			FactionRosterWindow._thousands(flows), int(d.get("goods_moving", 0)),
			int(d.get("elapsed_ms", 0)), supplied, importing,
			max(0, importing - supplied)])
	DccWidgets.note(_flows_body,
		"Carried by land %d%% · river %d%% · sea %d%% -- by volume, not by count: one sea lane "
		% [int(round(100.0 * float(d.get("land_share", 0.0)))),
			int(round(100.0 * float(d.get("river_share", 0.0)))),
			int(round(100.0 * float(d.get("sea_share", 0.0))))]
		+ "moving a city's demand is not one flow's worth of trade.")

	_fill_flows_goods(d)
	_fill_flows_partners(d)
	_fill_flows_unmet(d)
	_fill_flows_ways(d)

	DccWidgets.note(_flows_body,
		("Built on demand and dropped: %.2f MB at its peak, held for the length of the call "
		+ "and retained nowhere. CivData gained no field and nothing here is saved.")
		% (float(d.get("transient_bytes", 0)) / 1048576.0))

## § BY GOOD -- what moves, and how far it gets.
func _fill_flows_goods(d: Dictionary) -> void:
	var rows: Array = d.get("goods", [])
	var g := DccWidgets.group(_flows_body, "By good")
	if rows.is_empty():
		DccWidgets.note(g,
			"Nothing moves. Every settlement's surplus is a good no reachable settlement is "
			+ "short of -- which is a real world, not an error.")
		return
	var sorted := rows.duplicate()
	sorted.sort_custom(func(a, b): return float(a.get("volume", 0.0)) > float(b.get("volume", 0.0)))
	for r in sorted:
		var row: Dictionary = r
		DccWidgets.note(g, "%s -- %d exporters → %d importers, %s carried, mostly %s (%s)" % [
			String(row.get("name", "?")), int(row.get("exporters", 0)),
			int(row.get("importers", 0)),
			FactionRosterWindow._thousands(int(round(float(row.get("volume", 0.0))))),
			String(row.get("dominant_mode", "land")),
			"bulk" if bool(row.get("bulk", false)) else "luxury"])
	DccWidgets.note(g,
		"Reach is the reference's own rule: a luxury travels anywhere, a bulk good needs water "
		+ "-- sea lane long, river regional, neither local. The same surplus is a regional "
		+ "export from a river port and a purely local one from an inland hamlet.")

## § PARTNERS -- who, specifically.
func _fill_flows_partners(d: Dictionary) -> void:
	var rows: Array = d.get("flows", [])
	var g := DccWidgets.group(_flows_body, "Busiest partners")
	if rows.is_empty():
		DccWidgets.note(g, "No pair trades.")
		return
	var shown: int = min(12, rows.size())
	for i in range(shown):
		var row: Dictionary = rows[i]
		var from_i := int(row.get("from", -1))
		var b := DccWidgets.action(g, "%s → %s -- %s, %s, %s %d km" % [
			String(row.get("from_name", "?")), String(row.get("to_name", "?")),
			String(row.get("good", "?")),
			FactionRosterWindow._thousands(int(round(float(row.get("volume", 0.0))))),
			String(row.get("mode", "land")), int(round(float(row.get("distance_km", 0.0))))],
			func():
				if from_i >= 0:
					app.right_dock_ctrl.on_settlement_selected(
						bridge.settlements()[from_i], from_i))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = ("%s reach; %d%% of the exporter's scale survives the carriage. "
			+ "Opens the exporter in the right dock.") % [
				String(row.get("reach", "?")),
				int(round(100.0 * float(row.get("deliverable", 0.0))))]
	if rows.size() > shown:
		DccWidgets.note(g, "%s more, biggest first."
			% FactionRosterWindow._thousands(rows.size() - shown))
	if int(d.get("flow_count", 0)) > rows.size():
		DccWidgets.note(g,
			"%s flows matched in total; the list is capped so a very large world cannot hand "
			% FactionRosterWindow._thousands(int(d.get("flow_count", 0)))
			+ "the shell a hundred-thousand-row array. Every total above counts all of them.")

## § UNSUPPLIED -- a need, and nothing in reach.
func _fill_flows_unmet(d: Dictionary) -> void:
	var rows: Array = d.get("unmet", [])
	if rows.is_empty():
		return
	var g := DccWidgets.group(_flows_body, "Needs nothing can reach")
	var shown: int = min(10, rows.size())
	for i in range(shown):
		var row: Dictionary = rows[i]
		var goods: PackedStringArray = row.get("goods", PackedStringArray())
		DccWidgets.note(g, "%s -- %s (%s)" % [
			String(row.get("name", "?")), ", ".join(goods),
			"no exporter in reach" if bool(row.get("exporter_exists", false))
				else "nobody in the world exports it"])
	if rows.size() > shown:
		DccWidgets.note(g, "%d more." % (rows.size() - shown))
	DccWidgets.note(g,
		"The reference's own distinction, generalised past food: an unmet need with no viable "
		+ "supply is not an import relationship. It is a dependency the world cannot carry.")

## § WAY LOAD -- what the network is actually carrying.
func _fill_flows_ways(d: Dictionary) -> void:
	var rows: Array = d.get("ways", [])
	var g := DccWidgets.group(_flows_body, "Way load")
	if rows.is_empty():
		DccWidgets.note(g,
			"No way carries anything: every matched flow is either short enough to need no road "
			+ "at all (under 50 km, the reference's own local supply radius) or seaborne.")
	else:
		for r in rows:
			var row: Dictionary = r
			DccWidgets.note(g, "%s -- %s" % [
				String(row.get("name", "?")),
				FactionRosterWindow._thousands(int(round(float(row.get("load", 0.0)))))])
		DccWidgets.note(g, "%d of %d ways carry nothing."
			% [int(d.get("idle_ways", 0)), int(d.get("way_count", 0))])
	DccWidgets.note(g,
		"Drawn on the map as way thickness -- Cartography ▸ Roads & routes ▸ Trade load. Width "
		+ "and not colour, because a way's colour is already its type.")

func _match_trade_flows() -> void:
	if _flows_body == null or not is_instance_valid(_flows_body):
		return
	var d := TradeStore.refresh(bridge)
	_clear_body(_flows_body)
	if d.is_empty():
		DccWidgets.note(_flows_body,
			"Nothing to match: this world carries no civilisation layer (which is every loaded "
			+ "save) or has no settlements.")
		return
	_fill_flows()
	## The map draws the same match as way thickness, so it is handed the
	## per-way volumes here -- one owner of the computation, three readers of
	## it (this dock, the place editor's ledger, and the overlay).
	if app != null and app.viewport != null and app.viewport.overlay != null:
		app.viewport.overlay.set_trade_load(d.get("way_load", PackedFloat32Array()))

## The Rivers category left CIVIL for WORLD ▸ Hydrology, which is where v3
## puts the river network. Its one honest finding travels with it -- CIVIL had
## nothing to add to it beyond a heading.
##
## **`static`, and called from `world_workspace.gd`.** The v3 re-parenting pass
## wrote this function for exactly that and then never wired it up, so between
## 2026-08-24 and 2026-08-25 the IN-01 disclosure existed in the source and
## appeared nowhere in the app -- CIVIL had stopped drawing it and WORLD had
## never started. Found by grepping for callers rather than by a user hitting
## the empty category. One owner of the text, two possible hosts, which is the
## same discipline `build_*_into()` above uses for the controls.
##
## **Corrected twice, and the second correction is the one to read.**
##
## 2026-09-01: this note used to say the only river-derived output crossing the
## boundary was baked into `build_color_texture()`'s raster. Not true then --
## Strahler order crosses as data in three places (the `strahler` analysis
## raster, the `river_order` reading on `explain_settlement()`, and
## `measure_section()`'s crossings, which label a river "River · order 3"
## precisely because there is no name to give it), and discharge and drainage
## cross per cell too (Sample ▸ Drainage).
##
## 2026-09-03: the *replacement* was false as well, and had been for longer. It
## said no river crosses as an ENTITY and that there is no `get_rivers()`.
## There is: `WorldGen::get_rivers(min_order)` over
## `cartalith_hydrology::river_entities` returns one record per traced run,
## `WorldGen::river_at()` picks one from a click, and `right_dock.gd`'s River
## context -- the file this note cited as *agreeing* with it -- had already
## corrected itself in its own source, saying "every clause of it is now
## false". Landed under `OUTSTANDING_WORK.md` §2.2; this note did not follow.
static func rivers_note() -> String:
	return ("Rivers cross as ENTITIES. WorldGen.get_rivers(min_order) returns one record "
		+ "per traced channel run -- its polyline, Strahler order, routed length in km, "
		+ "discharge at the head and at the mouth, catchment_km2, tributary count, source "
		+ "and mouth elevation, drop, and drawn channel width -- and river_at() picks one "
		+ "from a map click, which is what the right dock's River context is wired to "
		+ "(arm Inspect, click a river). Per-cell readings cross as well: Strahler order "
		+ "as an analysis field and on a settlement's own explanation, discharge and "
		+ "drainage in Sample, and a labelled crossing on every measured section. Of v3's "
		+ "four per-reach rows, discharge, catchment and tributaries are real readings and "
		+ "navigability is not: the only navigability test in the engine is landmark.rs's "
		+ "trade-depot flow threshold, which rates a bank rather than a run. A river also "
		+ "has no NAME -- cartalith-civ's naming::FeatureKind offers Continent, Province, "
		+ "Bay, MountainRange and Lake and no river form -- and a world opened from a "
		+ "project archive has no rivers to select at all, because SAVEFILE_COMPAT.md "
		+ "stores no channel topology, so its rivers exist only in the baked raster.")

func _fill_roads(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Network")
	var roads := bridge.roads()
	if roads.is_empty():
		DccWidgets.note(sec, "No roads -- generate a world first (World ▸ Generate).")
	else:
		## `bridge.roads()` now carries hand-drawn ways alongside the
		## generated tiers (`get_roads()`, IN-02), so the tier tally below
		## no longer sums to `roads.size()` on its own -- WAY_TYPE_ORDER is
		## the generated vocabulary only. Counting the manual ones out
		## explicitly keeps the sentence arithmetically true rather than
		## quietly short by however many the user has drawn.
		var counts := {}
		var manual := 0
		for r in roads:
			if (r as Dictionary).get("manual", false):
				manual += 1
				continue
			var t := String((r as Dictionary).get("way_type", "road"))
			counts[t] = int(counts.get(t, 0)) + 1
		var parts: Array[String] = []
		for t in WAY_TYPE_ORDER:
			if counts.has(t):
				parts.append("%d %s" % [counts[t], t])
		if manual > 0:
			parts.append("%d hand-drawn" % manual)
		DccWidgets.note(sec, "%d ways -- %s." % [roads.size(), ", ".join(parts)])

		var longest := DccWidgets.group(sec, "Longest, by point count")
		var ranked := roads.duplicate()
		ranked.sort_custom(func(a, b): return (a as Dictionary).points.size() > (b as Dictionary).points.size())
		for i in range(mini(6, ranked.size())):
			_route_row(longest, ranked[i], "road")

	_build_routes_teaser(parent)
	_build_manual_ways(parent)
	_build_road_gaps(parent)


## `04-left-dock.md` §6c's own `ROUTES` block: one compact, click-to-plan row
## per committed route, directly under the Network group above -- the spec's
## own `WAYS · {count}` list. Deliberately not the same widget as `Routes
## committed this session` below (`_manual_routes_list`, select/rename/
## delete) -- that one is this port's own richer addition for editing a
## route; this one is the spec's own simpler shape, whose one action is
## opening the Journey Planner. Both read the same `bridge.route_count()`/
## `route_get()`, the same "filtered view, not a second store" reasoning
## `_build_manual_ways()`'s own doc comment already gives for Hand-drawn vs
## Network below.
##
## `<n> stages` from the spec's own mockup row is not drawn here: a stage is
## `jp_compute`'s own output and needs a party form, which is exactly the
## second computation surface `_refresh_manual_routes()`'s doc comment
## (below) already reasoned against for this same list. `route_get()` itself
## carries no stage count either way -- `lib.rs`'s own doc for the call
## lists only `points`/`brks`/`km`/`mode`/`unreachable_legs`/`name`. `km` is
## shown instead: real, exact, and already this file's own convention for a
## route/way row (`_route_row`, bottom of this file).
func _build_routes_teaser(parent: Control) -> void:
	_routes_teaser_section = DccWidgets.section(parent, "Routes")
	_refresh_routes_teaser()

## Clear-and-refill, the same shape and the same reason as
## `_refresh_manual_routes()` just below -- and called FROM it, so every
## trigger that already refreshes the editable routes list (a commit, a
## delete, `rebuild_readouts()`) keeps this teaser list in step with it too,
## rather than needing its own copy of the same three call sites.
func _refresh_routes_teaser() -> void:
	if _routes_teaser_section == null or not is_instance_valid(_routes_teaser_section):
		return
	for c in _routes_teaser_section.get_children():
		c.queue_free()
	var n := bridge.route_count()
	if n <= 0:
		DccWidgets.note(_routes_teaser_section,
			"None yet -- arm Route in the TOOLS block above, click two or more " +
			"stops, then ✓ Commit.")
	else:
		var settlements := bridge.settlements()
		for i in n:
			var r := bridge.route_get(i)
			if not r.is_empty():
				_routes_teaser_row(_routes_teaser_section, i, r, settlements)
	## §6c's own section footnote, quoted verbatim.
	DccWidgets.note(_routes_teaser_section,
		"a way is durable geometry others route over · a route is a journey along existing geometry — two tools, two records")

## One row: glyph, name, the nearest settlement at each end, length -- click
## opens the Journey Planner. Reuses `app.open_journey_planner()`, the SAME
## call `_fill_logistics()` below and `Data ▸ Journey planner… ⇧J`
## (`menus.gd`) already make -- not a second way to open the same window.
func _routes_teaser_row(parent: Control, i: int, r: Dictionary, settlements: Array) -> void:
	var name := String(r.get("name", ""))
	if name.is_empty():
		name = "Journey %d" % (i + 1)
	var label_text := "➔ %s" % name
	var pts: PackedVector2Array = r.get("points", PackedVector2Array())
	if pts.size() > 0:
		var origin := _nearest_settlement_name(pts[0], settlements)
		var dest := _nearest_settlement_name(pts[pts.size() - 1], settlements)
		if origin != "" and dest != "":
			label_text += " · %s → %s" % [origin, dest]
	label_text += " -- %d km" % int(round(float(r.get("km", 0.0))))
	var b := DccWidgets.action(parent, label_text, func(): app.open_journey_planner())
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = ("Opens the Journey Planner. It opens to its own Journeys list -- " +
		"usually route #1 or the most recently saved journey, not necessarily this " +
		"one; the dock has no per-route preselect hook into journey_planner_view.gd.")

## The nearest settlement to a route endpoint, by straight Euclidean
## distance -- valid because `bridge.settlements()`'s `x`/`y` (grid-cell
## index, `get_settlements()`'s own doc comment) and `route_get()`'s
## `points` (continuous coordinates) share one coordinate frame:
## `map_overlay.gd`'s own `_cell_to_screen`/`_point_to_screen` both divide by
## the identical `_gw`/`_gh`, the one confirming the other. `route_get()`
## carries no origin/destination of its own, so this is a presentation-only
## label, not a stored fact -- always the CLOSEST settlement, however far,
## not a snap-radius match to "the settlement this route was drawn from"
## (`civ_snap_radius` is Rust-internal, not exposed to GDScript).
func _nearest_settlement_name(pt: Vector2, settlements: Array) -> String:
	var best_name := ""
	var best_d2 := INF
	for s in settlements:
		var d: Dictionary = s
		var dx := float(d.get("x", 0)) - pt.x
		var dy := float(d.get("y", 0)) - pt.y
		var d2 := dx * dx + dy * dy
		if d2 < best_d2:
			best_d2 = d2
			best_name = String(d.get("name", ""))
	return best_name


## Every way the Way tool has committed this session (`GUI_GAP_REGISTER.md`
## IN-02's "or a list" half).
##
## Deliberately a filtered view of the SAME two getters the Network group
## above reads, not a second store: since `get_roads()`/`get_sea_routes()`
## append `InfraTools::ways` to the generated network (tagged `manual`),
## "the user's own ways" is a predicate over one list rather than a rival
## inventory that could drift out of step with what the map draws. That is
## also the reference's arrangement -- `#civWayList` is one list holding
## generated and hand-drawn ways together, told apart by a per-row type
## icon. This group exists because the Network group above shows only a
## six-row "longest" ranking, in which a freshly-drawn 12 km way would
## essentially never appear.
##
## Repopulated in place by `_refresh_manual_ways` rather than rebuilt:
## `Workspace` has no rebuild hook (`_build` runs once, from `setup`), and
## rebuilding the dock on every commit would collapse the accordion the
## user is working inside.
func _build_manual_ways(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Hand-drawn")
	_manual_list = DccWidgets.group(sec, "Committed this session")
	_refresh_manual_ways()
	## Routes get their own group rather than joining the ways above: a way is
	## a piece of network the next route can be solved over, a route is a
	## solved path across it. Merging them would make "committed this session"
	## mean two different kinds of thing under one heading.
	_manual_routes_list = DccWidgets.group(sec, "Routes committed this session")
	_refresh_manual_routes()


## Clear-and-refill, deliberately, rather than appending only the newest
## row: a commit can also *renumber* nothing but can produce either a road
## or a sea lane depending on the way type, so re-reading both getters is
## both shorter than branching and immune to the two lists getting out of
## order. At the handful-of-ways scale a person draws by hand, rebuilding a
## few buttons costs nothing.
func _refresh_manual_ways() -> void:
	if _manual_list == null:
		return
	for c in _manual_list.get_children():
		c.queue_free()
	var rows := 0
	for r in bridge.roads():
		if (r as Dictionary).get("manual", false):
			_route_row(_manual_list, r, "road")
			rows += 1
	for r in bridge.sea_routes():
		if (r as Dictionary).get("manual", false):
			_route_row(_manual_list, r, "sea")
			rows += 1
	if rows == 0:
		DccWidgets.note(_manual_list,
			"None yet -- arm Way in the TOOLS block above, click two or more " +
			"waypoints, then ✓ Commit. Committed ways draw on the map with the " +
			"generated network and are routed over by the next way you draw.")


## The Route tool's own committed list -- same clear-and-refill shape as
## `_refresh_manual_ways` above, over `route_count`/`route_get` instead of the
## two network getters.
##
## Unlike every other list in this file these rows are *editable*, because
## since 2026-08-24 there is something to edit with: `route_set_name` and
## `route_delete` (`GUI_GAP_REGISTER.md` IN-09's second half). The row layout
## is the reference journey card's, minus its planner summary: select glyph ·
## name field · km · `×`, in that order (`_civRenderJourneyList`, reference
## line 17235). No planner summary here because that card only shows one for
## the *selected* row and computes it with `_jpPlan`, which is the Journey
## Planner's own screen in this shell (`journey_planner_view.gd`), not a
## left-dock row -- duplicating it here would mean two places computing a
## plan and disagreeing.
func _refresh_manual_routes() -> void:
	if _manual_routes_list == null:
		return
	## `04-left-dock.md` §6c's own ROUTES teaser reads the same two calls this
	## function makes below -- refreshed first and unconditionally, so the
	## early return three lines down (zero committed routes) cannot skip it
	## the way a call placed after the loop would. Kept in step here rather
	## than at each of this function's own three call sites (`rebuild_
	## readouts()`, `_commit_route()`, `_delete_route()`), so a fourth
	## trigger never has to remember to add a second refresh call beside
	## this one.
	_refresh_routes_teaser()
	for c in _manual_routes_list.get_children():
		c.queue_free()
	var n := bridge.route_count()
	if _selected_route >= n:
		_selected_route = -1
	if n <= 0:
		DccWidgets.note(_manual_routes_list,
			"None yet -- arm Route in the TOOLS block above, click two or more " +
			"stops, then ✓ Commit. A route is solved over the existing network " +
			"(mixed land and sea) and drawn on the map in amber, over the ways " +
			"it follows.")
		return
	for i in n:
		var r := bridge.route_get(i)
		if r.is_empty():
			continue
		_manual_route_row(i, r)


## One editable row. `i` is a *live* index into the engine's route list, so
## every callable below closes over it and the whole list is rebuilt after a
## delete -- a row holding a stale index would rename or delete its neighbour
## (`route_delete` renumbers; see its doc comment in `lib.rs`).
func _manual_route_row(i: int, r: Dictionary) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = 24

	## The reference marks selection by re-styling the whole card border; this
	## dock has no card, so the row carries an explicit glyph instead -- the
	## same substitution `DccWidgets.segment` already makes for a toggled
	## state in a flat row.
	var sel := DccWidgets.text_button(row, "●" if i == _selected_route else "○",
		func(): _select_route(-1 if i == _selected_route else i))
	sel.tooltip_text = "Select this route -- drawn brighter and thicker on the map (the reference's own sel stroke, drawCivLayer block 2b). Click again to deselect."

	## `text_changed`, not `text_submitted`: the reference renames on `oninput`,
	## i.e. per keystroke, and no row rebuild happens here (which would steal
	## focus mid-word -- the same reasoning `civilization_workspace.gd`'s
	## `_settlement_name_field` records).
	var name_edit := LineEdit.new()
	name_edit.text = String(r.get("name", ""))
	name_edit.placeholder_text = "Journey %d" % (i + 1)
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.custom_minimum_size.x = 90
	name_edit.tooltip_text = "route_set_name. Blank restores the \"Journey %d\" fallback -- that label is computed here, never stored, so it follows the row after a delete renumbers it." % (i + 1)
	DccWidgets.well(name_edit, 6, 2)
	name_edit.text_changed.connect(func(t: String): bridge.route_set_name(i, t))
	row.add_child(name_edit)

	var meta := "%d km" % int(round(float(r.get("km", 0.0))))
	var unreachable := int(r.get("unreachable_legs", 0))
	if unreachable > 0:
		meta += " · %d straight-lined" % unreachable
	var meta_lbl := DccTheme.mono_label(meta, "text_ghost", DccTheme.FS_TINY)
	meta_lbl.tooltip_text = "%s mode, %d path points." % [
		String(r.get("mode", "mixed")),
		(r.get("points", PackedVector2Array()) as PackedVector2Array).size()]
	row.add_child(meta_lbl)

	var del := DccWidgets.text_button(row, "×", func(): _delete_route(i))
	del.tooltip_text = "Delete this route (the reference's own per-row × , line 17250). Later routes renumber down by one, exactly as civJourneys.splice does."

	_manual_routes_list.add_child(row)


func _select_route(index: int) -> void:
	_selected_route = index
	app.viewport.overlay.set_selected_manual_route(index)
	_refresh_manual_routes()


## Deleting renumbers, so the selection is fixed up here rather than only
## cleared. **This diverges from the reference on purpose**: it clears the
## selection only when the index runs off the end (`if(_civSelectedJourneyIdx
## >=civJourneys.length) _civSelectedJourneyIdx=-1`), which silently moves the
## selection onto a *different* journey whenever a lower-indexed one is
## deleted. Following that here would highlight the wrong line on the map.
func _delete_route(index: int) -> void:
	if not bridge.route_delete(index):
		return
	if _selected_route == index:
		_selected_route = -1
	elif _selected_route > index:
		_selected_route -= 1
	_refresh_map_routes()
	app.viewport.overlay.set_selected_manual_route(_selected_route)
	_refresh_manual_routes()
	app.set_status("hint", "Route #%d deleted -- later routes renumbered." % index, "text_ghost")


## The reference's two whole-network road operations, **both wired
## 2026-09-02** (`LARGE_ITEM_RULINGS.md`'s civ-authoring ruling, stages 2 and
## 3 of 5).
##
## They sat disabled under "Not built" saying route generation "is part of
## compute_civilisation inside generate(); no civ_auto_routes #[func] runs it
## on its own", and that clearing had "nothing here that could honestly claim
## to clear both" generated and manual ways. Both statements were true and
## both are now false: `civ_auto_routes` rebuilds the network alone, and
## `civ_clear_ways` empties CivData's ways and sea lanes *and* InfraTools'
## manual ways and committed journeys in one press -- which is what the
## reference's own single handler does (`civWays=[]; civJourneys=[]`).
func _build_road_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Network")
	_gen_roads_btn = DccWidgets.action(sec, "Generate roads", _generate_roads)
	_gen_roads_btn.disabled = not bridge.has_world
	_gen_roads_btn.tooltip_text = ("The reference's #civAutoRoutesBtn. Rebuilds the whole route "
		+ "network over the settlements that exist right now: the hierarchical land topology, the "
		+ "smoothed and classified ways it becomes, and the port-to-port sea lanes. Settlements, "
		+ "territory, provinces and the timeline are left alone.\n\n"
		+ "Needs settlements to connect -- Auto-populate the world first (CIVIL ▸ Settlements). "
		+ "Seconds, not milliseconds, on the main thread: the road builder reads river order, "
		+ "biome and the water-body map, so it costs a full civilisation pass even though only "
		+ "the network is kept.\n\n"
		+ "Manual ways drawn with the Way tool are not touched, and the network prefers no "
		+ "particular route through them -- draw one and it stays.")
	_roads_note = DccWidgets.note(sec,
		"Drawing a way or a journey by hand stays available in the TOOLS block above; this is "
		+ "the whole-network pass.")
	_clear_ways_btn = DccWidgets.action(sec, "Clear ways & journeys", _clear_ways)
	_clear_ways_btn.disabled = not bridge.has_world
	_clear_ways_btn.tooltip_text = ("The reference's #civClearRoadsBtn. Empties both networks in "
		+ "one press: the generated ways and sea lanes, and the manual ways and committed "
		+ "journeys from this session. Settlements stay where they are.\n\n"
		+ "Not undoable -- Generate roads builds a new network rather than restoring this one. "
		+ "To remove a single journey instead, use the × on its row in Routes.")

## Stage 2 of the civ-authoring ruling's five. Same progress affordance as
## `civilization_workspace.gd`'s `_recompute_civ`, and for the same reason:
## a synchronous engine call with no progress signal, so relabel, disable, let
## two frames actually paint that, then block.
func _generate_roads() -> void:
	var b := _gen_roads_btn
	if b != null and is_instance_valid(b):
		b.text = "Generating…"
		b.disabled = true
		await get_tree().process_frame
		await get_tree().process_frame
	var r: Dictionary = bridge.civ_auto_routes()
	if b != null and is_instance_valid(b):
		b.disabled = false
		b.text = "Generate roads"
	var ok := bool(r.get("ok", false))
	var outcome := ""
	if not ok:
		outcome = "No roads generated. %s" % String(r.get("reason", "Unknown reason."))
	else:
		outcome = "Network rebuilt in %.1f s: %d ways over %d settlements." % [
			float(r.get("ms", 0.0)) / 1000.0, int(r.get("ways", 0)), int(r.get("settlements", 0))]
	if _roads_note != null and is_instance_valid(_roads_note):
		_roads_note.text = outcome
	app.set_status("hint", outcome, "text" if ok else "accent")
	_refresh_map_ways()

## Stage 3. Destructive and irreversible, so it confirms first -- reusing
## CIVIL's own `_confirm_destructive` rather than building a second dialog
## helper, since this class is always composed as that workspace's child
## (see `_refresh_map_ways` for the same guarded `get_parent()` cast and why
## a future recomposition must cost a feature, never a crash).
func _clear_ways() -> void:
	var civ := get_parent() as CivilizationWorkspace
	var n := bridge.roads().size()
	if civ == null:
		_clear_ways_now()
		return
	civ._confirm_destructive(
		"Clear all ways and journeys?",
		"Removes %d generated way%s plus every sea lane, every hand-drawn way and every "
			% [n, "" if n == 1 else "s"]
			+ "committed journey. Settlements are untouched.\n\nThis cannot be undone.",
		"Clear",
		n == 0 and bridge.route_count() == 0,
		func(): _clear_ways_now())

func _clear_ways_now() -> void:
	var r: Dictionary = bridge.civ_clear_ways()
	_selected_route = -1
	app.viewport.overlay.set_selected_manual_route(-1)
	var outcome := "Cleared %d way(s), %d sea lane(s), %d hand-drawn way(s) and %d journey(s)." % [
		int(r.get("ways", 0)), int(r.get("sea_routes", 0)),
		int(r.get("manual_ways", 0)), int(r.get("journeys", 0))]
	if _roads_note != null and is_instance_valid(_roads_note):
		_roads_note.text = outcome + " Generate roads builds a new network."
	app.set_status("hint", outcome, "text")
	_refresh_map_ways()
	_refresh_map_routes()
	_refresh_manual_routes()

# -- Ports ------------------------------------------------------------------

func _build_ports() -> void:
	_ports_body = DccWidgets.category(self, "Ports", categories)
	_fill_ports(_ports_body)

func _fill_ports(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Coastal settlements")
	var settlements := bridge.settlements()
	var coastal: Array = []
	for s in settlements:
		if (s as Dictionary).get("coastal", false):
			coastal.append(s)
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first.")
	elif coastal.is_empty():
		DccWidgets.note(sec, "No coastal settlements in this world.")
	else:
		var msg := "%d of %d settlements are coastal (get_settlements()'s own field) -- the " + \
			"engine has no separate \"port\" concept beyond that flag."
		DccWidgets.note(sec, msg % [coastal.size(), settlements.size()])
		var list := DccWidgets.group(sec, "Coastal settlements")
		for s in coastal:
			var d: Dictionary = s
			var l := DccTheme.label("%s -- %s" % [d.get("name", "?"), String(d.get("kind", "?")).capitalize()],
				"text", DccTheme.FS_SMALL)
			list.add_child(l)

	var sea := bridge.sea_routes()
	if not sea.is_empty():
		var lanes := DccWidgets.group(sec, "Sea lanes")
		for i in range(mini(6, sea.size())):
			_route_row(lanes, sea[i], "sea")

# -- Trade ------------------------------------------------------------------

func _build_trade() -> void:
	_trade_body = DccWidgets.category(self, "Trade", categories)
	_fill_trade(_trade_body)

func _fill_trade(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Flows")
	var settlements := bridge.settlements()
	var balances := bridge.trade_balances()
	if balances.is_empty():
		DccWidgets.note(sec, "No trade balances -- generate a world first.")
		return
	var trading := 0
	for t in balances:
		var d: Dictionary = t
		var ex: PackedStringArray = d.get("exports", PackedStringArray())
		var im: PackedStringArray = d.get("imports", PackedStringArray())
		if ex.size() > 0 or im.size() > 0:
			trading += 1
	DccWidgets.note(sec, "%d of %d settlements carry a trade relationship." % [trading, settlements.size()])
	DccWidgets.note(sec,
		"Same civ_resource_trade_balance data the Civilization workspace's Economy " +
		"category reads -- goods flow, not route assignment: nothing ties a trade " +
		"relationship to the road or sea lane that would carry it.")

# -- Logistics ----------------------------------------------------------

## `JOURNEY_PLANNER_SCOPE.md`'s own "Update (2026-08-19)" closed the engine
## boundary (`jp_options`/`jp_default_plan`/`jp_compute`, `route_count`/
## `route_get`) -- this file's own older doc comment above ("exports nothing
## past the Rust crate boundary") is now stale for Logistics specifically.
##
## **2026-08-19 redesign**: the party form and results panel no longer live
## in a popup window -- `DCC_SHELL_SPEC.md` §4.5.4's own addition makes
## Journey a third INFRA tool, arming a full in-shell takeover
## (`journey_planner_view.gd`). This button is one of the two real entry
## points that arm it (the other is `Data ▸ Journey planner… ⇧J`,
## `menus.gd`) -- the mockup's own "rail-foot slot" phrasing describes where
## the tool's *context readout* lives while armed (`app.gd`'s `set_rail_foot`,
## already wired), not a second clickable target: `DccShell.rail_foot` is a
## plain `Label` shared by every domain's context text, and making only
## INFRA's foot cell clickable would be a shared-base-class change for no
## capability this dock button doesn't already provide.
func _build_logistics() -> void:
	_logistics_body = DccWidgets.category(self, "Logistics", categories)
	_fill_logistics(_logistics_body)

## **`04-left-dock.md` §6d decision (2026-09-01).** v3's `Travel` category
## (the header text `_railfold_probe.gd`'s `EXPECTED["civilization"]` locks
## in -- the rail's own expandable node list uses the spec's own "Journey
## planner" label instead, `dcc_shell.gd`'s `RAIL_NODES`, a different file's
## naming for the same subject) is where §6d's own `JOURNEY PLANNER`
## category lands in this port.
##
## §6d's own body is a full accordion -- TRAVELER / SEASON / CARRIAGE /
## ROUTE / STOPS, five parameter groups plus a stage list, the pipeline's
## own disclosure pattern reused. **Not embedded here.** `_refresh_manual_
## routes()`'s doc comment (above, in the Ways & routes category) already
## reasoned through this for a single route's one-line summary; the same
## reasoning applies with more force to the full form. That form is
## `journey_planner_view.gd`'s own `_plan_values` / `_stage_overrides` /
## `_route_index` -- private fields that file exposes no accessor for, and
## every other entry point into it already lives with that: `right_dock.gd`'s
## Settlement "Logistics" and Measure "Plan a journey", `menus.gd`'s `Data ▸
## Journey planner… ⇧J`, and this file's own new `ROUTES` row above are all
## bare buttons, none of them a live preview. Building a second TRAVELER/
## SEASON/CARRIAGE/ROUTE/STOPS surface here would either bind to nothing (a
## form that edits no state) or reach into those private fields from outside
## the file that owns them -- two files mutating one form through no shared
## contract, which is a worse outcome than the window this already is.
##
## So this stays the standing choice: a thin, honest summary of what
## `bridge` alone can say (route count; the five group names, so a reader
## knows what is behind the door without this dock claiming to show live
## values it cannot), plus the one real, already-shared way in --
## `app.open_journey_planner()`.
func _fill_logistics(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Journey planning")
	var count := bridge.route_count()
	if count == 0:
		DccWidgets.note(sec,
			"No committed routes yet -- draw one with the Route tool above (arm " +
			"Route, click waypoints, ✓ Commit), then open the Journey Planner.")
	else:
		DccWidgets.note(sec, "%d committed route%s available to plan a journey along." %
			[count, "" if count == 1 else "s"])
	DccWidgets.note(sec,
		"The planner itself is TRAVELER, SEASON, CARRIAGE, ROUTE and STOPS -- group " +
		"size, pace and supplies; season, weather and rest days; carriage and mounts; " +
		"road quality and closures; and a per-stage override list. All five live in " +
		"the planner's own window, not copied here.")
	var g := DccWidgets.group(sec, "Journey Planner")
	var b := DccWidgets.action(g, "Open Journey Planner", func(): app.open_journey_planner(), true)
	b.tooltip_text = ("Arms the Journey tool and swaps to its own in-shell takeover " +
		"(journey_planner_view.gd) -- the same call Data ▸ Journey planner… ⇧J makes. " +
		"Opens to its own Journeys list: usually route #1 or the most recently saved " +
		"journey, not necessarily whichever route you were just looking at here.")

# -- Shared ---------------------------------------------------------------

## One clickable row for a `get_roads()`/`get_sea_routes()` entry.
##
## The empty-name fallback below is not defensive padding: a hand-drawn way
## is committed with `name: ""` (`civ_commit_way` sets `String::new()`, the
## reference's own `name:''` at line 26077), so every manual row would read
## as a blank button without it. The reference's own way list falls back the
## same way, to `'Way '+(ri+1)` -- an index, which this helper has no honest
## access to (its caller may be showing a filtered or ranked subset), so it
## names the source instead of inventing a number that would not match the
## engine's own commit index.
func _route_row(parent: Control, entry: Dictionary, kind: String) -> void:
	var label_text := String(entry.get("name", ""))
	if label_text.is_empty():
		label_text = "Hand-drawn way" if entry.get("manual", false) else "unnamed"
	if kind == "road":
		label_text += " (%s)" % String(entry.get("way_type", "road"))
	else:
		label_text += " (sea lane)"
	var km := float(entry.get("km", 0.0))
	if km > 0.0:
		label_text += " -- %d km" % int(round(km))
	var b := DccWidgets.action(parent, label_text, func(): app.right_dock_ctrl.show_route(entry, kind))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Open this route in the right dock."
