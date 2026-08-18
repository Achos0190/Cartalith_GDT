extends Workspace
class_name CivilizationWorkspace

## CIVIL domain (§3): settlements, population, economy, politics, culture.
##
## The engine backs four of the five as *readable* data (`get_settlements`,
## `get_provinces`, `get_trade_balances`). Culture has no GDExtension binding
## at all -- `cartalith-civ` computes culture profiles internally
## (`STATUS.md`'s "generation rules and culture profiles" milestone) but
## nothing exports them. It also backs placing a settlement and painting
## territory, which this dock has no surface for -- `STRANDED_TOOLS.md` rows
## 10 and 12.
##
## Rows here read only -- clicking a settlement or faction pins it into the
## right dock (`right_dock.gd`'s Settlement/Faction contexts) exactly like
## clicking it on the map does; nothing in this file writes to `bridge`.

const KIND_ORDER := ["capital", "city", "town", "village", "hamlet"]
const KIND_PLURAL := {
	"capital": "capitals", "city": "cities", "town": "towns",
	"village": "villages", "hamlet": "hamlets",
}

func _build() -> void:
	_build_settlements()
	_build_population()
	_build_economy()
	_build_politics()
	_build_culture()

# -- Settlements --------------------------------------------------------

func _build_settlements() -> void:
	var cat := DccWidgets.category(self, "Settlements", categories, true)
	var sec := DccWidgets.section(cat, "Roster")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first (World ▸ Generation Pipeline).")
		return

	var counts := {}
	for s in settlements:
		var kind := String((s as Dictionary).get("kind", "?"))
		counts[kind] = int(counts.get(kind, 0)) + 1
	var parts: Array[String] = []
	for kind in KIND_ORDER:
		if counts.has(kind):
			parts.append("%d %s" % [counts[kind], kind if int(counts[kind]) == 1 else KIND_PLURAL[kind]])
	DccWidgets.note(sec, "%d settlements -- %s." % [settlements.size(), ", ".join(parts)])

	var by_pop := DccWidgets.group(sec, "Largest, by population")
	var ranked: Array = []
	for i in range(settlements.size()):
		ranked.append({"index": i, "data": settlements[i]})
	ranked.sort_custom(func(a, b): return int(a.data.population) > int(b.data.population))
	for i in range(mini(8, ranked.size())):
		_settlement_row(by_pop, ranked[i].data, ranked[i].index)

func _settlement_row(parent: Control, data: Dictionary, index: int) -> void:
	var text := "%s -- %s, pop %d" % [data.get("name", "?"), String(data.get("kind", "?")).capitalize(), int(data.get("population", 0))]
	var b := DccWidgets.action(parent, text, func(): app.right_dock_ctrl.on_settlement_selected(data, index))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Pin this settlement in the right dock (same as clicking it on the map)."

# -- Population -----------------------------------------------------------

func _build_population() -> void:
	var cat := DccWidgets.category(self, "Population", categories)
	var sec := DccWidgets.section(cat, "Totals")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first.")
		return

	var total := 0
	var largest_name := ""
	var largest_pop := -1
	for s in settlements:
		var d: Dictionary = s
		var pop := int(d.get("population", 0))
		total += pop
		if pop > largest_pop:
			largest_pop = pop
			largest_name = String(d.get("name", "?"))
	DccWidgets.note(sec, "Total population %d across %d settlements. Largest: %s (%d)." % [
		total, settlements.size(), largest_name, largest_pop])
	DccWidgets.note(sec,
		"Per-settlement population is real (get_settlements()'s own field). Faction- or " +
		"province-level population aggregation has no binding -- see Politics below for " +
		"what get_provinces() does carry.")

# -- Economy ----------------------------------------------------------------

func _build_economy() -> void:
	var cat := DccWidgets.category(self, "Economy", categories)
	var sec := DccWidgets.section(cat, "Trade balance")
	var settlements := bridge.settlements()
	var balances := bridge.trade_balances()
	if balances.is_empty():
		DccWidgets.note(sec, "No trade balances -- generate a world first.")
		return

	var exports := {}
	var imports := {}
	var trading := 0
	for i in range(balances.size()):
		var t: Dictionary = balances[i]
		var ex: PackedStringArray = t.get("exports", PackedStringArray())
		var im: PackedStringArray = t.get("imports", PackedStringArray())
		if ex.size() > 0 or im.size() > 0:
			trading += 1
		for r in ex:
			exports[r] = int(exports.get(r, 0)) + 1
		for r in im:
			imports[r] = int(imports.get(r, 0)) + 1
	DccWidgets.note(sec, "%d of %d settlements carry a trade relationship (surplus or deficit)." % [
		trading, settlements.size()])
	DccWidgets.note(sec, "Most-exported: %s. Most-imported: %s." % [
		_top_key(exports), _top_key(imports)])
	DccWidgets.note(sec,
		"This is the hinterland term only (civ_resource_trade_balance) -- the full " +
		"faction-level aggregation (population, tax, the five-axis power heuristic) is " +
		"real future scope per ECONOMY_SCOPE.md, not yet computed.")

func _top_key(counts: Dictionary) -> String:
	if counts.is_empty():
		return "none"
	var best_key := ""
	var best_n := -1
	for k in counts:
		if int(counts[k]) > best_n:
			best_n = int(counts[k])
			best_key = String(k)
	return "%s (%d settlements)" % [best_key, best_n]

# -- Politics -----------------------------------------------------------

func _build_politics() -> void:
	var cat := DccWidgets.category(self, "Politics", categories)
	var sec := DccWidgets.section(cat, "Factions")
	var provinces := bridge.provinces()
	var settlements := bridge.settlements()
	if provinces.is_empty():
		DccWidgets.note(sec, "No provinces -- generate a world first.")
	else:
		var by_faction := {}
		for p in provinces:
			var d: Dictionary = p
			var f := int(d.get("faction", 0))
			if not by_faction.has(f):
				by_faction[f] = []
			by_faction[f].append(d)
		var factions: Array = by_faction.keys()
		factions.sort()
		var roster := DccWidgets.group(sec, "Roster, by province count")
		for f in factions:
			var provs: Array = by_faction[f]
			var cap_name := "—"
			if not provs.is_empty():
				var cap_idx := int((provs[0] as Dictionary).get("capital_settlement_index", -1))
				if cap_idx >= 0 and cap_idx < settlements.size():
					cap_name = String((settlements[cap_idx] as Dictionary).get("name", "—"))
			var text := "Faction %d -- %d provinces, capital %s" % [f, provs.size(), cap_name]
			var b := DccWidgets.action(roster, text, func(): app.right_dock_ctrl.show_faction(f))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = "Open this faction in the right dock."

	DccWidgets.note(sec,
		"Territory has an engine (cartalith-civ::tools.rs merge_territory_paint) and a " +
		"rendered overlay (build_territory_texture) but no paint surface here -- " +
		"STRANDED_TOOLS.md row 12. Placing a new settlement (civ_drop_place, " +
		"civ_pick_place_at) is likewise engine-only -- row 10.")

# -- Culture ----------------------------------------------------------------

func _build_culture() -> void:
	var cat := DccWidgets.category(self, "Culture", categories)
	var sec := DccWidgets.section(cat, "Profiles")
	DccWidgets.note(sec,
		"cartalith-civ generates culture profiles internally (generation rules + culture " +
		"profiles milestone, STATUS.md), but no GDExtension method exports them -- " +
		"get_settlements()/get_provinces() carry no culture field, and there is no " +
		"get_cultures() to read one from. Nothing here can be honest until that binding " +
		"lands.")
