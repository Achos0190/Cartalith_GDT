## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, the World rows (milestone PC-9). The menu keeps its name, "World"
## (Ruling CK; renaming it is PQ6, asked by name, and nothing here pre-empts the answer).
##   - Generate world, Ctrl+R, first on the PC: the tablet's Run pipeline command, same id (M1).
##   - Factions… gains Ctrl+Shift+F and opens on the current faction once PC-1 lands (W6, M8).
##   - Markdown vault… gains Ctrl+Shift+V (M8).
##   - The second Timeline row becomes the signpost "Timeline is in Story ▸ Timeline" (M3).
##
## Target: `shell/menus.gd`. Depends on `01_menus_ids_helpers.gd` (`_host_has()`).
## Applies to: the PC and the phone (whose MENU ▸ World re-presents this popup through
## `phone_menu.gd::_fill_data`). The tablet's World menu is `_tablet_world()`, which builds
## `_data()` below its own Run pipeline, so it gets every change here except the new first row.
##
## Forms: every row here is live from the moment it is pasted, except where Factions… opens. Before
## PC-1 it opens on the last faction viewed (shipped behaviour, `FACTIONS_TIP`). Once the host has
## `current_faction()` it opens on the current faction (`FACTIONS_CURRENT_TIP`).
##
## What to do, in order. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT W1  `build()`: in the non-tablet branch, replace the line
##                shell.add_menu("World", _data)
##            with
##                shell.add_menu("World", _pc_world)
##            (same two tabs of indentation). **If `drafts/mobile-shell/08_menus_tablet_bar.gd` is applied, skip
##            this**: 08 replaces `build()` whole and its whole bar already uses `_pc_world` (corrected
##            2026-11-01, when the menu parity check found 08's `build()` brought back `_data` and dropped
##            Generate world from the desktop).
##   ADD      `_pc_world()`, `RUN_PIPELINE_TIP`, `FACTIONS_TIP`, `FACTIONS_CURRENT_TIP`,
##            `TIMELINE_MOVED_SIGNPOST` and `_open_factions()` (below), beside `_data()`.
##   REPLACE  `_data()`'s FUNCTION (from `func _data(p: PopupMenu) -> void:` down to and including
##            its last line `p.id_pressed.connect(_on_data)`; 44 lines at a38e0ac) with the one below.
##            KEEP the long doc block above it, and append the `## **PC-9 …` paragraph marked below to
##            the end of that block, directly above the `func` line.
##   REPLACE  `_on_data()` (its whole body) with the one below.
##   EDIT W2  `_build_vault_rows()`: replace
##                _live(p, "Markdown vault…", ID_VAULT)
##            with the two lines
##                ## PC-9 (§3.8/§3.9, M8): Ctrl+Shift+V. Free: no row binds it; Paste is Ctrl+V.
##                _live(p, "Markdown vault…", ID_VAULT, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_V)
##   EDIT W3  `_world()` (tablet): the Run pipeline row's tooltip call is six lines: the line
##                p.set_item_tooltip(p.item_count - 1,
##            then four `##` comment lines, then the string that starts "Runs the whole generation
##            pipeline" and ends `from the start.")`. Replace all six with the one line
##                p.set_item_tooltip(p.item_count - 1, RUN_PIPELINE_TIP)
##            The four comment lines move to `RUN_PIPELINE_TIP`'s doc (done below), so one string
##            serves both rows and cannot drift.
##   EDIT W4  `SESSION_MODE_TIP`: replace the substring
##                Window ▸ Workspace ▸ World, and on a tablet World ▸ Run pipeline,
##            with
##                Window ▸ Workspace ▸ World, World ▸ Generate world, and on a tablet World ▸ Run pipeline,
##            (`_pc_world()` gates its row; `MISTAKES.md` "Change behaviour": the tooltip lists them).
##   PROSE    Comments the change makes false (`MISTAKES.md`, "Change behaviour"):
##            - `ID_WORLD_TIMELINE`'s comment ("SM-1: World ▸ RECORDS ▸ Timeline …"): add
##              "Retired by PC-9: the row is the signpost `TIMELINE_MOVED_SIGNPOST` now. Kept defined, never
##              reissued (`_storymenus_probe.gd` names it; a pin `menu:722` reads 'not in this build')."
##            - `TIMELINE_TIP`'s doc ("Tooltip of both Timeline rows (World ▸ RECORDS and, from SM-2,
##              Story ▸ TIME)"): it is Story ▸ TIME ▸ Timeline's tooltip only.
##            - `_open_timeline()`'s doc ("Both Timeline rows' action"): Story ▸ TIME ▸ Timeline's action.
##            - `_tablet_world()`'s doc ("40-48, 400+, 716, 719-722"): 719-721; 722 is retired.
##
## Verify: the parse check on `res://shell/menus.gd`; `check_symbols.sh`; then the probes the README
## lists for this draft (`_storymenus_probe.gd`'s WORLD_ROWS and its ID_WORLD_TIMELINE press,
## `_sessionmode_probe.gd`'s GATED_DESKTOP, `_tabletparity_probe.gd`). By hand, on the PC: Ctrl+R with
## an empty map generates; with Session mode on, World ▸ Generate world is dark and Ctrl+R does
## nothing; Ctrl+Shift+F opens Factions; Ctrl+Shift+V opens the vault.

