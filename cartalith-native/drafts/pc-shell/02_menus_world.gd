## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, the World rows (milestone PC-9), and **Ruling DY** (owner, 2026-11-01: PQ6,
## "Rename to 'Records'", and its two follow-ups). The menu that held the records is renamed, and the regenerate
## row moves out of it:
##   - **Records** is the menu's title on the PC, the phone and the wide tablet. That reopens Ruling CK Q3, which
##     named the old Data menu "World"; the audit found the name collided with the WORLD domain, which means
##     terrain. Every row of `_data()` stays, in order.
##   - **Generate world (Ctrl+R) sits in File, directly after New world…** (the owner: "File, after New world…"),
##     on the PC, the phone and the wide tablet (`_build_generate_world_row()`, EDIT W6). It is the tablet's
##     regenerate command, same id (M1), titled "Generate world" on every form factor (Ruling DX PQ11, EDIT W5);
##     search keeps "Run pipeline" as an alias (`07_command_index.gd`).
##   - **The touch tablet's short bar is ☰ · File · World · Records · Story** (the owner chose that bar over
##     keeping three menus; it reopens Ruling CK Q2, which merged Data's rows into the tablet's World menu). That
##     bar is every tablet's until MS-7, and the one below 1 180 dp after it (`drafts/mobile-shell/08`). World is
##     `_world()` alone: the WORLD domain's own rows (Generate world, Reset generation parameters, Reset one stage,
##     Generation pipeline / Sculpt, Seed & size…). Records is `_data()`, the same popup as on every other bar. On
##     that bar File leaves Generate world out, because World holds it: one row per command per bar (a labelled
##     judgement, `_generate_in_world`). `_tablet_world()`, which merged the two, is retired (EDIT W1).
##   - Factions… gains Ctrl+Shift+F and opens on the current faction once PC-1 lands (W6, M8).
##   - Markdown vault… gains Ctrl+Shift+V (M8).
##   - The second Timeline row becomes the signpost "Timeline is in Story ▸ Timeline" (M3).
##   - Every user-visible string that names a path through the old World menu says Records (EDITs W7, W9-W13);
##     two that were already wrong are fixed in the same sweep (W14); and on the phone, until
##     `drafts/mobile-shell/05` retires them, the two lookups of the menu by its title follow it (W8).
##
## **Superseded before it was pasted (2026-11-01).** This draft's first form added `_pc_world()`: Generate world,
## then `_data()`, as the PC's World menu. The drafts are not applied, so nothing was pasted and nothing has to be
## undone; `_pc_world()` is gone, and `drafts/mobile-shell/08` no longer calls it.
##
## Target: `shell/menus.gd`; EDITs W8-W14 name their own files. Depends on `01_menus_ids_helpers.gd`
## (`_host_has()`).
## Applies to: every form factor. The PC and the phone build Records (the phone's MENU ▸ Records re-presents this
## popup row for row, `drafts/mobile-shell/05_phone_menu_root.gd`, Ruling DX Q13) and File with Generate world.
## The tablet builds the short bar above until MS-7; from MS-7 a tablet from 1 180 dp builds the PC's eight menus
## (`drafts/mobile-shell/08_menus_tablet_bar.gd`).
##
## Forms: every row here is live from the moment it is pasted, except where Factions… opens. Before PC-1 it opens
## on the last faction viewed (shipped behaviour, `FACTIONS_TIP`). Once the host has `current_faction()` it opens
## on the current faction (`FACTIONS_CURRENT_TIP`).
##
## What to do, in order. Each EDIT quotes its old text exactly as it stands at `a38e0ac` (W6-W14 at `3099309`).
##   EDIT W1  `build()`. **Skip all three parts if `drafts/mobile-shell/08_menus_tablet_bar.gd` is applied**: 08
##            replaces `build()` whole and builds the same bars (corrected 2026-11-01, when the menu parity check
##            found an older 08 `build()` brought back `_data` and dropped Generate world from the desktop).
##            W1a  directly after the line (tablet branch, two tabs)
##                     shell.add_menu("☰", _overflow)
##                 insert the two lines (two tabs)
##                     ## Ruling DY (PQ6): this bar's World menu holds Generate world, so File leaves its own out.
##                     _generate_in_world = true
##            W1b  replace the line
##                     shell.add_menu("World", _tablet_world)
##                 with the two lines (two tabs)
##                     shell.add_menu("World", _world)
##                     shell.add_menu("Records", _data)
##            W1c  replace the line (the `else` branch)
##                     shell.add_menu("World", _data)
##                 with
##                     shell.add_menu("Records", _data)
##   DELETE   `_tablet_world()` with its six-line doc: from the line `## The tablet's single World menu since SM-1
##            (Q2's default, Ruling CK): the` down to and including its last line `_data(p)` (10 lines at
##            `a38e0ac`; fingerprinted in `check_symbols.sh`, section 4). Inventory (`MISTAKES.md`, "Delete a
##            surface"): its only caller was `build()` (W1b); it built `_world()`'s rows and `_data()`'s on one
##            popup, and both are on the same bar after W1b, each on its own popup with its own dispatchers
##            (`_world()` connects `_on_edit` and `_on_world`, `_data()` connects `_on_data`). Nothing is dropped.
##   ADD      `_generate_in_world`, `_build_generate_world_row()`, `RUN_PIPELINE_TIP`, `FACTIONS_TIP`,
##            `FACTIONS_CURRENT_TIP`, `TIMELINE_MOVED_SIGNPOST` and `_open_factions()` (below), beside `_data()`.
##   REPLACE  `_data()`'s FUNCTION (from `func _data(p: PopupMenu) -> void:` down to and including
##            its last line `p.id_pressed.connect(_on_data)`; 44 lines at a38e0ac) with the one below.
##            KEEP the long doc block above it, and append the `## **PC-9 …` paragraph marked below to
##            the end of that block, directly above the `func` line.
##   REPLACE  `_on_data()` with its doc (from `## The World menu's dispatcher, shared by the popup and its three
##            route` down to its last `match` arm) with the one below.
##   EDIT W2  `_build_vault_rows()`: replace
##                _live(p, "Markdown vault…", ID_VAULT)
##            with the two lines
##                ## PC-9 (§3.8/§3.9, M8): Ctrl+Shift+V. Free: no row binds it; Paste is Ctrl+V.
##                _live(p, "Markdown vault…", ID_VAULT, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_V)
##   EDIT W3  `_world()` (tablet): the regenerate row's (`ID_WORLD_RUN_PIPELINE`) tooltip call is six lines: the line
##                p.set_item_tooltip(p.item_count - 1,
##            then four `##` comment lines, then the string that starts "Runs the whole generation
##            pipeline" and ends `from the start.")`. Replace all six with the one line
##                p.set_item_tooltip(p.item_count - 1, RUN_PIPELINE_TIP)
##            The four comment lines move to `RUN_PIPELINE_TIP`'s doc (done below), so one string
##            serves both rows and cannot drift.
##   EDIT W4  `SESSION_MODE_TIP`: replace the substring
##                Window ▸ Workspace ▸ World, and on a tablet World ▸ Run pipeline,
##            with
##                Window ▸ Workspace ▸ World, File ▸ Generate world, and on a tablet whose bar has a World menu World ▸ Generate world,
##            (the sentence then reads "…, File ▸ Generate world, and on a tablet whose bar has a World menu World
##            ▸ Generate world, Generation pipeline, Sculpt and Seed & size…;"). `_build_generate_world_row()`
##            gates File's row and W5 renames the tablet's; `MISTAKES.md` "Change behaviour": the tooltip lists
##            every gated row by the name and the menu it carries. "Whose bar has a World menu" is true before
##            MS-7 (every tablet) and after it (below 1 180 dp) alike.
##   EDIT W5  `_world()` (tablet), Ruling DX PQ11: replace the line
##                _live(p, "Run pipeline", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)
##            with
##                _live(p, "Generate world", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)
##            One command, one label: the owner's ruling departs from the tablet canvas's "Run pipeline"
##            (DS-03), and an owner decision is newer than a canvas. Same id, key, gate and tooltip, so a
##            rebind or a quick-bar pin (`menu:710`) is untouched. In the same change: `07_command_index.gd`'s
##            "Run pipeline" alias, and the two probes that name the row (README, "Probes to update").
##   EDIT W6  `_file()`, Ruling DY: directly after the line
##                _live(p, "New world…", ID_NEW_WORLD, KEY_MASK_CTRL | KEY_N)
##            and the `_session_gate_row(p, p.item_count - 1)` line that follows it, insert the two lines (one tab)
##                ## Ruling DY (PQ6): Generate world, directly after New world…, unless this bar's World menu holds it.
##                _build_generate_world_row(p)
##            Every row index `_file()` captures (`region_new_idx`, `save_idx`, …) is taken after its own row is
##            added, so the extra row moves none of them.
##   EDIT W7  `_file()`'s two-line signpost, Ruling DY: replace the two lines
##                ## "World", not "Data", since SM-1 renamed the menu (Ruling CK).
##                for line in ["imports live under World ▸ Import",
##            with
##                ## "Records" since Ruling DY renamed the menu (it was "Data" until SM-1, then "World", Ruling CK).
##                for line in ["imports live under Records ▸ Import",
##   EDIT W8  `shell/phone_menu.gd`, **only while `drafts/mobile-shell/05_phone_menu_root.gd` has not landed** (05
##            retires both functions with the screens they serve; skip W8 if it is applied). The shipped MORE root
##            looks the menu up by its title, so after W1 the `data` screen would find no "World" and the Records
##            popup would land in the "Not on the MORE list" band as a second route to it:
##            - `_fill_data()`: replace `var p := _menu_popup("World")` with `var p := _menu_popup("Records")`,
##              and the text `"The World menu is not on this build's menu bar."` with
##              `"The Records menu is not on this build's menu bar."`;
##            - `_named_in_root()`: replace the line
##                  if ["File", "World", "Story", "Preferences", "Assets", "Help"].has(menu_title):
##              with
##                  if ["File", "Records", "Story", "Preferences", "Assets", "Help"].has(menu_title):
##              and its comment's first line ("World" was "Data" until SM-1 renamed the menu (Ruling CK); the `data`)
##              with ("Records" since Ruling DY; it was "Data" until SM-1, then "World"; the `data`).
##   EDIT W9  `shell/data_manager_window.gd`, `VAULT_NOTE`: the substring `(World ▸ Markdown vault)` becomes
##            `(Records ▸ Markdown vault)`.
##   EDIT W10 `shell/data_manager_window.gd`: the substring `the panel that owns those links is World ▸ Markdown
##            vault.` becomes `the panel that owns those links is Records ▸ Markdown vault.`
##   EDIT W11 `shell/data_manager_window.gd`: the substring `"Opens World ▸ Markdown vault, which owns` becomes
##            `"Opens Records ▸ Markdown vault, which owns`.
##   EDIT W12 `shell/place_editor_window.gd`, `_build_backlinks()`: the substring `"World ▸ Markdown vault… ▸ Index
##            builds it.` becomes `"Records ▸ Markdown vault… ▸ Index builds it.`
##   EDIT W13 `shell/workspaces/world_workspace.gd`, the Load button's tooltip: the substring `Opens World ▸ Import,
##            whose Heightmaps route` becomes `Opens Records ▸ Import, whose Heightmaps route`.
##   EDIT W14 Two strings that were wrong before Ruling DY: Travel library left this menu for Story ▸ JOURNEYS with
##            SM-2 (Ruling CK, 2026-10-18), and after W1 no desktop menu is called World at all.
##            - `shell/journey_planner_view.gd`: `party set-ups live in World ▸ Travel library (⇧L)` becomes
##              `party set-ups live in Story ▸ Travel library… (⇧L)`.
##            - `shell/data_manager_window.gd`: `"resolution": "World ▸ Travel library ⇧L ▸ %s"` becomes
##              `"resolution": "Story ▸ Travel library… ⇧L ▸ %s"`.
##   PROSE    Comments the change makes false (`MISTAKES.md`, "Change behaviour"):
##            - `build()`'s doc ("tablet: ☰ / File / World, the old Data rows merged into World; PC and phone:
##              File, Edit, Assets, World, …") and its SM-1 paragraph ("so the tablet does NOT get a second one:
##              Q2's default merges Data's rows into it … ☰ · File · World"), when W1 is the path taken: Ruling DY
##              names the menu Records and splits the tablet's bar into ☰ · File · World · Records · Story.
##            - `ID_DATA_MANAGER` ("World ▸ Data manager (the menu was "Data" until SM-1)"), `ID_VAULT`,
##              `ID_WORLD_DATA`, the SM-1 block above `ID_PLACES` ("World ▸ RECORDS ▸ Places…"), the block
##              comment "on two menus (World ▸ RECORDS and Story ▸ NOTES)", `PLACES_WITHOUT_NOTES_TIP`'s doc,
##              Story's comment "The same row as World ▸ RECORDS", and `vault_window.gd`'s "The menu entry (World ▸
##              RECORDS and Story ▸ NOTES": Records.
##            - `ID_WORLD_TIMELINE`'s comment ("SM-1: World ▸ RECORDS ▸ Timeline …"): add
##              "Retired by PC-9: the row is the signpost `TIMELINE_MOVED_SIGNPOST` now. Kept defined, never
##              reissued (`_storymenus_probe.gd` names it; a pin `menu:722` reads 'not in this build')."
##            - `TIMELINE_TIP`'s doc ("Tooltip of both Timeline rows (World ▸ RECORDS and, from SM-2,
##              Story ▸ TIME)"): it is Story ▸ TIME ▸ Timeline's tooltip only.
##            - `_open_timeline()`'s doc ("Both Timeline rows' action"): Story ▸ TIME ▸ Timeline's action.
##            - `_world()`'s doc ("Builds the tablet's World menu from its canvas: Run pipeline (Ctrl+R), …")
##              and `_on_world()`'s ("Run pipeline calls the world workspace's guarded …"): Generate world
##              (W5). `ID_WORLD_RUN_PIPELINE` keeps its name: an identifier, not a label. `_world()`'s "PC and
##              phone have no such menu" stays true: only the tablet's short bar has World.
##            - `ID_NEW_WORLD_FROM_SELECTION`'s doc ("Appended after `Recent worlds`, not inserted under `New
##              world…`, so no row the canvas draws moves"; "File's own signpost ("imports live under Data ▸
##              Import")"): add that Ruling DY then inserted Generate world under New world…, which moves Open
##              project… and Recent worlds down one row (an owner decision is newer than a canvas), and that the
##              signpost reads Records ▸ Import (W7).
##
## Verify: the parse check on `res://shell/menus.gd` and on every file W8-W14 touches; `check_symbols.sh`; then the
## probes the README lists for this draft (`_storymenus_probe.gd`, `_sessionmode_probe.gd`,
## `_tabletparity_probe.gd`, `_factionhub_probe.gd`, `_v3menu_probe.gd`, `_vfyb13_probe.gd`,
## `_structmove_probe.gd`, `_vfy_batch0905_probe.gd`, `_adaudit_probe.gd`, `_adpress_probe.gd`,
## `_phonemore_reach_probe.gd`). By hand, on the PC: the bar reads File · Edit · Assets · Records · Story ·
## Preferences · Window · Help; File's second row is Generate world · Ctrl+R; Ctrl+R with an empty map generates;
## with Session mode on, File ▸ Generate world is dark and Ctrl+R does nothing; Ctrl+Shift+F opens Factions;
## Ctrl+Shift+V opens the vault. On a tablet (`--force-touch`): ☰ · File · World · Records · Story; World starts
## with Generate world · Ctrl+R and ends with Seed & size…; File has no Generate world; Records is the PC's.

