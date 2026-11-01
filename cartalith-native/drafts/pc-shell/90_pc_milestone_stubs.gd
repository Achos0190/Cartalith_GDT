## DRAFT, not loaded by Godot. NOT parse-checked. **NOT TO BE PASTED BEFORE ITS MILESTONE.**
##
## The contract the PC menu drafts (01-09) call into, for state the proposal adds and the shell does not
## have yet: PC-1 (one selection, `PC_SHELL_PROPOSAL.md` §3.1 and §5), PC-4 (the Context panel, §3.3, and
## since Ruling DY its modules), PC-5 (its Note tab, §3.4), PC-6 (its Ask tab, §3.5) and PC-8 (the planner
## in its own window, Ruling DY). Target: `shell/app.gd` (`DccApp`). Every name here was free in `shell/` at
## `a38e0ac` (the Ruling DY names at `3099309`); `check_symbols.sh` re-checks that.
##
## **Why a contract and not code.** The menu drafts choose each row's form by asking whether the host
## has the method the new form calls (`DccMenus._host_has()`) or the tab it opens
## (`DccMenus._context_tab_ready()`). A placeholder body pasted before its milestone would make that
## answer "yes" over nothing. Each body below therefore answers "nothing" honestly (`false`, `-1`,
## an empty list), so a premature paste lights no Context panel row and makes Previous selection say
## "No earlier selection to go back to", which is true. But it would still switch Deselect's tooltip
## and the Factions rows' tooltips to their PC-1 wording, which would be false. So: **each milestone
## writes the real body in the same change that builds the state behind it, and deletes the
## `STUB(PC-n)` marker line.** `check_symbols.sh` fails while any `STUB(PC-` marker is in the shipped tree.
##
## Which menu rows each method switches on (form = which row the menu builds; see each draft's header):
##   PC-1  select_back            Edit ▸ Previous selection (Alt+←); Deselect's tooltip
##         select_forward         Edit ▸ Next selection (Alt+→)
##         current_faction        Records ▸ Factions… and Story ▸ Faction history… open on it
##         selection              read by Story's PC-4/PC-5 forms (`_story_selection()`)
##   PC-4  context_tabs ∋ inspect the Context panel label, Window ▸ Context panel ▸ Inspect (Ctrl+1),
##                                Pin, Follow; Story ▸ Political history of the selection; the panel's
##                                row in Window ▸ Open windows
##         open_context_tab, pin_context_panel, follow_selection: called by those rows
##         context_module_registry ∋ key   Window ▸ Context panel ▸ MODULES ▸ that module's check row
##                                (Ruling DY; desktop and tablet)
##         context_panel_modules, set_context_module_shown: read and called by those rows
##         reset_context_panel    Window ▸ Context panel ▸ Reset panel
##   PC-5  context_tabs ∋ note    Window ▸ Context panel ▸ Note (Ctrl+2); Story ▸ Note for the selection
##   PC-6  context_tabs ∋ ask     Window ▸ Context panel ▸ Ask (Ctrl+3); Story ▸ Ask the assistant about
##                                the selection (Ctrl+J)
##   PC-8  planner_in_window      Story ▸ Journey planner's tooltip says "its own window" (Ruling DY, PQ4)
##
## Not here, because no menu draft calls it: `DccApp.select(kind, id, opts)`, the one path every producer
## calls (§3.1, §5), and `selection_changed`. They are PC-1's core; `select_back()` and `select_forward()`
## below are defined in their terms, so their contract is written in the same place for the reader.
##
## Verify, per milestone: the menu rows above show their new form on the next boot (menus are built
## once); `check_symbols.sh` reports the method as present and no `STUB(PC-` marker left.

## PC-1. The one selection (`PC_SHELL_PROPOSAL.md` §2 rule 1): `{"kind": String, "id": int}`, kind one
## of "settlement", "landmark", "faction", "culture", "conflict", "note"; `{}` when nothing is selected.
## Written only by `select()`, `select_back()`, `select_forward()` and `clear_selection()`; every
## surface reads it and listens to `selection_changed`, none keeps a copy (§2: "No surface keeps a
## copy"). Never a plausible default (no `{"kind": "settlement", "id": 0}` for "none"): absent is `{}`.
## `menus.gd::_story_selection()` reads it through `get("selection")` and copies it.
var selection: Dictionary = {}

## PC-1. Emitted after `selection` changes, never before (`MISTAKES.md`, "Emit a change signal": a
## synchronous listener must read the new value), with the new value. The argument is not named
## `selection`, so it cannot be read as shadowing the property.
signal selection_changed(current: Dictionary)