## The tooltip of the one "regenerate the world" command, on both of its rows: the PC's World ▸
## Generate world (`_pc_world()`) and the tablet's World ▸ Run pipeline (`_world()`). One string, so
## the two rows cannot come to describe the same command differently.
## Same guarded path as `world_workspace.gd::_regenerate_live()`.
## "The engine has no partial recompute" removed 2026-09-24 (ALIGNMENT_
## AUDIT Part 2 B6): `recompute_stale_stages`/`recompute_civilisation`
## exist; Generate itself always runs every stage.
const RUN_PIPELINE_TIP := "Runs the whole generation pipeline -- the same path the WORLD dock's own Generate button uses: it asks first if the world holds hand-authored work (sculpt stamps, icons, labels, paint, routes, hand-drawn ways) a regenerate would discard. Every stage runs, from the start."

## World ▸ Factions…'s tooltip while the window opens on the last faction viewed (shipped
## behaviour, until the host has `current_faction()`). The shipped string, moved here unchanged.
const FACTIONS_TIP := "Open the Factions window: each faction's identity, territory, settlements, economy and military in one place."

## World ▸ Factions…'s tooltip once the host has `current_faction()` (PC-1, finding W6). Says what
## `90_pc_milestone_stubs.gd::current_faction()`'s contract says, so the claim and the call agree.
const FACTIONS_CURRENT_TIP := "Open the Factions window on the current faction: the selected faction, or the one that holds the selected place in the year the timeline shows; the last faction you viewed when nothing is selected. Each faction's identity, territory, settlements, economy and military in one place."

## The signpost that replaces World ▸ RECORDS ▸ Timeline (PC-9, finding M3), in the mockup's own
## words (`design/mobile-shell-2026-10-31/Shell.dc.html`, the desktop World menu). Prose, not a
## command: `_signpost()` marks it, so the command index skips it and the probe that wants every
## disabled row to state a reason does not read it as an undisclosed gap.
const TIMELINE_MOVED_SIGNPOST := "Timeline is in Story ▸ Timeline"