## True while `build()` builds a bar whose World menu is `_world()` (the WORLD domain's own rows, Generate
## world first): the touch tablet's short bar, ☰ · File · World · Records · Story (Ruling DY). `_file()` then
## leaves its own Generate world row out (`_build_generate_world_row()`): one row per command per bar, so the
## command index never lists one command twice on one bar. Written only by `build()`, before it adds File,
## and in both of its branches where `build()` can run again on one instance (MS-1's mount path rebuilds the
## bar; `drafts/mobile-shell/08` sets it each time). Must never be read as "this is a tablet": a wide tablet
## from MS-7 builds the PC's bar and leaves it false.
var _generate_in_world := false

## File ▸ Generate world (Ctrl+R), directly after New world… (Ruling DY: the owner chose "File, after New
## world…"): the PC's, the phone's and the wide tablet's row for the one "regenerate the world" command.
##
## The SAME command as the short tablet bar's World ▸ Generate world (`_world()`, EDIT W5): one id
## (`ID_WORLD_RUN_PIPELINE`, 710), one dispatcher arm (`_on_world()`, connected here), one guarded path
## (`world_workspace.gd::_regenerate_live()`, which asks before discarding hand-authored work), one tooltip
## (`RUN_PIPELINE_TIP`). One id is what keeps a rebind and a quick-bar pin (`menu:710`) the same command on
## every form factor. Ctrl+R is the tablet's key already; on the PC nothing else binds it (`check_symbols.sh`,
## keys section). Session mode disables the row (`_session_gate_row`), as it does New world….
##
## Why File: the owner's answer to where it goes once World became Records. File already holds the commands
## that decide which world is open (New world…, Open project…, Recent worlds, New world from selection…).
## Builds nothing when `_generate_in_world` is set, because that bar's World menu carries the row. Must never
## be added to `_data()` (Records): the owner moved it out of that menu.
func _build_generate_world_row(p: PopupMenu) -> void:
	if _generate_in_world:
		return
	_live(p, "Generate world", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)
	_session_gate_row(p, p.item_count - 1)
	p.set_item_tooltip(p.item_count - 1, RUN_PIPELINE_TIP)
	## `_on_file()` has no arm for 710 and `_on_world()` matches only 710-713, none of which is a File id,
	## so the two dispatchers share File's popup the way `_overflow()`'s four do.
	p.id_pressed.connect(_on_world)