## PC-1. The one path every producer calls (a map click, a Find hit, the context card, a window row, a
## link in Inspect, a `[[link]]` in a note, a citation in an answer: §3.1 "Producers"). Records the
## previous selection in the history unless `opts.get("history", true)` is false (back and forward pass
## false), drops any forward history when it records, then sets `selection` and emits.
## Not called by any menu draft.
func select(kind: String, id: int, opts: Dictionary = {}) -> void:
	## STUB(PC-1)
	push_error("DccApp.select(%s, %d, %s) is PC-1's and is not built" % [kind, id, str(opts)])

## PC-1. Edit ▸ Previous selection (Alt+←): makes the selection before this one current again, like a
## browser's Back, through `select(…, {"history": false})`. Returns false, and changes nothing, when
## there is no earlier selection; the menu then says so in the status line. Deselect is a step in the
## history ("Alt+← brings it back", §3.1), so back after Deselect restores what was cleared.
## Never reached from a key typed into a text field: Ruling DX, PQ12, so the menu's handler drops that
## press (`menus.gd::_on_edit_selection()`, `_typed_into_field()`); a click on the row still calls it.
func select_back() -> bool:
	## STUB(PC-1)
	return false

## PC-1. Edit ▸ Next selection (Alt+→): the reverse of `select_back()`. Returns false, and changes
## nothing, when there is nothing to go forward to.
func select_forward() -> bool:
	## STUB(PC-1)
	return false

## PC-1 (with PC-2's one year). The faction Records ▸ Factions… and Story ▸ Faction history… open on
## (finding W6): the selected faction; for a selected settlement, the faction that holds it in the year
## the timeline shows; -1 when nothing is selected or the selection has no faction. -1 is
## `open_faction_roster()`'s own documented "none" (it then opens on the last faction viewed), so it
## is the honest answer, never a guess. Must never return a faction from a stale copy (the right dock's
## own `_faction_id`, CIVIL's `_selected_index`): §2 forbids them.
func current_faction() -> int:
	## STUB(PC-1)
	return -1

## PC-4, PC-5, PC-6. The Context panel's tabs that exist in this build, in order: "inspect" from
## PC-4, "note" from PC-5, "ask" from PC-6. `menus.gd::_context_tab_ready()` asks it once per row at
## build time. On the phone, which has no panel (§3.11: "the model, not the panel"), it lists the tabs
## whose phone routes exist (`open_context_tab()` below). Since Ruling DX PQ13 the phone builds Window ▸
## Context panel and its submenu as the desktop does, so this list switches those rows on there too, as
## it does Story's. An empty list means "no panel": every panel row stays a `_todo`.
func context_tabs() -> PackedStringArray:
	## STUB(PC-4)
	return PackedStringArray()

## PC-4 (the tab, and the panel), PC-5 ("note"), PC-6 ("ask"). Shows the Context panel on `tab` and
## returns true; returns false and changes nothing when `tab` is not in `context_tabs()`. `""` keeps
## the tab the panel already shows (Window ▸ Open windows' panel row). `section` scrolls Inspect to a
## part of its page ("history": a place's holders year by year, a faction's wars, a conflict's years).
## "ask" also puts the cursor in the input (§3.5: "Ctrl+J opens Ask from anywhere and focuses the
## input"). Shows the panel region first if it is hidden.
##
## **Ruling DY (tabs you arrange):** Ctrl+1, Ctrl+2 and Ctrl+3 reach Inspect, Note and Ask by name, wherever
## the user has dragged them. If the user removed that tab from the panel, this puts it back and shows it,
## saving the arrangement (`context_panel_modules()`); without that, a removed Note could come back only
## through Reset panel. A labelled judgement of the drafts session (2026-11-01), not the owner's words.
##
## **On the phone** (Ruling DX PQ13, decided under the owner's delegation; §3.11) the rows open what the
## phone has, never a panel: "inspect" and `""` show the selection sheet (the right dock as a sheet,
## `DccShell.phone_show_right_sheet()`), "note" opens the vault at the selection's note, "ask" opens the
## assistant. Window ▸ Context panel's check is `toggle_region(ID_WIN_RIGHT)`, which on the phone already
## shows or hides that same sheet (`app.gd::toggle_region()`).
func open_context_tab(tab: String, section: String = "") -> bool:
	## STUB(PC-4)
	push_error("DccApp.open_context_tab(%s, %s) is PC-4's and is not built" % [tab, section])
	return false