## The World menu of the PC and of the phone (whose MENU ▸ World re-presents this popup through
## `phone_menu.gd::_fill_data`): **Generate world, Ctrl+R, first** (`PC_SHELL_PROPOSAL.md` §3.8,
## finding M1: the PC had no row and no key that starts a generation), then `_data()`'s rows.
##
## The SAME command as the tablet's World ▸ Run pipeline: one id (`ID_WORLD_RUN_PIPELINE`), one
## dispatcher arm (`_on_world()`, connected here), one guarded path (`world_workspace.gd::
## _regenerate_live()`, which asks before discarding hand-authored work), one tooltip. One id is what
## keeps a rebind and a quick-bar pin (`menu:710`) the same command on every form factor. Ctrl+R is
## the tablet's key already; on the PC nothing else binds it (`check_symbols.sh`, keys section).
##
## Why not a row inside `_data()`: the tablet builds `_data()` too, under its own Run pipeline, and
## one id under two titles in one popup would make `command_index.gd::_disambiguate_pin_keys()`
## rename the tablet's pin key and orphan its pins. Session mode disables this row
## (`_session_gate_row`), as it does the tablet's. Must never be added to `_tablet_world()`.
func _pc_world(p: PopupMenu) -> void:
	_live(p, "Generate world", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)
	_session_gate_row(p, p.item_count - 1)
	p.set_item_tooltip(p.item_count - 1, RUN_PIPELINE_TIP)
	_data(p)
	## `_data()` connects `_on_data`; this row is `_on_world`'s. Each ignores the other's ids
	## (`_on_world` owns 710-713, of which only 710 is on this popup).
	p.id_pressed.connect(_on_world)

## World ▸ Factions… and Story ▸ Faction history… (`tab = "history"`): opens the Factions window on
## the current faction (finding W6) once the host can name one (`current_faction()`, PC-1), and on
## the last faction viewed before that, which is what `open_faction_roster(-1, …)` has always done
## and what `current_faction()` answers too when nothing is selected. Never guesses a faction: -1
## is `open_faction_roster`'s own documented "none", not a plausible id.
func _open_factions(tab: String = "") -> void:
	var fid := -1
	if _host_has("current_faction"):
		fid = int(_host.current_faction())
	_host.open_faction_roster(fid, tab)

## ── APPEND to the end of `_data()`'s doc block, directly above `func _data(` ──────────────────────
##
## **PC-9, `PC_SHELL_PROPOSAL.md` §3.8.** Two rows changed and none left. Factions… gained
## Ctrl+Shift+F and opens on the current faction once the host can name one (`_open_factions()`,
## finding W6). Markdown vault… gained Ctrl+Shift+V (`_build_vault_rows()`, finding M8). The Timeline
## row is a signpost to Story ▸ Timeline (finding M3: two rows, one action), the mockup's wording.
## Generate world sits ABOVE this function's rows on the PC and the phone (`_pc_world()`), not inside
## it, because the tablet builds this function too, under its own Run pipeline.
## ── end of the appended paragraph ──────────────────────────────────────────────────────────────

