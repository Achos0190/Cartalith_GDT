## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, the Story rows (milestone PC-9; findings S2, W6, A1, A4, M3):
##   - "Political history of this place…" becomes "Political history of the selection" (the Inspect page).
##   - Faction history… opens on the current faction.
##   - "Notes for this place" becomes "Note for the selection" (Ctrl+Shift+N, the Note tab), any kind.
##   - "Assistant panel…" becomes "Ask the assistant about the selection" (Ctrl+J).
##   - Story's "Places without notes…" becomes a signpost to World ▸ Records (the id stays in World).
##
## Target: `shell/menus.gd`, the Story section. Depends on `01_menus_ids_helpers.gd` (`_host_has()`,
## `_context_tab_ready()`) and `02_menus_world.gd` (`_open_factions()`).
## Applies to: every form factor (the tablet's bar has Story; the phone re-presents it row for row in
## MENU ▸ Story, Ruling DX Q13, and band by band on the STORY tab through `phone_menu.gd::_fill_story`).
##
## Forms (each row keeps its id in both, so rebinds and pins follow it):
##   Political history  before PC-4: shipped ("…of this place…", the Place editor's Political tab,
##                      settlement only). From PC-4 (`context_tabs()` has "inspect"): "…of the
##                      selection", `open_context_tab("inspect", "history")`, any kind.
##   Faction history…   before PC-1: shipped (the last faction viewed). From PC-1 (`current_faction()`):
##                      on the current faction. The label does not change.
##   Note row           before PC-5: shipped ("Notes for this place", the vault window, settlement
##                      only). From PC-5 ("note"): "Note for the selection", `open_context_tab("note")`.
##   Assistant row      before PC-6: shipped ("Assistant panel…", no key). From PC-6 ("ask"): "Ask the
##                      assistant about the selection", Ctrl+J, `open_context_tab("ask")`.
##   Places without     a signpost from the moment this is pasted (no dependency).
##   notes
## **Why the shipped form, not a `_todo`, before each milestone:** the shipped rows work. Replacing
## "Notes for this place" with a disabled row would take away a working command and Ctrl+Shift+N, which
## `MISTAKES.md` ("Delete a surface") forbids without a stated home. `_todo` is for rows with no
## behaviour at all (Edit's Previous / Next selection, Window ▸ Context panel's rows).
##
## What to do:
##   REPLACE  `_story()` together with its doc block: from the line `## **The Story menu**
##            (`STORY_MENUS_PROPOSAL.md` §3.3, …` down to and including `p.id_pressed.connect(_on_story)`
##            (99 lines at a38e0ac), with the version below.
##   REPLACE  `_refresh_story_menu()` with its two-line doc (41 lines at a38e0ac), and `_on_story()` with
##            its one-line doc (30 lines), with the versions below.
##   ADD      the constants, `_story_forms` and the three functions in the next block, beside
##            `STORY_POLITICAL_TIP`. `STORY_NO_PLACE`, `STORY_BUSY`, `STORY_NO_WORLD`, the four `_story_*`
##            index variables, `STORY_POLITICAL_TIP`, `STORY_NOTES_TIP`, `PLACES_WITHOUT_NOTES_TIP` and
##            `_story_place()` stay exactly as they are.
##   PROSE    comments the change makes stale (`MISTAKES.md`, "Change behaviour"):
##            - `ID_PLACES_WITHOUT_NOTES` ("World ▸ / Story ▸ Places without notes…"), its block comment
##              ("Places without notes… is one id on two menus"), `PLACES_WITHOUT_NOTES_TIP`'s doc
##              ("shared by World ▸ RECORDS and Story ▸ NOTES") and `app.gd::open_places_without_notes`'s
##              doc ("World ▸ / Story ▸"): World only now; Story has a signpost.
##            - `ID_STORY_POLITICAL`, `ID_STORY_NOTES_HERE` and `ID_STORY_ASSISTANT`'s comments name the
##              shipped labels: add "(…of the selection / Note for the selection / Ask the assistant
##              about the selection once the Context panel has the tab)".
##            - `right_dock.gd::current_settlement_index`'s doc ("SM-2's Story rows … read it as 'the
##              current place'") is true of the shipped forms only.
##
## Verify: the parse check on `res://shell/menus.gd`; `check_symbols.sh`. Probes: `_storymenus_probe.gd`
## (STORY_ROWS: "Places without notes…" becomes the signpost text; drop its press of
## ID_PLACES_WITHOUT_NOTES on the Story popup; keep the World one). `_llmplace_probe.gd` and
## `_llmpanel_probe.gd` find "Assistant panel…" by title, which is still the row until PC-6, so they
## stay green now and PC-6 updates them. By hand: with nothing selected, Story's two gated rows are
## dark with STORY_NO_PLACE's reason while the menu is open, and Ctrl+Shift+N still answers after it
## closes (the status line says why); the NOTES band reads "Places without notes is in World ▸ Records".

