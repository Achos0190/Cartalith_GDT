extends RefCounted
## AC-1 (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1; Ruling DW): the shell half of the ACTION CATALOGUE.
## `cartalith_llm::actions` builds the catalogue (ids, descriptions, effects, the by-construction
## refusals); this file has TWO jobs. AC-1: [`facts`] reads, from the live app, the facts Rust cannot know
## (it writes nothing, runs nothing and opens no menu). AC-3: [`run_card`] runs a `propose_actions` card
## through each kind's existing click path, in one undo group, after [`preflight`] (see the AC-3 block at
## the end of the file). Only `run_card` changes anything, and only from the card's Apply button.
##
## What it reads, every request (and every turn of one), so availability is the state NOW and never
## the snapshot a menu was built in (the proposal's "stale availability" hazard):
##
##   * the RUNNABLE `CommandIndex` rows: `kind == "menu"` and `pinnable`. `pinnable` is NOT a safety
##     mark (Save, Close, New world and Export are all pinnable; `CommandIndex` says so itself), so
##     this list is only the candidate set: Rust drops every row naming a forbidden word and lists
##     every row nobody reviewed as `unreviewed` and not proposable. Each row is named by its
##     `DccMenus` `ID_` constant, not its number, so an id survives a renumbering;
##   * which of those rows open a curated window (`llm_view.gd::WINDOWS`, the single table), so Rust
##     lists the window once, not twice;
##   * whether a world is open, and the year cursor's track (`DccShell.tl_range()`, the world's own range since PC-3).
##
## The view verbs, the windows and the parameters are NOT read here: `llm_view.gd::facts` and the
## registry already supply them to the same request, and a second reader would be a second source.
## The context-card actions and the map tools are not in any runtime registry; Rust embeds a scan
## of the shell source for them (`cartalith-llm/data/action_static_sources.json`, kept honest by
## `tests/ac1_static_sources.rs`).
##
## No `class_name` (preloaded by `llm_assistant_window.gd`), so adding it needs no `godot --import`
## pass, which strips `project.godot`'s comments (`CLAUDE.md`).
##
## MUST NEVER ([`facts`]): run a row (`CommandIndex.run_row` is `run_card`'s alone); emit `about_to_popup` (the
## index builder documents why: it evicts chunks and enumerates GPU adapters); report a row as
## available that the index marks unavailable; invent a `has_world` when the bridge cannot say
## (the key is then ABSENT, and Rust treats absence as "not said", not as "no world").

const _VIEW := preload("res://shell/llm_view.gd")


## `DccMenus` command id -> EVERY constant name that has that value, lower-cased without `ID_`, sorted
## (`719 -> ["factions"]`). Two constants can share a value (the 2026-10-31 probe found `ID_DESELECT` and
## `ID_LOD_EXPORT_ATLAS` both on the Deselect row), so a name is chosen per row by [`pick_name`]. A range
## marker (`..._FIRST`, `..._LAST`) is skipped: it names the start of a generated span, not a command.
static func menu_names() -> Dictionary:
	var out := {}
	var consts: Dictionary = (DccMenus as GDScript).get_script_constant_map()
	var keys: Array = consts.keys()
	keys.sort()
	for k in keys:
		var name := String(k)
		if not name.begins_with("ID_") or name.ends_with("_FIRST") or name.ends_with("_LAST"):
			continue
		var v = consts[k]
		if typeof(v) != TYPE_INT:
			continue
		if not out.has(int(v)):
			out[int(v)] = []
		(out[int(v)] as Array).append(name.substr(3).to_lower())
	return out


## The constant name of a row among those sharing its id: the one sharing the most words with the row's
## title, ties to the alphabetically first. Deterministic, and it names `Deselect` `deselect` rather than
## the unrelated alias that sorts first. A single candidate is returned as is.
static func pick_name(candidates: Array, title: String) -> String:
	if candidates.size() == 1:
		return String(candidates[0])
	var words := {}
	for w in title.to_lower().split(" ", false):
		var clean := ""
		for ch in String(w):
			if (ch >= "a" and ch <= "z") or (ch >= "0" and ch <= "9"):
				clean += ch
		if clean != "":
			words[clean] = true
	var best := ""
	var best_score := -1
	for c in candidates:
		var score := 0
		for t in String(c).split("_", false):
			if words.has(t):
				score += 1
		if score > best_score:
			best_score = score
			best = String(c)
	return best


