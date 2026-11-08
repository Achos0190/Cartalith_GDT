extends RefCounted

## PC-10 (`PC_SHELL_PROPOSAL.md` §3.3 "Modules: tabs you arrange", milestone PC-10; Ruling DY, PQ1):
## the ten modules the user can add to the Context panel as tabs, and the builder each one runs.
##
## **What it does.** `registry()` answers `DccApp.context_module_registry()`'s contract
## (`drafts/pc-shell/90_pc_milestone_stubs.gd`): one Dictionary per module with its key and title
## (`DccMenus.CTX_MODULES`' spelling, never a second one), the selection kinds it shows for, whether it
## also shows with nothing selected, its builder `Callable(parent: Control, subject: Dictionary,
## year: int)`, and the sentence it shows when the subject has nothing for it. The panel
## (`right_dock.gd`'s module tabs) and a module's own window (`context_module_window.gd`) call the same
## builder, so a module has one body and two hosts.
##
## **One builder, two hosts, and never a second implementation** (§3.3; `MISTAKES.md`, "Re-frame or
## redesign a window": start from its own builder). Every module whose content already ships in a
## window calls THAT window's builder function, on a detached instance of the window's own script whose
## subject is set to the panel's (`_place_host()`, `_roster_host()`): the window's tab and the module are
## the two call sites of one definition. The detached instance is never shown; it is a hidden child of
## the content it built and lives exactly as long (`_bind_lifetime()`), and an edit made through its
## controls that would rebuild the window instead refreshes the module (`module_refresh`, a hook both
## windows read first in their rebuild paths). A labelled judgement (PC-10, 2026-11-08): the
## alternative, swapping the open window's own subject for the length of a build, would leave every
## control the module drew editing whatever the window shows by the time it is pressed.
##
## **What it must never do.** Write the arrangement (that is `DccApp`'s, through `DccSettings`); keep a
## copy of the selection (it is handed the subject per build); show a plausible value for one that is
## absent (each builder it calls already dashes or explains, and a kind a module cannot draw gets its
## `empty` sentence, never a blank tab).

## `DccApp`, set by `_init`. Read for the bridge, the selection history, the bookmarks and the windows.
var app
## The module specs: key, kinds, without_selection, empty. Titles come from `DccMenus.CTX_MODULES` in
## `registry()`, so a module's tab, its menu row and this table cannot spell it two ways.
##
## Kinds are §3.3's table "Shown for" column, verbatim, in `DccApp.selection`'s vocabulary. "Here" and
## "Recent & bookmarks" are the two with `without_selection` (the table: "the cursor when nothing is
## selected", "always"). Links' row says "any subject with a note": the vault binds notes to the kinds
## `cartalith_vault::EntityKind::parse` names that are also selection kinds (settlement, faction,
## culture, landmark), read at the symbol on 2026-11-08; a conflict and a note have no binding.
const SPECS := [
	{"key": "history", "kinds": ["settlement", "faction", "landmark"], "without_selection": false,
		"empty": "Nothing dated for this subject: no recorded year, no period of rule and no note with dated entries."},
	{"key": "economy", "kinds": ["settlement", "faction"], "without_selection": false,
		"empty": "No economy is recorded for this subject."},
	{"key": "military", "kinds": ["faction", "settlement"], "without_selection": false,
		"empty": "No military figures for this subject."},
	{"key": "faith", "kinds": ["faction", "culture"], "without_selection": false,
		"empty": "No relations, faith or culture recorded for this subject."},
	{"key": "territory", "kinds": ["faction"], "without_selection": false,
		"empty": "This faction claims and holds nothing."},
	{"key": "city", "kinds": ["settlement"], "without_selection": false,
		"empty": "No layout for this settlement."},
	{"key": "here", "kinds": ["settlement", "landmark"], "without_selection": true,
		"empty": "Nothing to read here: no world has been generated."},
	{"key": "routes", "kinds": ["settlement", "landmark"], "without_selection": false,
		"empty": "No road and no saved journey reaches this subject."},
	{"key": "links", "kinds": ["settlement", "faction", "culture", "landmark"], "without_selection": false,
		"empty": "No note links to this subject."},
	{"key": "recent", "kinds": ["settlement", "landmark", "faction", "culture", "conflict", "note"],
		"without_selection": true,
		"empty": "Nothing selected yet this session, and no bookmark."},
]