## Why a PC-4/PC-5 form of a gated row is disabled: nothing is selected. Names what a selection is
## made from rather than the right dock, because from PC-1 the dock reads the selection.
const STORY_NO_SELECTION := "Nothing selected. Select a place, a faction or a conflict on the map or in a list first."
## Why a PC-4/PC-5 form of a gated row is disabled while a generation runs: the Inspect page and the
## note both read the engine, which the worker holds (the same gate as `STORY_BUSY`, worded for a
## selection of any kind).
const STORY_BUSY_SELECTION := "A generation is running. The selection is read when it finishes."
## "Political history of the selection"'s tooltip (PC-4). What `PC_SHELL_PROPOSAL.md` §3.3 says
## Inspect shows for each kind; the page itself is PC-4's to build.
const STORY_POLITICAL_SEL_TIP := "Opens the Context panel's Inspect page on the selection, at its history: who held a place year by year, the places a faction held and its wars, a conflict's years and sides. Every name there selects it and every year moves the timeline."
## "Note for the selection"'s tooltip (PC-5), from §3.4.
const STORY_NOTE_SEL_TIP := "Opens the Context panel's Note tab on the selection's own note, found through its link rather than its name. With no note yet it offers to create one from the kind's template, already linked. The whole vault is World ▸ Markdown vault… (Ctrl+Shift+V)."
## "Ask the assistant about the selection"'s tooltip (PC-6), from §3.5. Says nothing about whether a
## conversation survives a session: that is owner question PQ5, not yet answered.
const STORY_ASK_TIP := "Opens the Context panel's Ask tab with the cursor in its input. The chips above it show what goes with your next message (the selection, the year, its note), read when you press Send. Off until you turn it on in Preferences ▸ Assistant; nothing is sent until then."
## "Assistant panel…"'s tooltip: the shipped string, moved here unchanged.
const STORY_ASSISTANT_TIP := "Optional language-model help with prose for a place, its story, or a Vault note. Off until you turn it on in Preferences ▸ Assistant; saved only when you click."
## Faction history…'s tooltip before PC-1: the shipped string, moved here unchanged.
const FACTION_HISTORY_TIP := "Opens the Factions window on its History tab: each faction's recorded years, territory and conflicts."
## Faction history…'s tooltip once the host has `current_faction()` (finding W6). Says what that
## method's contract says (`90_pc_milestone_stubs.gd`).
const FACTION_HISTORY_CURRENT_TIP := "Opens the Factions window on its History tab, on the current faction: the selected faction, or the one that holds the selected place in the year the timeline shows; the last faction you viewed when nothing is selected."
## The signpost that replaces Story's second "Places without notes…" row (finding M3), in the
## mockup's own words (`design/mobile-shell-2026-10-31/Shell.dc.html`, the desktop Story menu).
const PLACES_WITHOUT_NOTES_SIGNPOST := "Places without notes is in World ▸ Records"

