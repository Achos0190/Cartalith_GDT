extends RefCounted
class_name CommandIndex

## Every action, parameter and export the app has, as one searchable list.
##
## Owner, 2026-08-30: *"I'm still not happy with how functions and options are
## made known to the user on android the interface is hard to understand and
## muddled."* This is Direction A of the phone redesign: one search field that
## answers "what can this thing do?" without the user already knowing the
## vocabulary.
##
## ## It is BUILT, not written down
##
## The same principle `ShortcutsDialog` uses, for the same reason. Three real
## tables are read at build time:
##
## - **Generation parameters** — `EngineBridge.param_keys()` and
##   `param_info(key)`, which are `cartalith-godot`'s own `ParamSpec` rows
##   (`params.rs`): key, group, label, unit, type, range, and the reference
##   control id. That is where the wording already lives, and
##   `world_workspace.gd`'s own comment says of it: *"nothing about the range,
##   step, label or unit is guessed here."* Neither is anything here.
## - **Menu commands** — walked off the live `MenuBar`, exactly as
##   `ShortcutsDialog` walks it, so a row added to a menu appears here with no
##   edit to this file.
## - **Declared extras** — [`EXTRAS`], the few real actions that are neither a
##   parameter nor a menu row.
##
## A hand-maintained catalogue of an app's own features is the most reliably
## stale document a project can own. This one cannot disagree with the app,
## because it is assembled from the app.
##
## ## Availability is part of the entry, not a surprise
##
## Every entry carries `available` and, when false, `why`. The old shell had
## ten disabled rows that looked identical to live ones — the exact defect
## `PARITY_AUDIT.md` §23 has been finding all week, and the reason the Edit
## menu spent months telling users deletion was impossible while the Delete key
## deleted. A row that cannot run says so where it is read.
##
## ## Three states, not two
##
## A disabled menu row is one of **three** things, and this file read it as two
## until 2026-08-31:
##
## - **chrome** — a signpost ("imports live under Data ▸ Import", the six
##   "Rows here read state" lines under Assets ▸ Landmark types) or a
##   placeholder ("— loading —"). Skipped entirely. `menus.gd::_signpost()`
##   marks the first kind with `DccMenus.META_SIGNPOST`; the rest are still
##   recognised the old way, by being disabled and silent.
## - **a readout** — disabled, tooltipped, and carrying a live *value*:
##   Working set, the pack schema line, the VRAM estimate, the two GPU memory
##   lines, "No recent projects". `menus.gd::_readout()` marks these with
##   `DccMenus.META_READOUT`.
## - **an unavailable command** — `menus.gd::_todo()`, and only that. A thing
##   the app will one day do and cannot yet.
##
## Only the third belongs in the list as `available: false`. Indexing the five
## readouts that way put entries in the list that a user could search for, tap,
## and get nothing from — the exact defect this header opens by describing,
## reproduced by the fix for it. They are kept, as `kind: "readout"`, because
## searching "memory" and being shown `Working set 1.6 GB of 12 GB` is the
## answer, not a dead end: the row IS the result.

## Actions that are neither a generation parameter nor a menu row. Deliberately
## short: anything that belongs in a menu should BE in a menu, where the walk
## finds it for free.
const EXTRAS: Array = [
	{"title": "Zoom to fit", "blurb": "Frame the whole world in the viewport", "group": "View"},
	{"title": "Point sample", "blurb": "Read elevation, biome and climate under one cell", "group": "View"},
]

var _rows: Array = []
var _bridge
var _app

func build(app, bridge) -> void:
	_app = app
	_bridge = bridge
	_rows.clear()
	_add_parameters()
	_add_menu_commands()
	for e in EXTRAS:
		_rows.append({
			"title": String(e.get("title", "")),
			"blurb": String(e.get("blurb", "")),
			"group": String(e.get("group", "Other")),
			"kind": "action",
			"available": true,
			"why": "",
			"key": "",
		})

## Generation parameters, straight off the engine's own spec table.
func _add_parameters() -> void:
	if _bridge == null or not _bridge.has_method("param_keys"):
		return
	var has_world: bool = _bridge.has_world
	for key in _bridge.param_keys():
		var info: Dictionary = _bridge.param_info(String(key))
		if info.is_empty():
			continue
		var unit := String(info.get("unit", ""))
		var group := String(info.get("group", "")).capitalize()
		var ref_ctrl := String(info.get("reference_control", ""))
		## The blurb says what the thing IS, using the engine's own wording
		## plus the one fact a searcher wants: is this a switch or a value,
		## and in what units.
		var kind := String(info.get("type", "float"))
		var blurb := ("A switch" if kind == "bool" else "A value")
		if unit != "":
			blurb += ", in " + unit
		if ref_ctrl != "":
			blurb += " · reference control #" + ref_ctrl
		_rows.append({
			"title": String(info.get("label", key)),
			"blurb": blurb,
			"group": group if group != "" else "Generation",
			"kind": "param",
			"key": String(key),
			## A parameter is always readable; it only *applies* on a
			## generate, which is a different statement from "unavailable".
			"available": true,
			"why": "" if has_world else "Takes effect on the next Generate",
		})

