extends RefCounted
## AC-1 (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1; Ruling DW): the shell half of the ACTION CATALOGUE.
## `cartalith_llm::actions` builds the catalogue (ids, descriptions, effects, the by-construction
## refusals); this file does ONE job: read, from the live app, the facts Rust cannot know. It
## writes nothing, runs nothing and opens no menu.
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
##   * whether a world is open, and the year cursor's track (`DccShell.TL_YEAR_MIN`/`MAX`).
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
## MUST NEVER: run a row (`CommandIndex.run_row` is not called here); emit `about_to_popup` (the
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
		f["timeline"] = {"min": int(app.TL_YEAR_MIN), "max": int(app.TL_YEAR_MAX)}
	return f


## [`facts`] as the JSON text `LlmStream`'s `args.actions` takes.
static func facts_json(app, bridge) -> String:
	return JSON.stringify(facts(app, bridge))
