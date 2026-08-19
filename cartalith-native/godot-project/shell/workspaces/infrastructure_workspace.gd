extends Workspace
class_name InfrastructureWorkspace

## INFRA domain (§3): roads, rivers, ports, trade, logistics, and the Way/
## Route drawing tools (§4.5.4).
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
## `STRANDED_TOOLS.md` row 11) now has both an engine AND a surface -- the
## TOOLS block below arms Way/Route, click-chains a draft, and commits or
## discards it. What is still missing, confirmed by reading `get_roads`/
## `get_sea_routes` in `lib.rs`: **neither iterates `self.infra.ways`/
## `self.infra.routes`** (both only read `civ.ways`/`civ.sea_routes`, the
## auto-generated network) -- there is genuinely no getter yet that returns
## a committed manual way/route for display, matching `way_commit`'s own
## doc comment ("there is no getter for the manual-ways list itself yet,
## deliberately out of this milestone's exact scope"). So a committed way
## really is stored engine-side (and is real input to the next commit's own
## Dijkstra routing and to `snap_point`), but nothing here can make it
## *appear* on the map or in a list -- `bridge.roads()`/`bridge.sea_routes()`
## would not include it even if re-queried, so this file does not pretend
## to refresh the map after a commit. A status-bar line is the only
## acknowledgement a commit gets today.
##
## Rows here (Roads/Ports' network lists) read only; clicking a road or sea
## route pins it into the right dock (`right_dock.gd`'s Route context). The
## Way/Route *tools* below have no such inspector to pin into yet either,
## for the same getter gap.

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

func _build() -> void:
	_build_tools()
	_build_roads()
	_build_rivers()
	_build_ports()
	_build_trade()
	_build_logistics()

# -- Tools (§4.5.4: Way, Route) ------------------------------------------

## §4.5's TOOLS block: the three global tools (Inspect/Measure/Region,
## `GlobalTools.install`) plus this domain's own two. Built first, matching
## `render_workspace.gd`'s ordering (§4.5: "every left dock opens with a
## TOOLS block").
func _build_tools() -> void:
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
		app.set_status("hint",
			"Way #%d committed -- not shown on the map yet (no manual-way display getter; see this file's own doc comment)." % idx,
			"text_ghost")
	if _active_infra_tool == "way":
		_tool_options_way()

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
		app.set_status("hint",
			"Route #%d committed -- not shown on the map yet (no manual-route display getter; see this file's own doc comment)." % idx,
			"text_ghost")
	if _active_infra_tool == "route":
		_tool_options_route()

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

func _build_roads() -> void:
	var cat := DccWidgets.category(self, "Roads", categories, true)
	var sec := DccWidgets.section(cat, "Network")
	var roads := bridge.roads()
	if roads.is_empty():
		DccWidgets.note(sec, "No roads -- generate a world first (World ▸ Generation Pipeline).")
		return

	var counts := {}
	for r in roads:
		var t := String((r as Dictionary).get("way_type", "road"))
		counts[t] = int(counts.get(t, 0)) + 1
	var parts: Array[String] = []
	for t in WAY_TYPE_ORDER:
		if counts.has(t):
			parts.append("%d %s" % [counts[t], t])
	DccWidgets.note(sec, "%d ways -- %s." % [roads.size(), ", ".join(parts)])

	var longest := DccWidgets.group(sec, "Longest, by point count")
	var ranked := roads.duplicate()
	ranked.sort_custom(func(a, b): return (a as Dictionary).points.size() > (b as Dictionary).points.size())
	for i in range(mini(6, ranked.size())):
		_route_row(longest, ranked[i], "road")

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
	var cat := DccWidgets.category(self, "Ports", categories)
	var sec := DccWidgets.section(cat, "Coastal settlements")
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
	var cat := DccWidgets.category(self, "Trade", categories)
	var sec := DccWidgets.section(cat, "Flows")
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
## past the Rust crate boundary") is now stale for Logistics specifically; the
## party form and results panel live in `journey_planner_window.gd` (too big
## for a dock panel, same "AcceptDialog window" precedent as `world_data_
## window.gd`/`performance_window.gd`), opened from here.
func _build_logistics() -> void:
	var cat := DccWidgets.category(self, "Logistics", categories)
	var sec := DccWidgets.section(cat, "Journey planning")
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

func _route_row(parent: Control, entry: Dictionary, kind: String) -> void:
	var label_text := String(entry.get("name", "unnamed"))
	if kind == "road":
		label_text += " (%s)" % String(entry.get("way_type", "road"))
	else:
		label_text += " (sea lane)"
	var b := DccWidgets.action(parent, label_text, func(): app.right_dock_ctrl.show_route(entry, kind))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Open this route in the right dock."