## Which form each of Story's two-form rows was built in: `true` is PC-9's row, `false` the shipped
## one. Written once by `_story()`; read by `_refresh_story_menu()` and `_on_story()`, so the label, the
## gate and the handler cannot disagree about which row is on the menu. Never re-derived at press
## time: the menu does not rebuild when a milestone's method appears, so asking again could pick a
## form the row's label does not show.
var _story_forms := {"political": false, "note": false, "ask": false}

## The selection for the PC-4/PC-5 forms of Story's gated rows: the host's `selection` (`{kind, id}`,
## PC-1), or `{why}` with the reason it cannot be used. Busy first, as `_story_place()` asks it, then no
## world, then nothing selected. Returns a copy, so a caller can never edit the host's selection.
## Reads `DccApp.selection` through `get()`, which answers null where the property does not exist, so
## a host without PC-1 gets the "nothing selected" reason rather than an error.
func _story_selection() -> Dictionary:
	if bool(_bridge.generating):
		return {"why": STORY_BUSY_SELECTION}
	if not bool(_bridge.has_world):
		return {"why": STORY_NO_WORLD}
	var sel = _host.get("selection")
	if typeof(sel) != TYPE_DICTIONARY or (sel as Dictionary).is_empty():
		return {"why": STORY_NO_SELECTION}
	return (sel as Dictionary).duplicate()

## The political-history row's tooltip for the form `_story()` built.
func _story_political_tip() -> String:
	return STORY_POLITICAL_SEL_TIP if bool(_story_forms["political"]) else STORY_POLITICAL_TIP

## The note row's tooltip for the form `_story()` built.
func _story_note_tip() -> String:
	return STORY_NOTE_SEL_TIP if bool(_story_forms["note"]) else STORY_NOTES_TIP

## ── REPLACEMENTS for `_story()`, `_refresh_story_menu()` and `_on_story()`, each with its doc ──────