## How far a road's end may be from a place, in grid cells, for Routes & journeys to list it as a road
## "from the subject". A labelled judgement (PC-10, 2026-11-08): `bridge.roads()` carries a road's
## smoothed points and no endpoint settlement, and the smoothing moves an end by up to a cell, so two
## cells catches a road that ends at the place and not one that only passes its neighbour.
const ROAD_END_CELLS := 2.0

## The most roads Routes & journeys lists for one subject. A labelled judgement: a capital on a large
## world ends a few dozen ways at most; past this the section says how many it did not draw.
const ROAD_ROWS_CAP := 24

func _init(a) -> void:
	app = a

## The registry `DccApp.context_module_registry()` returns (see this file's header). Built per call:
## the Callables are bound to this object, which `DccApp` keeps for the session.
func registry() -> Array:
	var out: Array = []
	for spec in SPECS:
		var key := String(spec["key"])
		out.append({
			"key": key,
			"title": _title(key),
			"kinds": PackedStringArray(spec["kinds"]),
			"without_selection": bool(spec["without_selection"]),
			"builder": Callable(self, "build_" + key),
			"empty": String(spec["empty"]),
		})
	return out

## The module's title, `DccMenus.CTX_MODULES`' spelling. "" for a key that is not a module.
static func _title(key: String) -> String:
	for m in DccMenus.CTX_MODULES:
		if String(m[0]) == key:
			return String(m[1])
	return ""

## The spec for `key`, or `{}`.
static func spec(key: String) -> Dictionary:
	for s in SPECS:
		if String(s["key"]) == key:
			return s
	return {}

## Whether module `key` is shown for `subject` (a `DccApp.selection` value; `{}` is no subject):
## its kinds hold the subject's kind, or there is no subject and it shows without one. A module that
## does not fit is hidden from the strip, never removed from the arrangement (§3.3, "Fit").
static func fits(key: String, subject: Dictionary) -> bool:
	var s := spec(key)
	if s.is_empty():
		return false
	var kind := String(subject.get("kind", ""))
	if kind == "":
		return bool(s["without_selection"])
	return (s["kinds"] as Array).has(kind)

# -- Hosts: the windows' own scripts, detached ------------------------------------------------------

## Makes `host` a hidden child of `parent`, so the detached window instance lives exactly as long as the
## controls it built (their bound callables point at it), is freed with them, and is inside the tree for
## any builder that asks for it (a confirmation parented to it, `get_tree()`). A hidden `Window` child is
## not drawn and takes no input; a container lays out only `Control` children, so it takes no room.
func _bind_lifetime(parent: Control, host: Window) -> void:
	host.visible = false
	parent.add_child(host)

## A detached `PlaceEditorWindow` on settlement `index`: the Place editor's own builders run against it.
## `_body` is an orphan box, so an internal `_rebuild()` that runs anyway has somewhere to draw; the
## `module_refresh` hook sends it to the panel instead (`place_editor_window.gd::_rebuild()`).
func _place_host(parent: Control, index: int) -> PlaceEditorWindow:
	var h := PlaceEditorWindow.new()
	h.visible = false
	h.app = app
	h.bridge = app.bridge
	h._index = index
	h._body = VBoxContainer.new()
	h.add_child(h._body)
	h.module_refresh = func() -> void: app.refresh_context_modules()
	## An edit made here is announced through the window's own signals, so the listeners the CIVIL
	## workspace connected to the real window (the map, the dock, the territory) hear it the same way.
	var real: PlaceEditorWindow = app.place_editor_window
	if real != null:
		h.place_changed.connect(real.place_changed.emit)
		h.place_deleted.connect(real.place_deleted.emit)
		h.polity_moved.connect(real.polity_moved.emit)
	_bind_lifetime(parent, h)
	return h

