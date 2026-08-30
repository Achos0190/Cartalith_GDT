extends RefCounted
class_name PlaceSearch

## The world-entity search index `Edit ▸ Find on map…` (menus.gd ~532) was
## disabled for. That row's own tooltip said "the entity index and its
## pan-to-hit are both still owed" -- this file is the first half. The
## second half, wiring a hit to `ViewportHost.move_view_to()`, is
## `DccShell.open_find_on_map()`, owned by a different pass; this file only
## builds and searches the index.
##
## ## It is BUILT, not written down -- same discipline as `CommandIndex`
##
## Five real getters are read at `build()` time, every one already live on
## `EngineBridge`:
##
## - **Settlements** -- `EngineBridge.settlements()` (engine_bridge.gd:552 ->
##   `WorldGen::get_settlements`, lib.rs:4647). One row per settlement --
##   `x`/`y` are grid-cell ints, already the exact space `move_view_to`
##   expects (see "Coordinate space" below).
## - **Factions** -- `EngineBridge.get_factions()` (lib.rs:6175). A faction
##   has no `x`/`y` of its own; its position is derived the same way
##   `FactionRosterWindow._capital_of()` derives "Focus camera on capital"
##   (faction_roster_window.gd:794-812) -- the highest-population
##   capital/metropolis it owns, else its highest-population settlement of
##   any kind. A faction with zero settlements has nothing to derive a
##   position from and is **not indexed** -- see "What is declined" below.
## - **Labels** -- `EngineBridge.label_list()` (engine_bridge.gd:2047 ->
##   lib.rs:7865). Each carries its own `x`/`y`/`text` already.
## - **Roads and sea routes** -- `EngineBridge.roads()` / `sea_routes()`
##   (engine_bridge.gd:555/558 -> lib.rs:4737/4800). Each carries a `name`,
##   a `km` length and a `points` polyline; there is no single `x`/`y`, so
##   the polyline's midpoint is used (see "Coordinate space" below).
##
## Rebuilt wholesale on every `build()` call -- there is no incremental
## update, matching `CommandIndex`. A generate, an edit to a settlement/
## label/faction, or a territory recompute all change this data, and none of
## them fire a signal this file could subscribe to; the caller rebuilds when
## it knows the world changed (see `open_find_on_map()`'s own header, owned
## by the other pass).
##
## ## Coordinate space -- verified against the two real call sites, not assumed
##
## `ViewportHost.move_view_to(gx, gy)` (viewport_host.gd:663-672) centres
## `rect.position + Vector2((gx+0.5)/g.x, (gy+0.5)/g.y) * rect.size`, where
## `g = EngineBridge.grid_size()`. That is exactly `map_overlay.gd`'s own
## `_cell_to_screen` (map_overlay.gd:952-953) -- so `move_view_to` wants grid
## **cell** coordinates, the same space `get_settlements()`'s `x`/`y` (and
## `icon_get`/`label_get`'s) already use, confirmed live: every existing
## `move_view_to` call site (faction_roster_window.gd:608,707;
## place_editor_window.gd:587; civilization_workspace.gd:399) passes a
## settlement's raw `x`/`y` straight through, no scaling.
##
## Road/sea-route `points`, by contrast, are drawn through
## `_point_to_screen` (map_overlay.gd:956-960): `Vector2(p.x/_gw, p.y/_gh) *
## rect.size` -- **no `+0.5`**, because a route point is already continuous
## rather than a discrete cell index, per that function's own comment ("no
## `+0.5` centering, unlike `_cell_to_screen`'s settlement markers"). But it
## divides by the *same* `_gw`/`_gh` as the cell path, so a route point
## already lives on the identical `0.._gw` / `0.._gh` numeric range as a
## settlement's cell index -- just continuous instead of quantised. Feeding
## one straight into `move_view_to` costs at most half a cell of centring
## error, immaterial at world scale and the same order of error
## `move_view_to`'s own `+0.5` already accepts for a cell-snapped target.
##
## ## What is declined, and why
##
## **Icons** (`EngineBridge.icon_list()`) are not indexed. `icon_dict`
## (lib.rs:5939-5948) carries `family`/`slot`/`set`/`scale` -- every one a
## closed-vocabulary category, not an identifying name. A row reading
## "pine / forest" would not be a *place* a searcher typed a name to find;
## it would repeat the same handful of category strings once per placed
## icon, which `Assets ▸ Icon families` already shows better, aggregated. No
## icon carries anything this index could rank a text query against.
##
## **Provinces** (`EngineBridge.provinces()`) are not indexed either, even
## though `get_provinces()` (lib.rs:4828) does carry a real `name` and a
## derivable position (`capital_settlement_index` into `get_settlements()`,
## the identical derivation this file already does for factions). Declined
## on scope, not capability: the row this file makes live promises exactly
## four families in its own tooltip text -- "Searches world entities --
## places, labels, factions, routes" (menus.gd:533, unchanged by this pass).
## A province's own position is its capital settlement's, which is already a
## `settlement` row here, and a province's territory is its owning faction's,
## already a `faction` row -- so a fifth family would not find the searcher
## anything the other four don't already surface from the same data.
##
## ## Ranking -- `CommandIndex`'s own reasoning (command_index.gd:168-174),
## reused rather than re-argued
##
## Case-insensitive **substring**, deliberately not fuzzy: a searcher typing
## "riv" for a river-named settlement wants every row with "riv" in it, and a
## fuzzy matcher returning unrelated near-misses makes the list less
## trustworthy, not more -- the point is that what matches is visibly why it
## matched. Three bands, each alphabetical by name: the query is a *prefix*
## of the name, the query appears elsewhere in the name, and the query only
## appears in the subtitle or kind. A row lands in the first band it
## qualifies for.

