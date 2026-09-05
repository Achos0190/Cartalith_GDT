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
## The food-shed pass is cached the same way and for the same reason, and it
## now has the reader it was written for: `place_editor_window.gd`'s
## `_food_shed_note`, drawn in the Trade section right beside the
## [navigability] read, one `DccWidgets.note` per clause. (Through 2026-09-01
## this paragraph said the pass had "no dock section yet" and named the
## editor as somewhere a section "can be added"; the section landed and the
## paragraph did not. Corrected 2026-09-05 -- the symbol is
## `_food_shed_note`.
##
## That correction shipped its own false clause in the same edit: it said
## `_foodshed_probe.gd` "drives all four of its branches". It has **five**
## branches after that edit, and that probe reaches **branch 1 only** -- its
## `_render()` does `TradeStore._food_shed = {} if row == null else
## {"rows": [row]}`, so the empty case is the only absent state it produces.
## The "pass held, index past its rows" branch is driven by
## `_lanedfoodshed_probe.gd` T2. Caught by a verifier the same day: a stale
## clause written *inside* the correction that existed to remove one.)
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

## True when a smelting pass has been run against the current world.
##
## This and [is_salt_matched] complete a set that was two-of-four: [refresh]
## populates all four dictionaries and **any one of them can independently be
## `{}`** (each has its own `has_method` guard here and its own `_has()`
## binding guard in `engine_bridge.gd` -- which is exactly the state smelting
## and salt were in from 2026-09-02 to 2026-09-05). A reader that can only
## ask two of the four cannot tell "the pass is absent" from "the pass ran and
## this index is past it", and `place_editor_window.gd::_smelting_salt_note`
## needs that distinction for the same reason `_food_shed_note` does: without
## it, both branches print the whole-pass reason, and on the second branch
## that reason is false. `_bridgeforward_probe.gd` had to measure row counts
## through the per-settlement readers because these two did not exist.
static func is_smelting_matched() -> bool:
	return not _smelting.is_empty()

## True when a salt-access pass has been run against the current world. See
## [is_smelting_matched] for why both were added 2026-09-05.
static func is_salt_matched() -> bool:
	return not _salt.is_empty()

## The last food-shed pass, `{}` if there is none. Never runs one, for the
## same reason [last] does not.
static func food_shed() -> Dictionary:
	return _food_shed

## One settlement's food-shed row from the last pass -- `{}` when none has
## run, or when this index has no row. Parallel in shape to [navigability]
## below.
##
## Twelve keys, in two groups, because they come from two places and a reader
## writing a probe against this needs to know which:
##
## * From `cartalith_civ::trade::FoodShed` itself -- `local_capacity`,
##   `hinterland_capacity`, `import_capacity`, `supported`, `suppliers`,
##   `best_mode` (`land`/`river`/`sea`), `limited_by` (`local`/`trade`),
##   `sustainable`, `over_by`.
## * Added by the binding from `civ.settlements[i]`, not by the model --
##   `index`, `name`, `pop`. `pop` is the settlement's population **as the
##   pass saw it**, which is why `place_editor_window.gd::_food_shed_note`
##   draws it as "Population N when the match ran" rather than reading a
##   live figure back off the roster.
##
## This list previously named the nine model keys only and claimed the Rust
## doc "names every key" -- while `_food_shed_note` reads `pop`, one of the
## three it omitted. Corrected 2026-09-05 against `civ_trade_bridge.rs`'s
## `civ_food_shed` `dict!` literal, which is the definition; `_lanedfoodshed_probe.gd`
## now asserts all twelve by `has()` on a live pass so the two cannot drift
## again silently.
static func food_shed_for(index: int) -> Dictionary:
	var rows: Array = _food_shed.get("rows", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]

## One settlement's smelting economics from the last pass -- `{}` when none
## has run, or when this index has no row. Ten keys: `iron_kg_yr`,
## `charcoal_kg_yr`, `ore_kg_yr`, `woodland_ha`, `limited_by` (`fuel`/`ore`),
## `fuel_poor`, `ore_rich`, `coppice_ha_needed` from the model, plus `index`
## and `name` added by the binding -- the same two-group split
## [food_shed_for] documents above, and the same two the list here omitted
## until 2026-09-05. `_bridgeforward_probe.gd`'s T5 asserts all ten by
## `has()` on a live pass.
##
## `coppice_ha_needed` is the one key no surface draws:
## `place_editor_window.gd::_smelting_salt_note` reports the four figures
## and the two lopsided-case flags and stops there. Not a defect to paper
## over with a dashed row -- the value is present and correct, it simply has
## no reader yet.
static func smelting_for(index: int) -> Dictionary:
	var rows: Array = _smelting.get("rows", [])
	if index < 0 or index >= rows.size():
		return {}
	return rows[index]

## One settlement's salt access from the last pass -- `{}` when none has run,
## or when this index has no row. Four keys: `has` and `source`
## (`none`/`sea salt`/`salt deposit`/`salt lake`) from the model, plus
## `index` and `name` added by the binding -- the same two-group split
## [food_shed_for] documents above. `_bridgeforward_probe.gd`'s T5 asserts
## all four by `has()` on a live pass.
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
##
## Every read is `has_method`-guarded, on `place_search.gd::build()`'s own
## precedent -- the same shape, an untyped `bridge` feeding a run of
## consecutive optional reads, and it guards all five of its own. The guard
## asks a different question from the `_has()` inside each wrapper: that one
## answers *does the loaded cdylib export this `#[func]`*, this one answers
## *does this `EngineBridge` have a wrapper at all*, and only the second was
## ever false here. `civ_place_smelting`/`civ_salt_access` had no wrapper from
## 2026-09-02 to 2026-09-05 while the `#[func]`s existed and the cdylib
## exported them, so this function aborted at line 3 of 4 with `Nonexistent
## function 'civ_place_smelting' in base 'Node (EngineBridge)'` and never
## returned -- taking `Match trade flows` down with it. `menus.gd::_engine_has()`
## states the two-question split; this is the case that proves it needs both
## halves. All four are guarded rather than the two that broke: the run is one
## defect class, and a guard on half of it is the shape that produced this.
##
## `{}` from a refused read is the same absent value the four `static var`s
## already document and every reader already distinguishes -- `is_empty()` in
## `infrastructure_workspace.gd`, a dashed row with its reason in
## `place_editor_window.gd::_smelting_salt_note()`. Nothing here mints a zero.
static func refresh(bridge) -> Dictionary:
	_last = bridge.civ_trade_flows() if bridge.has_method("civ_trade_flows") else {}
	_food_shed = bridge.civ_food_shed() if bridge.has_method("civ_food_shed") else {}
	_smelting = bridge.civ_place_smelting() if bridge.has_method("civ_place_smelting") else {}
	_salt = bridge.civ_salt_access() if bridge.has_method("civ_salt_access") else {}
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