## **The Story menu** (`STORY_MENUS_PROPOSAL.md` §3.3, Ruling CK, owner
## 2026-10-18; mockup `design/story-menus-2026-10-17/` A2). It gathers the
## tools a writer or a DM uses to plot -- time, journeys, conflicts, notes --
## which were scattered over a rail node, an `app.gd` key, a window region, a
## dock category and three windows. Four labelled bands, in the mockup's order.
##
## Every enabled row calls something already built; it is a new entry point,
## never a second implementation:
##   TIME       Timeline (Ctrl+Shift+T, `_open_timeline()`; World's copy is a
##              signpost since PC-9) · Political history of this place…
##              (`place_editor_window.gd::open_for_tab(i, "political")`), or of
##              the selection (PC-4: the Context panel's Inspect page) · Faction
##              history… (`_open_factions("history")`, FH-8's tab; on the
##              current faction from PC-1)
##   JOURNEYS   Journey planner (`app.open_journey_planner()`) · Saved
##              journeys… (SM-6, `saved_journeys_window.gd`) · Travel library…
##              (⇧L, moved here from World by this milestone, same id)
##   CONFLICTS  Conflicts… (SM-6, `conflicts_window.gd`) · Draw a conflict (arms CIVIL's
##   & CAMPAIGNS Conflict tool, SP-4) · Campaigns this year (a readout over
##              `conflict_campaigns(year)`, one row per active conflict)
##   NOTES      Notes for this place (Ctrl+Shift+N, `app.open_vault(
##              "settlement", tid, name)`, the card's own call), or Note for the
##              selection (PC-5: the Note tab) · a signpost to World ▸ Records'
##              Places without notes… (PC-9; SM-5 put a second row here) ·
##              Assistant panel… (LLM-3), or Ask the assistant about the
##              selection (Ctrl+J, PC-6: the Ask tab)
##
## **Journey planner carries no accelerator column, on purpose.** ⇧J is an
## `app.gd::_unhandled_key_input` key (it outlived the row Ruling 4 retired on
## 2026-09-05) and is listed in `shortcuts_dialog.gd::UNLISTED`; drawing it here
## through `_bind_accelerator` would make it look rebindable while `app.gd`
## kept answering the old key, and a bare `set_item_accelerator` is what this
## file's rule forbids. The tooltip names the key instead. This row reverses
## nothing in Ruling 4: the rail node stays; this is an additional entry point
## (§7 risk 4).
##
## **Campaigns this year has no world-wide readout anywhere else** --
## `conflict_campaigns` is read per selected siege in the right dock
## (`_build_siege_garrison`) and by the CARTO layer -- so the row is a
## `_readout()` whose text is counted off that same call for the Timeline
## cursor's year, refreshed on every popup. Nothing is invented: before a world
## or during a generation it says which, and never prints a count it did not read.
##
## **PC-9 (`PC_SHELL_PROPOSAL.md` §3.8): three rows have two forms**, decided
## once here and recorded in `_story_forms`, so the label, the gate
## (`_refresh_story_menu()`) and the handler (`_on_story()`) always agree about
## which row is on the menu. Each keeps its id, so a rebind (keyed by id in
## `DccSettings`) and a quick-bar pin (`menu:<id>`) follow it from one form to
## the other:
##   724  "Political history of this place…" (Place editor, a settlement only)
##        → "Political history of the selection" (Inspect, any kind) once the
##        Context panel has Inspect (PC-4)
##   728  "Notes for this place" (vault window, a settlement only) → "Note for
##        the selection" (the Note tab, any kind) once it has Note (PC-5);
##        Ctrl+Shift+N in both
##   901  "Assistant panel…" (its own window) → "Ask the assistant about the
##        selection", Ctrl+J, once it has Ask (PC-6)
## The shipped form stays until then because it works: a `_todo` in its place
## would take away a working row and Ctrl+Shift+N (`MISTAKES.md`, "Delete a
## surface"). The new labels drop the ellipsis, as the mockup does: they open a
## tab of the panel, not a window. Places without notes… is a signpost to
## World ▸ Records from now (finding M3); the row and id 731 are World's.
##
## Must never call the engine while `bridge.generating` (the worker holds
## `WorldGen`): every refresh and handler asks `generating` first, which matters
## for `conflict_campaigns` in particular because its bridge wrapper has no
## gate of its own.
func _story(p: PopupMenu) -> void:
	_story_popup = p
	_story_forms = {
		"political": _context_tab_ready("inspect"),
		"note": _context_tab_ready("note"),
		"ask": _context_tab_ready("ask"),
	}
	p.add_separator("TIME")
	_live(p, "Timeline", ID_STORY_TIMELINE, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_T)
	p.set_item_tooltip(p.item_count - 1, TIMELINE_TIP)
	if bool(_story_forms["political"]):
		_live(p, "Political history of the selection", ID_STORY_POLITICAL)
	else:
		_live(p, "Political history of this place…", ID_STORY_POLITICAL)
	_story_political_idx = p.item_count - 1
	p.set_item_tooltip(_story_political_idx, _story_political_tip())
	_live(p, "Faction history…", ID_STORY_FACTION_HISTORY)
	p.set_item_tooltip(p.item_count - 1,
		FACTION_HISTORY_CURRENT_TIP if _host_has("current_faction") else FACTION_HISTORY_TIP)

	p.add_separator("JOURNEYS")
	_live(p, "Journey planner", ID_STORY_JOURNEY_PLANNER)
	p.set_item_tooltip(p.item_count - 1,
		"Opens the Journey planner (also the CIVIL rail's Journey planner node, or Shift+J anywhere on the map): party, season, carriage, stages and cost over a drawn route.")
	_live(p, "Saved journeys…", ID_SAVED_JOURNEYS)
	p.set_item_tooltip(p.item_count - 1,
		"Lists every journey saved in this world (name, party, start year, length, mode); a row opens it in the Journey planner. Journeys are saved from the planner.")
	_live(p, "Travel library…", ID_TRAVEL_LIBRARY, KEY_MASK_SHIFT | KEY_L)
	p.set_item_tooltip(p.item_count - 1,
		"Animals, vehicles, vessels and parties: the classifications and constraints the Journey planner reads.")

	p.add_separator("CONFLICTS & CAMPAIGNS")
	_live(p, "Conflicts…", ID_CONFLICTS_LIST)
	p.set_item_tooltip(p.item_count - 1,
		"Lists every conflict in this world with its kind, years and sides, active or not in the Timeline cursor's year; a row selects it on the map and opens it in the right dock.")
	_live(p, "Draw a conflict", ID_STORY_DRAW_CONFLICT)
	p.set_item_tooltip(p.item_count - 1,
		"Arms the CIVIL Conflict tool: click to draw a front, an arrow, a siege or a battle, then Commit and fill in its sides and years in the right dock.")
	_story_campaigns_idx = _readout(p, "Campaigns this year", "")

	p.add_separator("NOTES")
	if bool(_story_forms["note"]):
		_live(p, "Note for the selection", ID_STORY_NOTES_HERE, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_N)
	else:
		_live(p, "Notes for this place", ID_STORY_NOTES_HERE, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_N)
	_story_notes_idx = p.item_count - 1
	p.set_item_tooltip(_story_notes_idx, _story_note_tip())
	## PC-9 (finding M3): SM-2's second "Places without notes…" row
	## (`STORY_MENUS_PROPOSAL.md` §3.3: "the same row as in World") is a
	## signpost to that row now. Its id, 731, stays
	## on World ▸ RECORDS, which the command index already preferred (it walks
	## World first and made this copy an unpinnable alias).
	_signpost(p, PLACES_WITHOUT_NOTES_SIGNPOST)
	## LLM-3. Always enabled, in both forms: the panel says why it cannot send
	## (off, no provider, no consent), which is the `CommandIndex` rule -- a
	## reason, never a dead row. Ctrl+J only with the Ask tab (PC-6): bound to
	## the floating window it would open a fresh conversation on every press
	## (`llm_assistant_window.gd::open` calls `_rebuild()`, finding A1).
	if bool(_story_forms["ask"]):
		_live(p, "Ask the assistant about the selection", ID_STORY_ASSISTANT, KEY_MASK_CTRL | KEY_J)
		p.set_item_tooltip(p.item_count - 1, STORY_ASK_TIP)
	else:
		_live(p, "Assistant panel…", ID_STORY_ASSISTANT)
		p.set_item_tooltip(p.item_count - 1, STORY_ASSISTANT_TIP)

	_refresh_story_menu()
	p.about_to_popup.connect(_refresh_story_menu)
	## The two gated rows are disabled only while the menu is OPEN. A popup's
	## accelerator is skipped while its row is disabled, so a row left disabled
	## after the menu closed would swallow Ctrl+Shift+N for good once the user
	## had opened Story with nothing selected. Re-enabled on close; the handler
	## re-checks (`_story_place()`, or `_story_selection()` for a PC-4/PC-5 form)
	## and says why in the status line instead.
	p.popup_hide.connect(func() -> void:
		for i in [_story_political_idx, _story_notes_idx]:
			p.set_item_disabled(i, false))
	p.id_pressed.connect(_on_story)