## PC-4. Window ▸ Context panel ▸ Pin to the current subject: the panel keeps what it shows while the
## selection moves, and shows the banner "selection is now X" with PIN TO IT and FOLLOW (§3.3). Returns
## false, and changes nothing, when the panel shows nothing; the menu says so. On the phone the same pin
## holds the selection sheet, which is the right dock presented as a sheet, so the one state serves both.
func pin_context_panel() -> bool:
	## STUB(PC-4)
	return false

## PC-4. Window ▸ Context panel ▸ Follow the selection: releases a pin, so the panel shows the
## selection and follows each new one. A no-op when the panel is already following. On the phone it
## releases the selection sheet the same way.
func follow_selection() -> void:
	## STUB(PC-4)
	pass

## ── Ruling DY (owner, 2026-11-01): the Context panel is modular, "tabs you arrange" ─────────────────────
##
## The owner's PQ1 answer was "1, but it should be modular. Make suggestions for modules."; of the suggested
## shapes the owner took "Tabs you arrange" (each module a tab; Inspect · Note · Ask by default; + adds one;
## dragging reorders; any tab can open in its own window; a module that does not fit the selected kind is
## hidden; the arrangement kept with the user's settings, not in the world file), and these ten modules.
## The four methods below land together, in the milestone that builds the panel (PC-4), so that a module's
## menu row, which is live as soon as `context_module_registry()` lists it, always finds the other three.
##
## **One builder, two hosts.** Every module's content already ships inside a window. The module is that
## builder lifted out so the window and the panel call the same function; never a second implementation
## (`MISTAKES.md`, "Re-frame or redesign a window": start from its own builder). Read at the symbols on
## 2026-11-01 at `3099309`; re-open each before extracting it:
##   key        kinds (selection)                    shown with no selection   builds from today
##   history    settlement, faction, landmark        no    `place_editor_window.gd::_build_timeline` (Timeline
##                                                         tab) and `::_build_political` (Political history);
##                                                         `faction_roster_window.gd::_build_tab_history`; the
##                                                         note's chronos rows (`engine_bridge.gd::
##                                                         vault_entity_chronos`)
##   economy    settlement, faction                  no    `place_editor_window.gd::_build_trade` (the
##                                                         "economy" tab, Economy & notables) and
##                                                         `::_build_economy` (Overview); `faction_roster_window
##                                                         .gd::_build_economy_readout`, `::_build_currency`,
##                                                         `::_build_tariffs`
##   military   faction, settlement                  no    `faction_roster_window.gd::_build_military_block`
##                                                         (figures, garrison, conflicts); a place's garrison is
##                                                         `right_dock.gd::_build_settlement`'s Garrison row
##                                                         (`CivilizationWorkspace.garrison_row`), its fortified
##                                                         periods `place_editor_window.gd::_build_political`
##   faith      faction, culture                     no    `faction_roster_window.gd::_build_tab_relations` (with
##                                                         `::_build_faith_group`) and `::_build_culture_card`
##                                                         (the Identity tab's)
##   territory  faction                              no    `faction_roster_window.gd::_build_tab_territory`
##   city       settlement                           no    `place_editor_window.gd::_build_layout` (Layout tab);
##                                                         `city_viewer_window.gd::open`
##   here       settlement, landmark                 yes   `right_dock.gd::_build_sample` (`CTX_SAMPLE`,
##                                                         `SAMPLE_FIELDS`)
##   routes     settlement, landmark                 no    `right_dock.gd::_build_route` (`CTX_ROUTE`);
##                                                         `place_editor_window.gd::_build_journey_passes`;
##                                                         `saved_journeys_window.gd`
##   links      every kind                           no    `place_editor_window.gd::_build_backlinks`;
##                                                         `vault_window.gd::_build_browse_backlinks` (the vault
##                                                         crate's index: `engine_bridge.gd::vault_links_for`,
##                                                         `vault_refresh_backlinks`)
##   recent     every kind                           yes   nothing today: the history is PC-1's
##                                                         (`select_back()`, `select_forward()`); bookmarks are new,
##                                                         saved inside the world file (Ruling DY: "In the world
##                                                         file"), a new optional slot; the history is session-only
## The keys and titles are `menus.gd::CTX_MODULES`' (`drafts/pc-shell/05`), never a second spelling.

## PC-4 (Ruling DY). The modules this build can show, one Dictionary each: "key" (a `menus.gd::CTX_MODULES`
## key), "title" (its title there), "kinds" (a `PackedStringArray` of the selection kinds it shows for, from
## `selection`'s vocabulary), "without_selection" (a bool: whether it also shows when nothing is selected,
## true only for Here and Recent & bookmarks), "builder" (a `Callable(parent: Control, subject: Dictionary,
## year: int)`, the same function its window calls) and "empty" (the sentence the tab shows when the subject
## has nothing for it, never a blank tab). Lists a module only once its builder is extracted and both hosts
## call it, so `menus.gd::_context_module_ready()` lights exactly the rows that work. A module that does not
## fit the subject's kind stays in the arrangement and is hidden from the strip; it is never removed for that.
## Empty while there is no panel.
func context_module_registry() -> Array:
	## STUB(PC-4)
	return []