## A detached `FactionRosterWindow` on faction `fid`, as `_place_host()`: its list, overview and
## inspector boxes are orphans so an internal rebuild draws nowhere visible, and `module_refresh`
## refreshes the panel (`faction_roster_window.gd::_rebuild()` and its narrower rebuilds).
func _roster_host(parent: Control, fid: int) -> FactionRosterWindow:
	var h := FactionRosterWindow.new()
	h.visible = false
	h.app = app
	h.bridge = app.bridge
	h._selected = fid
	h._list_body = VBoxContainer.new()
	h._inspector_body = VBoxContainer.new()
	h._overview = Label.new()
	for c in [h._list_body, h._inspector_body, h._overview]:
		h.add_child(c)
	h.module_refresh = func() -> void: app.refresh_context_modules()
	## As `_place_host()`: the window's own signals carry an edit to the listeners of the real window.
	var real: FactionRosterWindow = app.faction_roster_window
	if real != null:
		h.roster_changed.connect(real.roster_changed.emit)
		h.tariff_changed.connect(real.tariff_changed.emit)
		h.belief_run_requested.connect(real.belief_run_requested.emit)
	_bind_lifetime(parent, h)
	return h

## The settlement the subject names, `{}` when it is not a settlement or the index is gone.
func _settlement(subject: Dictionary) -> Dictionary:
	if String(subject.get("kind", "")) != "settlement":
		return {}
	var all: Array = app.bridge.settlements()
	var i := int(subject.get("id", -1))
	return all[i] if i >= 0 and i < all.size() else {}

## The faction dictionary the subject names, `{}` when it is not a faction or the id is gone.
func _faction(subject: Dictionary) -> Dictionary:
	if String(subject.get("kind", "")) != "faction":
		return {}
	var fid := int(subject.get("id", -1))
	for f in app.bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == fid:
			return f
	return {}

## The landmark dictionary the subject names, `{}` when it is not a landmark or the index is gone.
func _landmark(subject: Dictionary) -> Dictionary:
	if String(subject.get("kind", "")) != "landmark":
		return {}
	var lms: Array = app.bridge.landmarks()
	var i := int(subject.get("id", -1))
	return lms[i] if i >= 0 and i < lms.size() else {}

## A note in the module's own voice, for a kind its window has no builder for.
func _note(parent: Control, text: String) -> void:
	var l := DccWidgets.note(parent, text)
	l.name = "ModuleNote"

# -- The ten builders. Each takes (parent, subject, year); `year` is the strip's (`tl_year()`). ------

## History. A settlement: the Place editor's Political history tab (`_build_tab_political`: the
## timeline strip, holders by period, authored events, journeys passing, conflicts here). A faction:
## the Factions window's History tab (`_build_tab_history`). A landmark: the vault's chronos index reads
## only settlement and faction notes (`dcc_shell.gd::tl_selection_events()`), so it says that.
func build_history(parent: Control, subject: Dictionary, _year: int) -> void:
	match String(subject.get("kind", "")):
		"settlement":
			var s := _settlement(subject)
			if not s.is_empty():
				var h := _place_host(parent, int(subject["id"]))
				h._build_tab_political(parent, s)
				return
		"faction":
			if not _faction(subject).is_empty():
				_roster_host(parent, int(subject["id"]))._build_tab_history(parent)
				return
		"landmark":
			_note(parent, "A landmark has no dated history in this build: its note's dated entries are not indexed (the chronos index reads settlement and faction notes only), and the engine records no event for a landmark.")
			return
	_note(parent, String(spec("history")["empty"]))

## Economy. A settlement: the Place editor's Economy & notables tab (`_build_trade`). A faction: the
## Factions window's Economy tab (`_build_tab_economy`: readout, currency, tariffs, gaps).
func build_economy(parent: Control, subject: Dictionary, _year: int) -> void:
	match String(subject.get("kind", "")):
		"settlement":
			if not _settlement(subject).is_empty():
				_place_host(parent, int(subject["id"]))._build_trade(parent)
				return
		"faction":
			if not _faction(subject).is_empty():
				_roster_host(parent, int(subject["id"]))._build_tab_economy(parent)
				return
	_note(parent, String(spec("economy")["empty"]))

