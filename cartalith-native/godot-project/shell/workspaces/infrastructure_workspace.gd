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
## source, per the task that built it. Rivers have no binding at all (no
## `get_rivers`, see `right_dock.gd`'s River context for the same finding).
## Ports here means coastal settlements, derived from `get_settlements()`'s
## real `coastal` field, not a separate concept the engine models. Logistics
## (the journey planner) is engine-complete per `JOURNEY_PLANNER_SCOPE.md`
## but, like culture, exports nothing past that crate boundary.
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

## The reference's `_civSelectedJourneyIdx`; `-1` for none. Lives here rather
## than in `map_overlay.gd` because the list is what changes it -- the overlay
## only draws it (`set_selected_manual_route`).
var _selected_route := -1

## The four data-backed categories' own body nodes, held so `rebuild_readouts()`
## can clear and refill exactly those and nothing else -- the same discipline
## `civilization_workspace.gd` uses for its own four. Rivers has no field
## because it has no data: `_build_rivers()` writes one fixed note about the
## missing `get_rivers()` binding, which a world does not change.
var _roads_body: Control
var _ports_body: Control
var _trade_body: Control
var _logistics_body: Control

func _build() -> void:
	_build_tools()
	_build_roads()
	_build_rivers()
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
		_:
			if _active_infra_tool != "":
				_active_infra_tool = ""
				app.viewport.tool_overlay.set_path_preview(PackedVector2Array())
				## Only reclaim the options bar for the plain "back to
				## Inspect" case. Measure/Region (`global_tools.gd`) set no
				## options-bar content of their own today, so guessing at
				## something to show while one of those is what just armed
				## would just be a different flavour of stale.
				if id == "inspect":
					_tool_options_infra_idle()

func _way_click(gx: float, gy: float) -> void:
	if not bridge.way_append_point(gx, gy):
		return
	_way_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_path_preview(_way_points)
	_tool_options_way()

func _route_click(gx: float, gy: float) -> void:
	if not bridge.route_append_stop(gx, gy):
		return
	_route_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_path_preview(_route_points)
	_tool_options_route()

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
			"Way #%d committed -- drawn on the map and listed under Roads ▸ Hand-drawn." % idx,
			"text_ghost")
	if _active_infra_tool == "way":
		_tool_options_way()


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
			"Route #%d committed -- %.0f km, drawn on the map and listed under Roads ▸ Hand-drawn." % [idx, float(r.get("km", 0.0))],
			"text_ghost")
	if _active_infra_tool == "route":
		_tool_options_route()


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

func _fill_roads(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Network")
	var roads := bridge.roads()
	if roads.is_empty():
		DccWidgets.note(sec, "No roads -- generate a world first (World ▸ Generation Pipeline).")
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

	_build_manual_ways(parent)
	_build_road_gaps(parent)


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


## The reference's two whole-network road operations. Same split as CIVIL's
## Settlements category: route generation runs inside `generate()`, so there
## is neither a "build the network now" button nor a partial teardown -- said
## rather than left as an unexplained absence, per `menus.gd`'s honesty rule.
func _build_road_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	var gen := DccWidgets.action(sec, "Generate roads", func(): pass)
	gen.disabled = true
	gen.tooltip_text = "The reference's #civAutoRoutesBtn. Route generation is part of compute_civilisation inside generate(); no civ_auto_routes #[func] runs it on its own, and there is no parameter for road density or which tiers get connected (params.rs carries no civ entries). Drawing a way by hand is the wired alternative -- the Way and Route tools in the TOOLS block above."
	var clear := DccWidgets.action(sec, "Clear ways & journeys", func(): pass)
	clear.disabled = true
	clear.tooltip_text = "The reference's #civClearRoadsBtn. CivData::ways/sea_routes are rebuilt wholesale by generate() with no clear #[func], and InfraTools::ways (where committed manual ways live -- readable since GUI_GAP_REGISTER.md IN-02, but read-only) has no clear either, so there is nothing here that could honestly claim to clear both. Journeys alone CAN now be cleared, one at a time, by the × on each row of Routes committed this session (route_delete, IN-09)."

# -- Rivers ---------------------------------------------------------------

func _build_rivers() -> void:
	var cat := DccWidgets.category(self, "Rivers", categories)
	var sec := DccWidgets.section(cat, "Hydrology")
	DccWidgets.note(sec,
		"No hydrological river entity is exposed to Godot -- cartalith-hydrology " +
		"computes river networks internally, but the only output that crosses the " +
		"GDExtension boundary is baked into build_color_texture()'s rendered raster. " +
		"There is no get_rivers() and no way to select one; see right_dock.gd's River " +
		"context for the same finding, field by field.")

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
	var g := DccWidgets.group(sec, "Journey Planner")
	DccWidgets.action(g, "Open Journey Planner", func(): app.open_journey_planner(), true)

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