## PC-4 (Ruling DY). The user's arrangement: the keys of the panel's tabs in strip order, the core tabs
## ("inspect", "note", "ask") and modules alike; `["inspect", "note", "ask"]` until the user changes it.
## Stored per user with the user's settings, in a `[context_panel]` section of `DccSettings` written through
## an allow-listed setter (the `[quick_bar]` pattern: `DccSettings._SEC_QUICK_BAR`, `quick_bar_value()`,
## `set_quick_bar_value()`). **Never in a world file or a project save**: it is how this user likes the panel,
## not a fact about the world (the owner's option text: "saved with your settings, not in the world file").
## Lists the arrangement, not what is drawn: a module whose kinds exclude the subject is listed and hidden.
## Empty when there is no panel, which `menus.gd::_refresh_context_modules()` reads as "nothing ticked".
func context_panel_modules() -> PackedStringArray:
	## STUB(PC-4)
	return PackedStringArray()

## PC-4 (Ruling DY). Shows (appended at the end of the strip) or removes tab `key`, saves the arrangement and
## returns true. Called by Window ▸ Context panel ▸ MODULES' rows, the strip's + menu and a tab's Remove. Takes
## core tabs and modules alike. Returns false and changes nothing for a key that is neither a core tab nor in
## `context_module_registry()`, and when removing it would leave the panel with no tab (the menu then says so,
## `menus.gd::CTX_MODULE_REFUSED`). Dragging a tab to reorder is the strip's own gesture and saves through the
## same store; it is not this method.
func set_context_module_shown(key: String, shown: bool) -> bool:
	## STUB(PC-4)
	push_error("DccApp.set_context_module_shown(%s, %s) is PC-4's and is not built" % [key, str(shown)])
	return false

## PC-4 (Ruling DY). Window ▸ Context panel ▸ Reset panel: the arrangement back to Inspect · Note · Ask, saved.
## Removes the added modules from the strip; touches nothing they showed (a module is a view). Desktop and
## tablet only, as the row is.
func reset_context_panel() -> void:
	## STUB(PC-4)
	push_error("DccApp.reset_context_panel() is PC-4's and is not built")

## ── Ruling DY (owner, 2026-11-01), PQ4: the planner in its own window ────────────────────────────────────
##
## The owner, verbatim: "Planner should open in a separate window that doesn't block the map (that way a user
## could put it on a second screen for example)". It replaces both today's form (`journey_planner_view.gd::
## _show()` hides the CIVIL panel, and on the desktop the live map with it) and the proposal's left-dock
## workspace. Desktop only; the tablet and the phone keep the shipped swap.
##
## What PC-8 builds, so that `planner_in_window()` can answer true:
## - An `AcceptDialog` (`exclusive = false`) under `DccApp` that hosts `JourneyPlannerView`'s content, like
##   PQ3's five context windows. It must be an `AcceptDialog`: `menus.gd::_visible_windows()` walks
##   `AcceptDialog`s for Window ▸ Open windows, and `PopoutHost.add_toggle_button()` takes one.
## - Its script's basename joins `PopoutHost.LIVE_KEYS`, with `PopoutHost.add_toggle_button(self)` in its
##   `setup()` and `PopoutHost.prepare(self)` before each `popup_centered()` (the two lines `popout_host.gd`'s
##   header asks of every key). That is Ruling DK Q2's per-window Pop out button, shown even with the global
##   setting off; popped out, the window is native and can go on a second screen, and DK Q5 keeps the map live.
## - `open_journey_planner()` and `open_journey_planner_with_route()` open that window instead of the swap.
## - The selection still feeds Start here, Add as stop and End here, one undo step each (`PC_SHELL_PROPOSAL.md`
##   §3.7, PC-8's bar), and the busy overlay mutes the window during a generate as it mutes the others.

## PC-8 (Ruling DY, PQ4). True when `open_journey_planner()` opens the planner in its own window: the desktop,
## from PC-8. False on the tablet and the phone, which keep the shipped swap, and everywhere before PC-8.
## `menus.gd::_story()` reads it for Story ▸ Journey planner's tooltip, so the row says "its own window" only
## where that is true. Must never answer true where the window does not exist: the row would promise a window
## while the swap hides the map.
func planner_in_window() -> bool:
	## STUB(PC-8)
	return false