func _data(p: PopupMenu) -> void:
	## The canvas's own head row: `⧉ DATA MANAGER`, the window this whole menu
	## is a shortcut into. It had no row at all here -- the four group rows were
	## the only way in.
	_live(p, "⧉ Data manager", ID_DATA_MANAGER)

	p.add_separator("RECORDS")
	## FH-1: the Factions hub is a window over world data (a faction's identity,
	## land, settlements, economy and military). The phone twin needs no row of
	## its own -- `phone_menu.gd::_fill_data` draws this popup through
	## `_rest_of`, so the row appears there by being here.
	##
	## **Ctrl+Shift+F since PC-9** (`PC_SHELL_PROPOSAL.md` §3.8 and §3.9, finding
	## M8; PQ8 asks the owner for the whole key set). This said "No accelerator: a
	## shortcut is a new binding the owner has not asked for"; the proposal asks
	## for it now. Free: no row in this file binds it, and nothing else in the
	## project matches a modified F (the Freehand tool's bare F is an exact-match
	## `Shortcut`; `check_symbols.sh`, keys section). Where the window opens is
	## `_open_factions()`'s decision, and the tooltip follows the same test.
	_live(p, "Factions…", ID_FACTIONS, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_F)
	p.set_item_tooltip(p.item_count - 1,
		FACTIONS_CURRENT_TIP if _host_has("current_faction") else FACTIONS_TIP)
	## SM-1: a new row over a built target -- the CIVIL dock's Settlements
	## category (`civilization_workspace.gd`, `DccWidgets.category(self,
	## "Settlements", ...)`), which `RAIL_NODES` lists under CIVIL's `owns`.
	_live(p, "Places…", ID_PLACES)
	p.set_item_tooltip(p.item_count - 1,
		"Opens the CIVIL dock's Settlements category: the place list, its "
		+ "filters and each place's editor.")
	_build_vault_rows(p)
	## SM-5: the vault panel's Places without notes section
	## (`vault_window.gd::open_places_without_notes`). Story ▸ NOTES carried the
	## same row until PC-9 made it a signpost to this one (finding M3), so this
	## is the row, and the id the command index and a pin resolve to.
	_live(p, "Places without notes…", ID_PLACES_WITHOUT_NOTES)
	p.set_item_tooltip(p.item_count - 1, PLACES_WITHOUT_NOTES_TIP)
	## PC-9 (finding M3): SM-1's Timeline row here ran the same `_open_timeline()`
	## as Story ▸ TIME ▸ Timeline, so it is a signpost now. Story's row keeps the
	## accelerator (Ctrl+Shift+T) and the id the assistant's window list names
	## (`llm_view.gd::WINDOWS`). `ID_WORLD_TIMELINE` is retired, never reissued.
	_signpost(p, TIMELINE_MOVED_SIGNPOST)
	## Travel library… (⇧L) closed this band until SM-2, which moved it to
	## Story ▸ JOURNEYS (`_story()`), the same id and accelerator.

	p.add_separator("TECHNICAL")
	_live(p, "World data tables…", ID_WORLD_DATA)
	_route_submenu(p, "Import", "WorldImport", ["Import"])
	_route_submenu(p, "Sources & validation", "WorldSources", ["Sources", "Validation"])
	## SM-3 (§3.1, Q4's default): the Export submenu moved to File ▸ Export
	## (`_build_file_export()`), with the same route ids; imports stay here.
	## The signpost is the proposal's own row, so a user who learned
	## World ▸ Export ▸ is told where it went.
	_signpost(p, "Export is in File ▸ Export")
	p.id_pressed.connect(_on_data)

## The World menu's dispatcher, shared by the popup and its three route
## submenus (and, on tablet, by the merged World popup beside `_on_world`).
## Ignores every id it does not own, so a fan-out with other dispatchers on one
## popup is safe.
func _on_data(id: int) -> void:
	## Bounded on BOTH ends. This used to be `if id >= ID_DATA_ROUTE_FIRST:
	## ... return`, which swallowed every id at or above 400 whether or not
	## it was a route -- and `ID_VAULT_BROWSE` (716) is one, so "Browse &
	## edit a note…" landed here as route 316 of 15, failed the size check,
	## and returned before the `match` below: the row did nothing, silently,
	## from the day it shipped (`_vaultclick_probe.gd`).
	var i := id - ID_DATA_ROUTE_FIRST
	if i >= 0 and i < _data_route_ids.size():
		_host.open_data_manager_route(_data_route_ids[i])
		return
	match id:
		ID_DATA_MANAGER: _host.open_data_manager()
		ID_WORLD_DATA: _host.open_world_data()
		## PC-9 (finding W6): through `_open_factions()`, which opens on the
		## current faction once the host can name one.
		ID_FACTIONS: _open_factions()
		ID_PLACES: _shell.select_domain_category("civilization", "Settlements")
		## `ID_WORLD_TIMELINE` has no arm since PC-9: no row carries it (the row is
		## a signpost), so an arm here could only ever be reached by a probe
		## emitting the id by hand, and would make that probe pass on dead code.
		ID_EXPORT_MAP_IMAGE: _host.open_export_dialog()
		ID_VAULT: _host.open_vault_overview()
		ID_VAULT_BROWSE: _host.open_vault_browse()
		ID_PLACES_WITHOUT_NOTES: _host.open_places_without_notes()
