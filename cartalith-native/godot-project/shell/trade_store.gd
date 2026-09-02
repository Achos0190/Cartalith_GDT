extends RefCounted
class_name TradeStore

## The last trade-flow match AND the last food-shed pass, held on the shell
## side so three surfaces can share one computation (`GUI_GAP_REGISTER.md`
## **IN-13**; the food shed is `ECONOMY_SCOPE.md` milestone 2).
##
## ## Why this exists at all
##
## `cartalith_civ::trade` is deliberately stateless — it matches, answers and
## drops, the way `territory_influence` and `wildlife` do, and `CivData`
## gains no field. That is the right shape for the engine and it leaves one
## real problem: the *same* answer is read by CIVIL ▸ Trade, by the place
## editor's per-settlement ledger, and by the map's way-load overlay. Running
## the match once per reader would mean a quarter-second recompute every time
## somebody opens a place editor.
##
## So the engine keeps nothing and the shell keeps one dictionary each. The
## difference is not cosmetic: a GDScript dictionary is dropped by
## `clear()` on any world change, and the engine is never asked to hold a
## per-cell field for the lifetime of a session — which was the whole of the
## register's memory objection.
##
## The food-shed pass is cached the same way and for the same reason, even
## though it has one real reader today ([refresh] itself) and no dock section
## yet: `food_shed_for()` is written for the place editor's Trade tab, right
## beside its existing [navigability] read, so that section can be added
## without a second engine call.
##
## The smelting and salt-access passes (`ECONOMY_SCOPE.md` EC-2/EC-7,
## 2026-09-02) are cached the same way again, one dictionary each: neither
## is cheap enough to recompute per place-editor open (both rebuild
## `lithology`/`biome`/`resources` on demand -- `civ_trade_bridge.rs`'s own
## module doc explains why), and both read the same
## `civ_place_smelting()`/`civ_salt_access()` shape [refresh] already
## triggers everything else from.
##
## ## When it is dropped
##
## `app.gd`'s `_refresh_world_dependent()` calls [`clear`] — the same one
## place that already re-runs every workspace's `on_world_changed()`. A
## generate, a load, an asset-pack swap and a civ recompute all pass through
## it. Nothing else may cache a match.
##
## Static, like `VaultStore`, and for the same reason: there is exactly one
## of these per running app and threading an instance through five call sites
## would buy nothing.

## The last trade-flow match, or `{}` when nothing has been matched since the
## last world change.
static var _last: Dictionary = {}

## The last food-shed pass, or `{}` when none has run since the last world
## change. Populated by [refresh] alongside the trade-flow match -- see this
## file's own module doc for why the two share one trigger.
static var _food_shed: Dictionary = {}

## The last smelting pass, or `{}` when none has run since the last world
## change. Populated by [refresh] alongside everything else -- see this
## file's own module doc.
static var _smelting: Dictionary = {}

## The last salt-access pass, or `{}` when none has run since the last world
## change. Populated by [refresh] alongside everything else -- see this
## file's own module doc.
static var _salt: Dictionary = {}

## True when a match has been run against the current world.
static func is_matched() -> bool:
	return not _last.is_empty()

## The last match, `{}` if there is none. Never runs one — a reader that
## finds this empty should say so rather than silently paying for a match the
## user did not ask for.
static func last() -> Dictionary:
	return _last

## True when a food-shed pass has been run against the current world.
static func is_food_shed_matched() -> bool:
	return not _food_shed.is_empty()

## The last food-shed pass, `{}` if there is none. Never runs one, for the
## same reason [last] does not.
static func food_shed() -> Dictionary:
	return _food_shed

## One settlement's food-shed row from the last pass -- `{}` when none has
## run, or when this index has no row. Parallel in shape to [navigability]
## below: `local_capacity`, `hinterland_capacity`, `import_capacity`,
## `supported`, `suppliers`, `best_mode`, `limited_by`, `sustainable`,
## `over_by` (`civ_food_shed()`'s own doc comment names every key).
static func food_shed_for(index: int) -> Dictionary:
	var rows: Array = _food_shed.get("rows", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]

## One settlement's smelting economics from the last pass -- `{}` when none
## has run, or when this index has no row. `iron_kg_yr`, `charcoal_kg_yr`,
## `ore_kg_yr`, `woodland_ha`, `limited_by` (`fuel`/`ore`), `fuel_poor`,
## `ore_rich`, `coppice_ha_needed` (`civ_place_smelting()`'s own doc comment
## names every key).
static func smelting_for(index: int) -> Dictionary:
	var rows: Array = _smelting.get("rows", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]

## One settlement's salt access from the last pass -- `{}` when none has run,
## or when this index has no row. `has`, `source` (`none`/`sea salt`/
## `salt deposit`/`salt lake` -- `civ_salt_access()`'s own doc comment names
## both keys).
static func salt_access_for(index: int) -> Dictionary:
	var rows: Array = _salt.get("rows", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]

## Run the trade-flow match and every per-settlement pass, and keep all four.
## Bundled behind this one call rather than four: all are on-demand,
## held-nowhere reads of the same settlement/way state, and the shell offers
## exactly one trigger for any of them (`infrastructure_workspace.gd`'s
## "Match trade flows"). Returns the same dictionary [`last`] will.
static func refresh(bridge) -> Dictionary:
	_last = bridge.civ_trade_flows()
	_food_shed = bridge.civ_food_shed()
	_smelting = bridge.civ_place_smelting()
	_salt = bridge.civ_salt_access()
	return _last

## Drop all four. Called from `app.gd` on every world change.
static func clear() -> void:
	_last = {}
	_food_shed = {}
	_smelting = {}
	_salt = {}

## Every flow touching one settlement, split by direction, from the last
## match. `{"imports": [...], "exports": [...]}` — both empty when no match
## has run, which the caller must distinguish from "this place trades
## nothing" and does.
##
## Linear over `flows` rather than indexed: the array is a few thousand rows
## at most (`MAX_FLOW_ROWS`), this runs on a window open and not on a frame,
## and a per-settlement index would be a second structure to keep in step
## with the first for no measurable gain.
static func ledger(index: int) -> Dictionary:
	var out := {"imports": [], "exports": []}
	for f in _last.get("flows", []):
		var d: Dictionary = f
		if int(d.get("to", -1)) == index:
			out["imports"].append(d)
		elif int(d.get("from", -1)) == index:
			out["exports"].append(d)
	return out

## What this settlement needs and nothing can reach, from the last match.
## Empty `PackedStringArray` when it is supplied, or when no match has run.
static func unmet_for(index: int) -> PackedStringArray:
	for u in _last.get("unmet", []):
		var d: Dictionary = u
		if int(d.get("index", -1)) == index:
			return d.get("goods", PackedStringArray())
	return PackedStringArray()

## One settlement's water access from the last match — `{}` when none has
## run. `kind` is `none`/`stream`/`river`/`sea` and `basis` is the
## reference's own reason string.
static func navigability(index: int) -> Dictionary:
	var rows: Array = _last.get("navigability", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]