## Refreshes the Story menu's live rows: the two gated rows (enabled, and
## tooltipped, only while their subject can be read: a settlement in the
## shipped forms, anything selected in the PC-4/PC-5 forms; `_story_forms`
## says which) and the campaigns readout.
func _refresh_story_menu() -> void:
	var p := _story_popup
	if p == null:
		return
	var place := _story_place()
	var subject := _story_selection()
	for row in [[_story_political_idx, bool(_story_forms["political"]), _story_political_tip()],
			[_story_notes_idx, bool(_story_forms["note"]), _story_note_tip()]]:
		var gate: Dictionary = subject if bool(row[1]) else place
		var ok := not gate.has("why")
		p.set_item_disabled(int(row[0]), not ok)
		p.set_item_tooltip(int(row[0]), String(row[2]) if ok else String(gate["why"]))
	var text := "Campaigns this year"
	var tip := ""
	if bool(_bridge.generating):
		text += " — generating"
		tip = "A generation is running; the conflict store is read when it finishes."
	elif not bool(_bridge.has_world):
		text += " — no world"
		tip = STORY_NO_WORLD
	elif not _engine_has("conflict_campaigns"):
		text += " — not in this build"
		tip = "This build of the engine has no conflict_campaigns binding."
	else:
		var year := int(_bridge.get_civ_year())
		var rows: Array = _bridge.conflict_campaigns(year)
		## A conflict's `name` can be blank until the user fills it in (the
		## Conflict tool commits first, names later), so the id stands in.
		var names: Array = []
		for r in rows:
			var nm := String((r as Dictionary).get("name", ""))
			names.append(nm if nm != "" else "#%d" % int((r as Dictionary).get("id", 0)))
		text = "Campaigns in year %d: %d" % [year, rows.size()]
		if names.is_empty():
			tip = "No conflict is active in the Timeline cursor's year. "
		else:
			## Six names keep the tooltip to a few lines; the rest are counted.
			tip = "Conflicts active in the Timeline cursor's year: %s" % ", ".join(names.slice(0, 6))
			tip += (", and %d more. " % (names.size() - 6)) if names.size() > 6 else ". "
		tip += "The CARTO Conflict layer draws them; select one under CIVIL ▸ Military ▸ Conflicts for its sides and manpower."
	p.set_item_text(_story_campaigns_idx, text)
	p.set_item_tooltip(_story_campaigns_idx, tip)