var _rows: Array = []

## (Re)builds the whole index from the live world. Safe to call with no
## world generated yet -- every getter below already returns an empty array
## before the first `generate()` (each one's own doc comment says so), which
## leaves `_rows` empty rather than raising.
func build(bridge) -> void:
	_rows.clear()
	if bridge == null:
		return
	var settlements: Array = bridge.settlements() if bridge.has_method("settlements") else []
	var factions: Array = bridge.get_factions() if bridge.has_method("get_factions") else []
	var labels: Array = bridge.label_list() if bridge.has_method("label_list") else []
	var roads: Array = bridge.roads() if bridge.has_method("roads") else []
	var sea_routes: Array = bridge.sea_routes() if bridge.has_method("sea_routes") else []

	var faction_names := {}   ## faction id (int) -> name (String), for a settlement's own subtitle.
	for f in factions:
		var fd: Dictionary = f
		faction_names[int(fd.get("id", 0))] = String(fd.get("name", ""))

	_add_settlements(settlements, faction_names)
	_add_factions(factions, settlements)
	_add_labels(labels)
	_add_roads(roads)
	_add_sea_routes(sea_routes)

func size() -> int:
	return _rows.size()

func all() -> Array:
	return _rows.duplicate()

## Substring match over name, subtitle and kind, case-insensitive. See the
## header comment for why substring rather than fuzzy, and for the three
## bands. Empty query returns every row, in build order.
func search(q: String) -> Array:
	var needle := q.strip_edges().to_lower()
	if needle == "":
		return _rows.duplicate()
	var prefix: Array = []
	var name_hit: Array = []
	var other: Array = []
	for r in _rows:
		var name_lc := String(r["name"]).to_lower()
		var at := name_lc.find(needle)
		if at == 0:
			prefix.append(r)
		elif at > 0:
			name_hit.append(r)
		elif String(r["subtitle"]).to_lower().find(needle) >= 0 or String(r["kind"]).to_lower().find(needle) >= 0:
			other.append(r)
	var by_name := func(a, b): return String(a["name"]) < String(b["name"])
	prefix.sort_custom(by_name)
	name_hit.sort_custom(by_name)
	other.sort_custom(by_name)
	return prefix + name_hit + other

# -- Settlements --------------------------------------------------------------

## `id` is the row's index into the `settlements` array `build()` fetched --
## the same handle `FactionRosterWindow._build_settlement_sublist()`
## (faction_roster_window.gd:688-710) already opens `open_place_editor(idx)`
## with, rather than the `tid` the dictionary also carries. Stale the moment
## a settlement is added or removed after this index was built, same as
## every other index-based handle already in this shell -- `build()` is the
## caller's own signal to re-fetch, not something this file can detect.
func _add_settlements(settlements: Array, faction_names: Dictionary) -> void:
	for i in settlements.size():
		var s: Dictionary = settlements[i]
		var name := String(s.get("name", "")).strip_edges()
		if name == "":
			continue
		var kind := String(s.get("kind", "settlement"))
		var fname: String = faction_names.get(int(s.get("faction", 0)), "unclaimed")
		_rows.append({
			"name": name,
			"kind": kind,
			"subtitle": "%s · pop %s · %s" % [kind.capitalize(), _thousands(int(s.get("population", 0))), fname],
			"x": float(s.get("x", 0)),
			"y": float(s.get("y", 0)),
			"entity": "settlement",
			"id": i,
		})

# -- Factions -------------------------------------------------------------------