## The tooltip of the one "regenerate the world" command, on both of its rows: File ▸ Generate world
## (`_build_generate_world_row()`: the PC, the phone, the wide tablet) and World ▸ Generate world on the
## tablet's short bar (`_world()`; W5 gives it the same label, Ruling DX PQ11). One string, so the two rows
## cannot come to describe the same command differently.
## Same guarded path as `world_workspace.gd::_regenerate_live()`.
## "The engine has no partial recompute" removed 2026-09-24 (ALIGNMENT_
## AUDIT Part 2 B6): `recompute_stale_stages`/`recompute_civilisation`
## exist; Generate itself always runs every stage.
const RUN_PIPELINE_TIP := "Runs the whole generation pipeline -- the same path the WORLD dock's own Generate button uses: it asks first if the world holds hand-authored work (sculpt stamps, icons, labels, paint, routes, hand-drawn ways) a regenerate would discard. Every stage runs, from the start."

## Records ▸ Factions…'s tooltip while the window opens on the last faction viewed (shipped
## behaviour, until the host has `current_faction()`). The shipped string, moved here unchanged.
const FACTIONS_TIP := "Open the Factions window: each faction's identity, territory, settlements, economy and military in one place."

## Records ▸ Factions…'s tooltip once the host has `current_faction()` (PC-1, finding W6). Says what
## `90_pc_milestone_stubs.gd::current_faction()`'s contract says, so the claim and the call agree.
const FACTIONS_CURRENT_TIP := "Open the Factions window on the current faction: the selected faction, or the one that holds the selected place in the year the timeline shows; the last faction you viewed when nothing is selected. Each faction's identity, territory, settlements, economy and military in one place."