## The Story menu's dispatcher. Ignores every id it does not own. Each row
## with two forms acts on the form `_story()` built (`_story_forms`), so the
## handler cannot do something the row's label did not say.
func _on_story(id: int) -> void:
	match id:
		ID_STORY_TIMELINE: _open_timeline()
		ID_STORY_FACTION_HISTORY: _open_factions("history")
		ID_STORY_JOURNEY_PLANNER: _host.open_journey_planner()
		ID_TRAVEL_LIBRARY: _host.open_travel_library()
		## No `ID_PLACES_WITHOUT_NOTES` arm since PC-9: Story's row is a signpost,
		## and World's row is dispatched by `_on_data()`.
		ID_SAVED_JOURNEYS: _host.open_saved_journeys()
		ID_STORY_ASSISTANT:
			if bool(_story_forms["ask"]):
				_host.open_context_tab("ask")
			else:
				_host.open_assistant_panel()
		ID_CONFLICTS_LIST: _host.open_conflicts_list()
		ID_STORY_DRAW_CONFLICT:
			if not bool(_bridge.has_world):
				_host.set_status("hint", STORY_NO_WORLD, "text_ghost")
				return
			## `select_domain` first: the Conflict tool belongs to the CIVIL
			## workspace, whose `_on_civ_tool_armed` fills the options row
			## only while CIVIL is the domain (the phone's Civilization screen
			## armed its tools with the same two calls, in the same order).
			_shell.select_domain("civilization")
			_host.arm_tool("conflict")
		ID_STORY_POLITICAL, ID_STORY_NOTES_HERE:
			if bool(_story_forms["political" if id == ID_STORY_POLITICAL else "note"]):
				var subject := _story_selection()
				if subject.has("why"):
					_host.set_status("hint", String(subject["why"]), "text_ghost")
					return
				if id == ID_STORY_POLITICAL:
					_host.open_context_tab("inspect", "history")
				else:
					_host.open_context_tab("note")
				return
			var place := _story_place()
			if place.has("why"):
				_host.set_status("hint", String(place["why"]), "text_ghost")
				return
			if id == ID_STORY_POLITICAL:
				_host.place_editor_window.open_for_tab(int(place["index"]), "political")
			else:
				_host.open_vault("settlement", int(place["tid"]), String(place["name"]))
