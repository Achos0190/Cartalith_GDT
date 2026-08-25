extends RefCounted
class_name TradeStore

## The last trade-flow match, held on the shell side so three surfaces can
## share one computation (`GUI_GAP_REGISTER.md` **IN-13**).
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
## So the engine keeps nothing and the shell keeps one dictionary. The
## difference is not cosmetic: a GDScript dictionary is dropped by
## `clear()` on any world change, and the engine is never asked to hold a
## per-cell field for the lifetime of a session — which was the whole of the
## register's memory objection.
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

## The last result, or `{}` when nothing has been matched since the last
## world change.
static var _last: Dictionary = {}

## True when a match has been run against the current world.
static func is_matched() -> bool:
	return not _last.is_empty()

## The last match, `{}` if there is none. Never runs one — a reader that
## finds this empty should say so rather than silently paying for a match the
## user did not ask for.
static func last() -> Dictionary:
	return _last

## Run a match and keep it. Returns the same dictionary [`last`] will.
static func refresh(bridge) -> Dictionary:
	_last = bridge.civ_trade_flows()
	return _last

## Drop it. Called from `app.gd` on every world change.
static func clear() -> void:
	_last = {}

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