## The facts Rust builds the catalogue's shell half from (`ShellFacts::from_json`), read NOW.
## Rows are sorted by name so two builds of the same app give byte-identical text.
static func facts(app, bridge) -> Dictionary:
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	var names := menu_names()
	var window_of := {}
	for w in _VIEW.WINDOWS:
		window_of[int((w as Dictionary)["menu"])] = String((w as Dictionary)["id"])
	var rows: Array = []
	for r in idx.all():
		var row: Dictionary = r
		if String(row.get("kind", "")) != "menu" or not bool(row.get("pinnable", false)):
			continue
		var cid := int(row.get("_id", -1))
		if not names.has(cid):
			continue
		var d := {
			"name": pick_name(names[cid], String(row.get("title", ""))),
			"title": String(row.get("title", "")),
			"group": String(row.get("group", "")),
			"available": bool(row.get("available", false)),
			"why": String(row.get("why", "")),
		}
		if window_of.has(cid):
			d["window"] = String(window_of[cid])
		rows.append(d)
	rows.sort_custom(func(a, b): return String(a["name"]) < String(b["name"]))
	var f := {"menu_rows": rows}
	if bridge != null:
		f["has_world"] = bool(bridge.has_world)
	if app != null and app.has_method("tl_set_year"):
		## PC-3: the track is the world's range (`DccShell.tl_range()`), the fallback axis for a world that
		## gave none, so the catalogue's bounds and `tl_set_year()`'s clamp are the same two numbers.
		var rng: Dictionary = app.tl_range()
		f["timeline"] = {"min": int(rng["min"]), "max": int(rng["max"])}
	## AC-3: the selection verbs' bounds. Both keys are ABSENT when the app cannot say (no world, or
	## `DccApp` without PC-1's selection), never `0` / `false`: Rust offers no selection entry for an absent
	## fact. `bridge.settlements()` is an engine read, so it is skipped while a generation owns the engine.
	if bridge != null and bool(bridge.has_world) and not bool(bridge.generating) and app != null and app.has_method("select"):
		f["settlements"] = (bridge.settlements() as Array).size()
		var back = app.get("_selection_back")
		if back is Array:
			f["can_select_back"] = (back as Array).size() > 0
	return f


## [`facts`] as the JSON text `LlmStream`'s `args.actions` takes.
static func facts_json(app, bridge) -> String:
	return JSON.stringify(facts(app, bridge))


# ---------------------------------------------------------------------------------------------
# AC-3: running a `propose_actions` card (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.5; Ruling DW)
# ---------------------------------------------------------------------------------------------
#
# Every action runs through the click path its KIND already has; nothing here is a second
# implementation of any of them:
#
#   param (setting)   `undo_group.write_param`, the setter every dialog and the parameter card use
#   view verbs        `LlmView.apply` (layers, landmark filter, zoom) and, for the style preset, the same
#                     `RenderWorkspace._apply_preset` it calls, into the card's group
#   window / menu     `CommandIndex.run_row` on the live row (what a click on the menu item emits)
#   year              `DccShell.tl_set_year` (the year writer, as it stands)
#   selection         `DccApp.select` / `clear_selection` / `select_back` (PC-1's one path)
#   tool              `DccApp.arm_tool`
#
# The decision whether a card may run AT ALL (Session mode, a generation, the five-action cap, a missing
# click path, a finalized world) is Rust's `cartalith_llm::action_run::check_card`, asked by the window
# before Apply is enabled; [`preflight`] below is the half only the shell can answer: is each action still
# true of the app NOW (the row still there, the index in range, the key still a setting).
#
# One undo group per card: the settings and the style preset are members of ONE group, so one Edit > Undo
# reverts them together. The other kinds are session state (a layer, the camera, an open window, the
# selection, the year cursor, an armed tool) and are not on the undo stack, as the view card has always
# said (`llm_view.gd` header); the card says so beside Apply.
#
# MUST NEVER: run a card that preflight refused (nothing changes on a refusal); call anything but the
# paths above; regenerate (a card never rebuilds the world: pressing Generate is the user's click); open a
# second undo group inside the card's (groups do not nest) or leave one open on any return.


## The `CommandIndex` row for menu constant name `name` that is pinnable and available now, or `{}`.
## Named exactly as [`facts`] names rows (`pick_name` over the constants sharing the row's id), so an id
## the catalogue offered resolves to the row it described.
static func menu_row(idx: CommandIndex, name: String) -> Dictionary:
	var names := menu_names()
	for r in idx.all():
		var row: Dictionary = r
		if String(row.get("kind", "")) != "menu" or not bool(row.get("pinnable", false)) or not bool(row.get("available", false)):
			continue
		var cid := int(row.get("_id", -1))
		if names.has(cid) and pick_name(names[cid], String(row.get("title", ""))) == name:
			return row
	return {}