## Military. A faction: the Factions window's Military tab (`_build_military_block`: figures, garrisons,
## conflicts). A settlement: its walls (the Place editor's override) and its garrison at the shown
## year, through the same `CivilizationWorkspace.garrison_row()` the Inspect page's Garrison row reads.
func build_military(parent: Control, subject: Dictionary, year: int) -> void:
	match String(subject.get("kind", "")):
		"faction":
			if not _faction(subject).is_empty():
				_roster_host(parent, int(subject["id"]))._build_military_block(parent)
				return
		"settlement":
			var s := _settlement(subject)
			if not s.is_empty():
				var sec := DccWidgets.section(parent, "Military")
				## The Place editor's Walls override (`_build_urban`'s chooser, the same three words):
				## -1 is its stored "no override", which is Auto, never "no walls".
				var walls := int(app.bridge.civ_settlement_details(int(subject["id"])).get("walls", -1))
				var wl := DccWidgets.note(sec, "Walls: %s" % ("Auto (the layout decides)" if walls < 0
					else ("No fortifications" if walls == 0 else "Fortified")))
				wl.name = "ModuleWalls"
				var gr := CivilizationWorkspace.garrison_row(
					app.bridge.civ_settlement_garrison(int(s.get("tid", 0)), year), year)
				var l := DccWidgets.note(sec, "Garrison in %d: %s" % [year, String(gr["value"])])
				l.name = "ModuleGarrison"
				l.tooltip_text = String(gr["why"])
				return
	_note(parent, String(spec("military")["empty"]))

## Faith, culture & relations. A faction: the Factions window's Relations tab (`_build_tab_relations`,
## with its faith group) and its culture card (`_build_culture_card`, the Identity tab's). A culture:
## the same culture card, for the culture the subject's id indexes in `bridge.get_cultures()`.
func build_faith(parent: Control, subject: Dictionary, _year: int) -> void:
	match String(subject.get("kind", "")):
		"faction":
			var d := _faction(subject)
			if not d.is_empty():
				var h := _roster_host(parent, int(subject["id"]))
				h._build_tab_relations(parent)
				var sec := DccWidgets.section(parent, "Culture")
				## The culture is a subject of its own (PC-4 follow-up): a link row selects it through
				## `DccApp.select()`, whose Inspect page and Faith module draw this same card.
				var ckey := String(d.get("culture", "common"))
				var ci: int = app.culture_index(ckey)
				if ci >= 0:
					var cname := String((app.bridge.get_cultures()[ci] as Dictionary).get("name", ckey.capitalize()))
					var cb := DccWidgets.action(sec, "Select culture: %s" % cname,
						func() -> void: app.select("culture", ci))
					cb.name = "ModuleCultureLink"
					cb.tooltip_text = "Selects the culture; Inspect shows its profile."
					cb.set_meta("ctx_target", {"kind": "culture", "id": ci})
				h._build_culture_card(sec, ckey)
				return
		"culture":
			var cs: Array = app.bridge.get_cultures()
			var i := int(subject.get("id", -1))
			if i >= 0 and i < cs.size():
				var sec := DccWidgets.section(parent, "Culture")
				_roster_host(parent, -1)._build_culture_card(sec, String((cs[i] as Dictionary).get("key", "")))
				return
	_note(parent, String(spec("faith")["empty"]))

## Territory. A faction: the Factions window's Territory tab (`_build_tab_territory`).
func build_territory(parent: Control, subject: Dictionary, _year: int) -> void:
	var d := _faction(subject)
	if d.is_empty():
		_note(parent, String(spec("territory")["empty"]))
		return
	_roster_host(parent, int(subject["id"]))._build_tab_territory(parent, d)

