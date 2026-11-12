extends RefCounted
## MS-8 (`MOBILE_SHELL_PROPOSAL.md` section 2.3, Ruling DT): New World's AUTO-POPULATE.
##
## **What this is.** The policy for which of four existing passes run after a new world's terrain is
## generated, and the runner that runs them. It holds no pass of its own: every pass goes through the
## very handler its menu button runs (`CivilizationWorkspace._populate_world_now`,
## `InfrastructureWorkspace._generate_roads`, `CivilizationWorkspace._lm_run`,
## `CartographyWorkspace._regenerate_labels`), so "a new world with all four on equals Generate plus
## the four passes run by hand" is true by construction, and `_autopopulate_probe.gd` measures it.
## There is no `class_name`, so adding this file needs no `godot --headless --import` pass.
##
## **What it must never do.** Re-implement a pass; run any pass when the world did not generate; run a
## pass the user switched off; run Routes without Settlements (routes connect settlements, and
## `civ_auto_routes` refuses an empty roster).
##
## **Order, and one deliberate departure from the proposal's table.** The table lists Names before
## Landmarks. The labelling pass reads the landmark store (`WorldGen::label_world` passes
## `landmark_store.last` as `LabelWorld.landmarks`), so names run first would name no landmark and the row's
## own promise ("place, region, water and landmark names") would be false. The order here is
## settlements, routes, landmarks, names; the proposal's section 2.3 carries the same correction.

const KEY_SETTLEMENTS := "settlements"
const KEY_ROUTES := "routes"
const KEY_LANDMARKS := "landmarks"
const KEY_NAMES := "names"

## The run order (see the class doc for why Names is last). A literal list on purpose: the probe asserts
## it as a literal, and a reorder is a behaviour change that needs an owner-visible reason.
const ORDER: Array[String] = [KEY_SETTLEMENTS, KEY_ROUTES, KEY_LANDMARKS, KEY_NAMES]

## Ruling DT, question Q4: auto-populate is on by default, all four.
const DEFAULT_ON := true

## The caption shown on the greyed Routes switch while Settlements is off.
const REASON_ROUTES_NEED_SETTLEMENTS := "needs settlements to connect"
## The caption shown on the four switches while the master switch is off.
const REASON_MASTER_OFF := "turn on Auto-populate the world"

## The passes to run, in order, for these switch states. `master` is the "Auto-populate the world" switch and
## `on` maps each key in `ORDER` to its own switch. Routes is dropped when Settlements is off, whatever its
## own switch says: the UI greys it, and this is the second guard for a caller that bypasses the UI.
static func plan(master: bool, on: Dictionary) -> Array[String]:
	var out: Array[String] = []
	if not master:
		return out
	for key in ORDER:
		if not bool(on.get(key, false)):
			continue
		if key == KEY_ROUTES and not bool(on.get(KEY_SETTLEMENTS, false)):
			continue
		out.append(key)
	return out

## Runs `passes` (a result of `plan`) over the world that has just generated, in the order given, through the
## workspaces `app` holds. Returns one row per pass, `{"pass": key, "ran": bool}` plus `"reason"` only when it
## did not run (a missing workspace or handler); a pass the engine itself refused reports that through its own
## workspace note and status line, exactly as when pressed by hand. Awaited by the caller.
static func run(app: Node, passes: Array) -> Array:
	var rows: Array = []
	var civ: Node = _workspace(app, "CivilizationWorkspace")
	var carto: Node = _workspace(app, "CartographyWorkspace")
	for key in passes:
		var row := {"pass": String(key), "ran": false}
		match String(key):
			KEY_SETTLEMENTS:
				if civ != null and civ.has_method("_populate_world_now"):
					await civ._populate_world_now()
					row["ran"] = true
				else:
					row["reason"] = "the CIVIL workspace is not mounted"
			KEY_ROUTES:
				var infra: Object = civ.get("_infra") if civ != null else null
				if infra != null and infra.has_method("_generate_roads"):
					await infra._generate_roads()
					row["ran"] = true
				else:
					row["reason"] = "the routes panel is not mounted"
			KEY_LANDMARKS:
				if civ != null and civ.has_method("_lm_run"):
					await civ._lm_run()
					row["ran"] = true
				else:
					row["reason"] = "the CIVIL workspace is not mounted"
			KEY_NAMES:
				if carto != null and carto.has_method("_regenerate_labels"):
					carto._regenerate_labels()
					row["ran"] = true
				else:
					row["reason"] = "the CARTO workspace is not mounted"
		rows.append(row)
	return rows

## The one workspace of script class `cls` that `app` built (`DccApp._workspaces`), or null. Matched by the
## script's global class name so no workspace file is preloaded here.
static func _workspace(app: Node, cls: String) -> Node:
	var list: Variant = app.get("_workspaces")
	if not (list is Array):
		return null
	for w in list:
		var s: Script = (w as Object).get_script() as Script
		if s != null and s.get_global_name() == cls:
			return w as Node
	return null