## Every menu row, walked off the live MenuBar. A `_todo` row is disabled and
## carries its reason as a tooltip -- both are read here, so the index reports
## the same truth the menu does rather than a second opinion.
func _add_menu_commands() -> void:
	if _app == null:
		return
	var buttons: Array = []
	_gather_menu_buttons(_app, buttons)
	for mb in buttons:
		var popup: PopupMenu = (mb as MenuButton).get_popup()
		if popup != null:
			_walk_popup(popup, String((mb as MenuButton).text))

func _walk_popup(popup: PopupMenu, menu_name: String) -> void:
	for i in popup.item_count:
		var text := popup.get_item_text(i)
		if text.strip_edges() == "" or popup.is_item_separator(i):
			continue
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_popup(node as PopupMenu, menu_name)
			continue
		var disabled := popup.is_item_disabled(i)
		var tip := popup.get_item_tooltip(i)
		## The readout marker, set by `menus.gd::_readout()`. Read off the
		## metadata rather than guessed from the text, because several of these
		## rows are rewritten wholesale on every `about_to_popup` and any test
		## over their wording would be a test over a moving string.
		var meta = popup.get_item_metadata(i)
		var marker := String(meta) if typeof(meta) == TYPE_STRING else ""
		var is_readout := marker == DccMenus.META_READOUT
		## **A signpost is prose, not a command.** Marked at the source rather
		## than inferred here, so a reworded sentence cannot turn one back into
		## an indexed action.
		if marker == DccMenus.META_SIGNPOST:
			continue
		## **A disabled row with no tooltip is chrome, not a command.**
		## `menus.gd::_todo(p, text, why)` ALWAYS sets a tooltip -- that is its
		## whole signature -- so a real not-built-yet row is never silent. What
		## is disabled and silent is a placeholder -- the Assets menu's
		## "-- loading --", which is the pack list mid-fetch. (The signposts
		## that used to be caught here are caught by the marker above; this
		## rule stays because a placeholder is not authored one row at a time.)
		## Indexing those as commands put entries in the list that a user could
		## search for, tap, and get nothing from.
		##
		## Found by the probe rather than by reading: it asserted every
		## unavailable row carries a reason and reported 26 of 29.
		if disabled and tip.strip_edges() == "":
			continue
		## A readout is available: it has already told the searcher what they
		## came for, in its own title. `why` stays empty for the same reason —
		## there is nothing being withheld to explain.
		_rows.append({
			"title": text.replace("…", "").strip_edges(),
			"blurb": tip if (disabled and tip != "") else (menu_name + " menu"),
			"group": menu_name,
			"kind": "readout" if is_readout else "menu",
			"key": "",
			"available": is_readout or not disabled,
			"why": tip if (disabled and not is_readout) else "",
		})

## `get_children(true)` -- a MenuButton keeps its popup as an INTERNAL child,
## so the default walk finds none of them. `_loddbg_probe.gd` cost a run
## learning that.
func _gather_menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather_menu_buttons(c, out)

func size() -> int:
	return _rows.size()

func all() -> Array:
	return _rows.duplicate()

## Substring match over title, blurb and group, case-insensitive.
##
## Deliberately not fuzzy. A searcher typing "riv" wants every river row, and a
## fuzzy matcher that also returns "Drive" and "Arrival" makes the list *less*
## trustworthy -- the whole point here is that what you see is what matched.
## Ordered: title matches first, then group, then blurb, each alphabetically,
## so the strongest reason a row is present is the reason it sorts.
func search(q: String) -> Array:
	var needle := q.strip_edges().to_lower()
	if needle == "":
		return _rows.duplicate()
	var t: Array = []
	var g: Array = []
	var b: Array = []
	for r in _rows:
		if String(r["title"]).to_lower().find(needle) >= 0:
			t.append(r)
		elif String(r["group"]).to_lower().find(needle) >= 0:
			g.append(r)
		elif String(r["blurb"]).to_lower().find(needle) >= 0:
			b.append(r)
	var by_title := func(a, c): return String(a["title"]) < String(c["title"])
	t.sort_custom(by_title)
	g.sort_custom(by_title)
	b.sort_custom(by_title)
	return t + g + b

## Distinct group names in the order they first appear, so a caller can band
## the list the way the phone canvas bands its own.
func groups() -> Array:
	var seen: Array = []
	for r in _rows:
		if not seen.has(r["group"]):
			seen.append(r["group"])
	return seen