## The `DccMenus` command id of curated window `w` (`llm_view.gd::WINDOWS`), or -1.
static func window_menu(w: String) -> int:
	for wd in _VIEW.WINDOWS:
		if String((wd as Dictionary)["id"]) == w:
			return int((wd as Dictionary)["menu"])
	return -1


## The `LlmView` change dictionary of a view-route action (`{action, args, label, undoable, to}`).
## `c` is one entry of the card's `valid` array. An action id this does not know returns `{}`.
static func view_change(c: Dictionary) -> Dictionary:
	var id := String(c.get("id", ""))
	var a: Dictionary = c.get("args", {})
	var label := String(c.get("title", id))
	if id.begins_with("view.layer."):
		return {"action": "set_layer_visible", "args": {"layer": id.substr("view.layer.".length()), "visible": bool(a.get("visible", false))}, "label": label, "undoable": false}
	match id:
		"view.style_preset":
			return {"action": "set_style_preset", "args": {"preset": String(a.get("preset", ""))}, "label": label, "undoable": true, "to": String(a.get("preset", ""))}
		"view.zoom_to":
			return {"action": "zoom_to", "args": a.duplicate(), "label": label, "undoable": false}
		"view.landmark_filter":
			return {"action": "set_landmark_filter", "args": {"kind": String(a.get("kind", "")), "visible": bool(a.get("visible", false))}, "label": label, "undoable": false}
	return {}


## The value to write for a setting: the card's number is a float (it crossed JSON), and an integer
## setting takes an integer. Compared with the setting's CURRENT type, which the registry fixes.
static func coerce_param(bridge, key: String, v):
	var cur = bridge.param_get(key)
	if typeof(cur) == TYPE_INT and (typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT):
		return int(v)
	return v


## "" when every action of `valid` is still true of the app NOW, else the reasons in words. Reads only:
## builds a `CommandIndex` (never opens a menu), asks `LlmView.first_stale`, reads counts. A card that
## fails here is STALE (the app changed since it was proposed) and Apply stays off.
static func preflight(app, bridge, valid: Array) -> String:
	var why: PackedStringArray = PackedStringArray()
	var views: Array = []
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	for c in valid:
		var cd: Dictionary = c
		var id := String(cd.get("id", ""))
		var a: Dictionary = cd.get("args", {})
		var title := String(cd.get("title", id))
		match String(cd.get("route", "")):
			"param":
				if bridge.param_get(id.substr("param:".length())) == null:
					why.append("%s is not a setting this build has" % title)
			"view":
				var vc := view_change(cd)
				if vc.is_empty():
					why.append("%s is not a view change this build can run" % title)
				else:
					views.append(vc)
			"window":
				var menu := window_menu(id.substr("window.".length()))
				if menu < 0 or _VIEW.window_row(idx, menu).is_empty():
					why.append("%s is not available now" % title)
			"menu":
				if menu_row(idx, id.substr("menu:".length())).is_empty():
					why.append("%s is not available now" % title)
			"year":
				var y := int(a.get("year", 0))
				if not app.has_method("tl_set_year") or not bool(app.tl_available()):
					why.append("the year cursor is not available")
				elif y < app.tl_range_min() or y > app.tl_range_max():
					why.append("year %d is outside the track" % y)
			"select":
				match id:
					"select.settlement":
						var n := (bridge.settlements() as Array).size()
						var sid := int(a.get("id", -1))
						if sid < 0 or sid >= n:
							why.append("settlement %d no longer exists (%d settlements)" % [sid, n])
					"select.back":
						var back = app.get("_selection_back")
						if not (back is Array) or (back as Array).is_empty():
							why.append("there is no earlier selection to return to")
					"select.none":
						pass
					_:
						why.append("%s is not a selection this build can run" % title)
			"tool":
				if not app.has_method("arm_tool"):
					why.append("tools cannot be armed here")
			_:
				why.append("%s has no way to run from a card" % title)
	if not views.is_empty():
		var gone: String = _VIEW.first_stale(app, bridge, views)
		if gone != "":
			why.append("%s is no longer available" % gone)
	return "; ".join(why)