## City layout. A settlement: the Place editor's Layout tab (`_build_layout`), then City viewer…
## (`DccApp.open_city_viewer`, the window that draws the layout full size).
func build_city(parent: Control, subject: Dictionary, _year: int) -> void:
	var s := _settlement(subject)
	if s.is_empty():
		_note(parent, String(spec("city")["empty"]))
		return
	var idx := int(subject["id"])
	_place_host(parent, idx)._build_layout(parent, s)
	var b := DccWidgets.action(parent, "City viewer…", func() -> void: app.open_city_viewer(idx))
	b.name = "ModuleCityViewer"
	b.tooltip_text = "Opens this settlement in the City viewer."

## Here. The terrain readout, the right dock's own Sample builder (`RightDock.build_here()`, which runs
## `_build_sample` and fills it at the subject's cell, or leaves it live on the cursor with nothing
## selected).
func build_here(parent: Control, subject: Dictionary, _year: int) -> void:
	if not bool(app.bridge.has_world):
		_note(parent, String(spec("here")["empty"]))
		return
	app.right_dock_ctrl.build_here(parent, subject)

## Routes & journeys. The roads that end at the subject (rows that open the road's reading in Inspect,
## the right dock's Route context, `RightDock.show_route()`), the saved journeys through a settlement
## (the Place editor's `_build_journey_passes`), and Plan a journey from here.
func build_routes(parent: Control, subject: Dictionary, _year: int) -> void:
	var at := Vector2.INF
	var s := _settlement(subject)
	var lm := _landmark(subject)
	if not s.is_empty():
		at = Vector2(float(s.get("x", 0)), float(s.get("y", 0)))
	elif not lm.is_empty() and lm.has("x") and lm.has("y"):
		at = Vector2(float(lm["x"]), float(lm["y"]))
	if at == Vector2.INF:
		_note(parent, String(spec("routes")["empty"]))
		return
	var sec := DccWidgets.section(parent, "Roads from here")
	sec.name = "ModuleRoads"
	var rows := 0
	var found := 0
	for r in app.bridge.roads():
		var pts: PackedVector2Array = (r as Dictionary).get("points", PackedVector2Array())
		if pts.size() < 2:
			continue
		if pts[0].distance_to(at) > ROAD_END_CELLS and pts[pts.size() - 1].distance_to(at) > ROAD_END_CELLS:
			continue
		found += 1
		if rows >= ROAD_ROWS_CAP:
			continue
		rows += 1
		var entry: Dictionary = r
		var nm := String(entry.get("name", ""))
		var km := float(entry.get("km", 0.0))
		var label := "%s · %s" % [nm if nm != "" else String(entry.get("way_type", "road")).capitalize(),
			DccUnits.format(km, 1) if km > 0.0 else "%d points" % pts.size()]
		var b := DccWidgets.action(sec, label, func() -> void: app.right_dock_ctrl.show_route(entry, "road"))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = "Shows this road's reading in Inspect."
		b.set_meta("ctx_module_road", true)
	if found == 0:
		DccWidgets.note(sec, "No road ends here.")
	elif found > rows:
		DccWidgets.note(sec, "%d more roads end here and are not listed (the first %d are)." % [found - rows, rows])
	if not s.is_empty():
		_place_host(parent, int(subject["id"]))._build_journey_passes(parent, s)
	var is_sel: bool = app.selection == subject and not s.is_empty()
	var plan := DccWidgets.action(parent, "Plan a journey from here", func() -> void:
		app.open_journey_planner()
		if app.journey_planner_view != null:
			app.journey_planner_view._stop_start())
	plan.name = "ModulePlanFromHere"
	plan.disabled = not is_sel
	plan.tooltip_text = "Opens the Journey planner and starts the journey here (its Start here)." if is_sel \
		else ("Select this settlement first: the planner starts from the selection." if not s.is_empty()
			else "A journey starts from a settlement; a landmark is not a stop.")

