extends Workspace
class_name InfrastructureWorkspace

## INFRA domain (§3): roads, rivers, ports, trade, logistics.
##
## Roads and sea routes are read from the engine today (`get_roads`,
## `get_sea_routes`) -- those two calls are this whole file's data source,
## per the task that built it. Rivers have no binding at all (no
## `get_rivers`, see `right_dock.gd`'s River context for the same finding).
## Ports here means coastal settlements, derived from `get_settlements()`'s
## real `coastal` field, not a separate concept the engine models. Logistics
## (the journey planner) is engine-complete per `JOURNEY_PLANNER_SCOPE.md`
## but, like culture, exports nothing past that crate boundary. Drawing a
## new route has an engine (`ManualWay`, `RouteContext`) and no surface --
## `STRANDED_TOOLS.md` row 11.
##
## Rows here read only; clicking a road or sea route pins it into the right
## dock (`right_dock.gd`'s Route context).

const WAY_TYPE_ORDER := ["highway", "regional", "road", "track"]

func _build() -> void:
	_build_roads()
	_build_rivers()
	_build_ports()
	_build_trade()
	_build_logistics()

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

func _build_logistics() -> void:
	var cat := DccWidgets.category(self, "Logistics", categories)
	var sec := DccWidgets.section(cat, "Journey planning")
	DccWidgets.note(sec,
		"The journey planner (JOURNEY_PLANNER_SCOPE.md) is engine-complete -- cost " +
		"rasters, stage breakdown, vessel legs -- but exports nothing past the Rust " +
		"crate boundary. No GDExtension method returns a journey, a cost trace or a " +
		"stage list, so nothing here can be honest until one does.")

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