## `id` is the faction id itself (`get_factions()`'s own `id`, 1..=6) -- the
## same int `civ_set_faction_field()` and `FactionRosterWindow._selected`
## already key faction identity by, unlike a settlement's array-index handle.
func _add_factions(factions: Array, settlements: Array) -> void:
	for f in factions:
		var fd: Dictionary = f
		var name := String(fd.get("name", "")).strip_edges()
		if name == "":
			continue
		var fid := int(fd.get("id", 0))
		var cap := _capital_of(fid, settlements)
		if cap.is_empty():
			## No settlement to derive a position from -- see the header's
			## "What is declined" note. Not indexed, not a bug: a faction that
			## owns no settlements yet (freshly added via Add faction) has
			## nowhere on the map that is "its" location.
			continue
		var count := int(fd.get("settlement_count", 0))
		_rows.append({
			"name": name,
			"kind": "faction",
			"subtitle": "Faction · %d settlement%s · pop %s · capital %s" % [
				count, "" if count == 1 else "s",
				_thousands(int(fd.get("population", 0))), String(cap.get("name", "?"))],
			"x": float(cap.get("x", 0)),
			"y": float(cap.get("y", 0)),
			"entity": "faction",
			"id": fid,
		})

## Identical derivation to `FactionRosterWindow._capital_of()`
## (faction_roster_window.gd:794-812): the highest-population
## capital/metropolis this faction owns, else its highest-population
## settlement of any kind. Fully derived from `kind`/`population`, no
## override field -- the reference derives it the same way.
func _capital_of(fid: int, settlements: Array) -> Dictionary:
	var best := {}
	var best_pop := -1
	var best_seat := false
	for s in settlements:
		var d: Dictionary = s
		if int(d.get("faction", 0)) != fid:
			continue
		var kind := String(d.get("kind", ""))
		var seat := kind == "capital" or kind == "metropolis"
		var pop := int(d.get("population", 0))
		if (seat and not best_seat) or (seat == best_seat and pop > best_pop):
			best = d
			best_pop = pop
			best_seat = seat
	return best

# -- Labels ---------------------------------------------------------------------

## `id` is `label_list()`'s own `index` field -- the handle `label_select()`/
## `label_get()`/`label_delete()` already take.
func _add_labels(labels: Array) -> void:
	for lb in labels:
		var d: Dictionary = lb
		var text := String(d.get("text", "")).strip_edges()
		if text == "":
			continue
		_rows.append({
			"name": text,
			"kind": "label",
			"subtitle": "Map label · at grid (%d, %d)" % [int(d.get("x", 0)), int(d.get("y", 0))],
			"x": float(d.get("x", 0)),
			"y": float(d.get("y", 0)),
			"entity": "label",
			"id": int(d.get("index", -1)),
		})

# -- Roads and sea routes --------------------------------------------------------

## `id` is the row's own index into `roads()` -- the only handle that array
## offers (no `tid`-equivalent exists for a way). A road with no name (an
## empty `w.name`) is not indexed for the same reason an unnamed anything
## else here is not: nothing to type to find it by.
func _add_roads(roads: Array) -> void:
	for i in roads.size():
		var d: Dictionary = roads[i]
		var name := String(d.get("name", "")).strip_edges()
		if name == "":
			continue
		var pts: PackedVector2Array = d.get("points", PackedVector2Array())
		if pts.is_empty():
			continue
		## Midpoint rather than the first point: a route's own start is often
		## a small spur off the settlement it leaves, and centring on it can
		## leave most of the route's length off-screen. The midpoint keeps
		## the pan roughly centred on the route regardless of its shape.
		var mid: Vector2 = pts[pts.size() / 2]
		var wtype := String(d.get("way_type", "road"))
		_rows.append({
			"name": name,
			"kind": wtype,
			"subtitle": "%s · %d km" % [wtype.capitalize(), int(round(float(d.get("km", 0.0))))],
			"x": mid.x,
			"y": mid.y,
			"entity": "route",
			"id": i,
		})

## Same shape as `_add_roads`, over `sea_routes()` -- which carries no
## `way_type` at all (`SeaRoute` has one tier, not four), so `kind` is the
## fixed string "sea route" rather than a read field.
func _add_sea_routes(sea_routes: Array) -> void:
	for i in sea_routes.size():
		var d: Dictionary = sea_routes[i]
		var name := String(d.get("name", "")).strip_edges()
		if name == "":
			continue
		var pts: PackedVector2Array = d.get("points", PackedVector2Array())
		if pts.is_empty():
			continue
		var mid: Vector2 = pts[pts.size() / 2]
		_rows.append({
			"name": name,
			"kind": "sea route",
			"subtitle": "%s · %d km" % ["sea route".capitalize(), int(round(float(d.get("km", 0.0))))],
			"x": mid.x,
			"y": mid.y,
			"entity": "route",
			"id": i,
		})

# -- Formatting -------------------------------------------------------------------

## Same space-grouped form `FactionRosterWindow._thousands()`
## (faction_roster_window.gd:819-828) already prints population and territory
## numbers with -- kept identical rather than reused across files so this
## stays the one `RefCounted` `menus.gd`'s row depends on, per this file's own
## two-file ownership.
static func _thousands(n: int) -> String:
	var s := str(absi(n))
	var out := ""
	var c := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		c += 1
		if c % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if n < 0 else "") + out