## The signpost that replaces Records ▸ RECORDS ▸ Timeline (PC-9, finding M3), in the mockup's own
## words (`design/mobile-shell-2026-10-31/Shell.dc.html`, the desktop menu). Prose, not a
## command: `_signpost()` marks it, so the command index skips it and the probe that wants every
## disabled row to state a reason does not read it as an undisclosed gap.
const TIMELINE_MOVED_SIGNPOST := "Timeline is in Story ▸ Timeline"

## Records ▸ Factions… and Story ▸ Faction history… (`tab = "history"`): opens the Factions window on
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
##
## **Ruling DY (owner, 2026-11-01): this menu is titled `Records` on every bar** (`build()`), which
## reopens Ruling CK Q3 ("World"): the audit behind `PC_SHELL_PROPOSAL.md` PQ6 found that name collided
## with the WORLD domain, which means terrain. Its rows did not change. Generate world is not one of them:
## it is File ▸ Generate world (`_build_generate_world_row()`) on the PC, the phone and the wide tablet, and
## World ▸ Generate world on the tablet's short bar, whose World menu is `_world()` alone. The short bar
## no longer merges this popup into World (`_tablet_world()`, Ruling CK Q2, retired by the same ruling).
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

## The Records menu's dispatcher (the World menu until Ruling DY, the Data menu
## until SM-1), shared by the popup and its three route submenus. Ignores every id
## it does not own, so a fan-out with other dispatchers on one popup is safe.
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