## Runs the card `valid` and returns `{ok, reason, stale, applied, group_id, undoable, session, failed}`.
## `ok` false means NOTHING was changed (a preflight, group or write refusal, rolled back whole); `ok` true
## with `failed` non-empty means the group committed and a later session-state step did not land, each
## named. The caller has already asked `check_card` (Session mode, generating, size, ...).
##
## Order, and why: (1) preflight everything; (2) ONE undo group holding every setting then the style preset
## (a rejected or clamped write aborts the whole group, `undo_group.gd`); (3) commit it; (4) the other view
## verbs (a style redraw re-fits the camera, so a camera move comes after it); (5) windows and menu rows,
## the year, the selection and tools, in the card's own order. `label` names the undo step.
static func run_card(app, bridge, ug, valid: Array, label: String) -> Dictionary:
	var stale := preflight(app, bridge, valid)
	if stale != "":
		return _fail("%s, so nothing was applied." % stale, true)
	var params: Array = []
	var views_undo: Array = []
	var views_instant: Array = []
	var rest: Array = []
	for c in valid:
		var cd: Dictionary = c
		match String(cd.get("route", "")):
			"param":
				params.append(cd)
			"view":
				var vc := view_change(cd)
				if bool(vc.get("undoable", false)):
					views_undo.append(vc)
				else:
					views_instant.append(vc)
			_:
				rest.append(cd)
	var applied := 0
	var group_id := 0
	var undoable := params.size() + views_undo.size()
	if undoable > 0:
		var refused: String = ug.begin(label)
		if refused != "":
			return _fail("Not applied: %s." % refused, false)
		for p in params:
			var pd: Dictionary = p
			var key := String(pd["id"]).substr("param:".length())
			var val = coerce_param(bridge, key, (pd["args"] as Dictionary)["value"])
			if not ug.write_param(key, val):
				## `write_param` already rolled the whole group back.
				return _fail("Not applied: %s. Nothing was changed." % String(ug.last_reason), false)
		var r0: Dictionary = _VIEW.apply_undoable_into_open_group(app, views_undo)
		if not bool(r0.get("ok", false)):
			var ab: Dictionary = ug.abort(String(r0.get("reason", "")))
			return _fail("Not applied: %s. %s" % [String(r0.get("reason", "")), String(ab.get("reason", ""))], false)
		var r1: Dictionary = ug.commit()
		if not bool(r1.get("ok", false)):
			return _fail("Not applied: %s. Nothing was changed." % String(r1.get("reason", "")), false)
		group_id = int(r1.get("id", 0))
		applied += undoable
	var failed: Array = []
	var session := 0
	if not views_instant.is_empty():
		## No undoable member, so `LlmView.apply` opens no group (and cannot collide with the card's).
		var rv: Dictionary = _VIEW.apply(app, bridge, ug, views_instant)
		if not bool(rv.get("ok", false)):
			failed.append(String(rv.get("reason", "the view changes were not applied")))
		else:
			session += int(rv.get("instant", 0))
			failed.append_array(rv.get("failed", []))
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	for c in rest:
		var cd: Dictionary = c
		var id := String(cd["id"])
		var a: Dictionary = cd.get("args", {})
		var bad := ""
		match String(cd.get("route", "")):
			"window":
				var wr: Dictionary = _VIEW.window_row(idx, window_menu(id.substr("window.".length())))
				if wr.is_empty() or not idx.run_row(wr):
					bad = "%s could not be opened" % String(cd["title"])
			"menu":
				var mr := menu_row(idx, id.substr("menu:".length()))
				if mr.is_empty() or not idx.run_row(mr):
					bad = "%s could not be run" % String(cd["title"])
			"year":
				app.tl_set_year(int(a["year"]))
				if int(app.tl_year()) != int(a["year"]):
					bad = "the year cursor is at %d, not %d" % [int(app.tl_year()), int(a["year"])]
			"select":
				match id:
					"select.settlement":
						app.select("settlement", int(a["id"]))
						if String(app.selection.get("kind", "")) != "settlement" or int(app.selection.get("id", -1)) != int(a["id"]):
							bad = "settlement %d was not selected" % int(a["id"])
					"select.none":
						app.clear_selection()
						if not (app.selection as Dictionary).is_empty():
							bad = "the selection was not cleared"
					"select.back":
						if not bool(app.select_back()):
							bad = "there was no earlier selection"
			"tool":
				var tid := id.substr("tool.".length())
				app.arm_tool(tid)
				if String(app.armed_tool) != tid:
					bad = "the %s tool was not armed" % tid
			_:
				bad = "%s has no way to run" % String(cd["title"])
		if bad == "":
			session += 1
		else:
			failed.append(bad)
	return {"ok": true, "reason": "", "stale": false, "applied": applied + session, "group_id": group_id, "undoable": undoable, "session": session, "failed": failed}


static func _fail(reason: String, stale: bool) -> Dictionary:
	return {"ok": false, "reason": reason, "stale": stale, "applied": 0, "group_id": 0, "undoable": 0, "session": 0, "failed": []}