## Links. The notes that link to the subject and its unlinked mentions: the Place editor's
## `_build_backlinks` (the vault crate's link index), which the Factions window and the province rows
## already call with their own kind. A settlement is addressed by its `tid`, the binding the vault keeps.
func build_links(parent: Control, subject: Dictionary, _year: int) -> void:
	var kind := String(subject.get("kind", ""))
	var id := int(subject.get("id", -1))
	var nm := ""
	var vault_id := id
	match kind:
		"settlement":
			var s := _settlement(subject)
			if s.is_empty() or int(s.get("tid", 0)) == 0:
				_note(parent, "This settlement has no stable id yet, so no note can be bound to it.")
				return
			vault_id = int(s["tid"])
			nm = String(s.get("name", ""))
		"faction":
			var d := _faction(subject)
			if d.is_empty():
				_note(parent, String(spec("links")["empty"]))
				return
			nm = String(d.get("name", ""))
		"culture", "landmark":
			nm = String(app.right_dock_ctrl.subject_name(subject))
		_:
			_note(parent, String(spec("links")["empty"]))
			return
	var sec := DccWidgets.section(parent, "Links")
	sec.name = "ModuleLinks"
	_place_host(parent, -1)._build_backlinks(sec, kind, vault_id, nm)

## Recent & bookmarks. The selection history as a list (session-only, `DccApp._selection_back` and
## `_selection_forward`; never saved), then the bookmarks (saved inside the world file, Ruling DY:
## `DccApp.context_bookmarks`, `bookmarks_document()`), each name a link that selects it through
## `DccApp.select()`. The subject can be bookmarked or unbookmarked here.
func build_recent(parent: Control, subject: Dictionary, _year: int) -> void:
	var dock = app.right_dock_ctrl
	var sec := DccWidgets.section(parent, "Recent")
	sec.name = "ModuleRecent"
	var hist: Array = []
	for e in app._selection_back:
		hist.append(e)
	if not app.selection.is_empty():
		hist.append(app.selection)
	for i in range(app._selection_forward.size() - 1, -1, -1):
		hist.append(app._selection_forward[i])
	## Newest first, each subject once, the empty steps (a Deselect) left out: a list of places to go
	## back to, not a log of clicks.
	var seen := {}
	var drawn := 0
	for i in range(hist.size() - 1, -1, -1):
		var e: Dictionary = hist[i]
		if e.is_empty():
			continue
		var k := "%s:%d" % [String(e.get("kind", "")), int(e.get("id", -1))]
		if seen.has(k):
			continue
		seen[k] = true
		drawn += 1
		_subject_link(sec, e, dock, "RecentRow")
	if drawn == 0:
		DccWidgets.note(sec, "Nothing selected yet this session.")
	var bsec := DccWidgets.section(parent, "Bookmarks")
	bsec.name = "ModuleBookmarks"
	if not subject.is_empty():
		var marked: bool = app.is_bookmarked(subject)
		var b := DccWidgets.action(bsec, ("★ Remove bookmark: %s" if marked else "☆ Bookmark %s") % dock.subject_name(subject),
			func() -> void: app.toggle_bookmark(subject))
		b.name = "ModuleBookmarkToggle"
		b.tooltip_text = "Bookmarks are saved inside the world file."
	var marks: Array = app.context_bookmarks
	if marks.is_empty():
		DccWidgets.note(bsec, "No bookmark in this world.")
	for m in marks:
		_subject_link(bsec, m, dock, "BookmarkRow")

## One subject as a link row: its name (read fresh, so a renamed place reads its new name) selecting it.
## A subject the world no longer holds is drawn dim and not pressable, with its stored name.
func _subject_link(parent: Control, e: Dictionary, dock, node_name: String) -> void:
	var nm: String = dock.subject_name(e)
	var gone := nm == ""
	if gone:
		nm = String(e.get("name", "%s #%d" % [String(e.get("kind", "")), int(e.get("id", -1))]))
	var b := DccWidgets.action(parent, "%s  · %s" % [nm, String(e.get("kind", "")).capitalize()],
		func() -> void: app.select(String(e.get("kind", "")), int(e.get("id", -1))))
	b.name = node_name
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.disabled = gone
	b.tooltip_text = "No longer in this world." if gone else "Selects it."
	b.set_meta("ctx_target", {"kind": String(e.get("kind", "")), "id": int(e.get("id", -1))})
