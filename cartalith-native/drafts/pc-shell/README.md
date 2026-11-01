# PC shell: GDScript drafts of the menus and keys

**What this is.** GDScript drafts of `PC_SHELL_PROPOSAL.md` §3.8 (menus) and §3.9 (keys), milestone PC-9, written
2026-11-01 against commit `a38e0ac`, and brought to **Ruling DY** the same day against `3099309` (the World menu
renamed Records, Generate world in File, the tablet's short bar, the Context panel's modules, the planner in its own
window; section "Ruling DY" below). The owner asked: *"Also make the pc menus ready in the godot format for easy
implementation."* Each draft's header names its target file, what it replaces or adds (by symbol, with the old text
quoted exactly), what it depends on, which form of each row applies when, and how to verify it.

**What it is not.** It is not a build. This folder is outside `godot-project/`, so Godot never loads it. How far
PC-9 or any other milestone has got is `cartalith-native/docs/STATUS.md`'s answer, never this folder's. The design is
`PC_SHELL_PROPOSAL.md`; the desktop rows are drawn in `design/mobile-shell-2026-10-31/Shell.dc.html` (the
`Desktop` frame), which these drafts follow where the proposal's prose is less exact.

**Not parse-checked.** The cloud session that wrote these had no Godot binary. What was checked instead:
`check_symbols.sh` (below: 217 shipped symbols and files the drafts call or name, 129 new names, 43 edit anchors,
9 replaced or deleted blocks; all pass at `3099309`. Before Ruling DY it was 161, 94, 27 and 8, passing at `a38e0ac`
and `7d2acd1`), a lint of the drafts (tab indentation only, balanced brackets per file: 0 problems),
and mutation runs of `check_symbols.sh` itself on a scratch copy of the tree (a renamed helper, a taken name, a
changed anchor, a one-character edit inside a replaced block, a pasted stub, a key bound to another row: each made
it fail; a CRLF copy passed; and for Ruling DY's checks, a renamed module builder, a taken `CTX_MODULES`, a moved
`_tablet_world()`, an applied EDIT W13 and a taken id 925: each made it fail, and the restored copy passed). None of that is a parse. The first thing to run on the PC, from the repository root,
after **every** edit (`MISTAKES.md`, "Re-run the parse check after EVERY edit"; "Run `--check-only`": from the
project root, with `--script`):

```
godot --headless --path cartalith-native/godot-project --check-only --script res://shell/<file>.gd
```

Then the same for `res://shell/app.gd`, whichever file was touched. A failed script load makes `--headless` hang
rather than exit.

## Order of work

Drafts 01-09 are one change (one commit), applied in this order with the parse check after each. The pairs marked
in the last column are mandatory: `MISTAKES.md` ("Move a command off the menu bar") wants the `EXTRAS` and `UNLISTED`
rows in the same change as the move, and Q breaks if 03 and 09 are split.

| # | Draft | Target | Replaces / adds (by symbol) | Lands with |
|---|---|---|---|---|
| 0 | `check_symbols.sh` | none | re-checks every draft against the current tree; run it first | |
| 1 | `01_menus_ids_helpers.gd` | `shell/menus.gd` | adds `ID_EDIT_SELECT_BACK` … `ID_WIN_CTX_FOLLOW` (910-918), the module rows' `ID_WIN_CTX_MOD_*` and `ID_WIN_CTX_RESET` (920-930, Ruling DY), `_host_has()`, `_context_tab_ready()`, `_typed_into_field()`, `cartwheel_keycode()` | everything |
| 2 | `02_menus_world.gd` | `shell/menus.gd`; W8-W14 name their own files | `build()`'s three bar lines (W1: World becomes Records, the tablet's short bar becomes ☰ · File · World · Records · Story; not needed when `drafts/mobile-shell/08` replaces `build()`); deletes `_tablet_world()`; replaces `_data()` and `_on_data()`; adds `_generate_in_world`, `_build_generate_world_row()` (File ▸ Generate world, W6), `_open_factions()`, `RUN_PIPELINE_TIP`; edits `_build_vault_rows()`, `_world()`'s tooltip and its row's label (W5, Ruling DX PQ11), `SESSION_MODE_TIP`, File's signpost (W7); W8 for `phone_menu.gd` until mobile draft 05 lands; W9-W14 rename "World ▸" in five user-visible strings and fix two already-stale ones | **07** |
| 3 | `03_menus_edit.gd` | `shell/menus.gd` | edits `_build_edit_actions()` (Deselect's tooltip; two insertions) and `_edit()`; adds `_build_selection_rows()`, `_build_map_surface_rows()`, `_on_edit_selection()`, `_deselect_tip()` | **08, 09** |
| 4 | `04_menus_story.gd` | `shell/menus.gd` | replaces `_story()`, `_refresh_story_menu()`, `_on_story()` (each with its doc); adds `_story_forms`, `_story_selection()` and their constants | |
| 5 | `05_menus_window.gd` | `shell/menus.gd` | edits `_window()`'s region loop, `_layout_readout()`, `_open_windows`' comment; replaces `_refresh_open_windows()`, `_on_open_window()`; adds `_region_label()`, `_build_context_panel_menu()` (with Ruling DY's MODULES band and Reset panel, not on the phone), `_on_context_panel()`, `_visible_windows()`, `CTX_MODULES` and the four module helpers | |
| 6 | `06_menus_help_assets.gd` | `shell/menus.gd` | replaces `_build_help_shortcuts_row()`; removes `_assets()`'s Import image row and `_on_assets()`'s arm; edits two `ID_*` comments | **07** |
| 7 | `07_command_index.gd` | `shell/command_index.gd` | five `EXTRAS` rows (four aliases and Ruling DY's "Records (was World)" pointer), the `alias` skip in `build()`, `_has_menu_title()` | **06, 02** |
| 8 | `08_shortcuts_dialog.gd` | `shell/shortcuts_dialog.gd` | one `UNLISTED` row (the Menu key); window title per mode (`_set_mode_title()`, `_phone_title`) | **03, 09** |
| 9 | `09_app_keys.gd` | `shell/app.gd` | replaces `_unhandled_key_input()` with its doc (four exact replacements of the shipped text); adds `open_context_card()`, `open_cartwheel_at_pointer()` | **03, 08** |
| 10 | `10_dcc_shell_menu_hover.gd` | `shell/dcc_shell.gd` | one line in `add_menu()`: `switch_on_hover` on the desktop | independent |
| 11 | `11_tool_options_bar.gd` | `shell/app.gd`, `shell/workspaces/world_workspace.gd` | **Ruling DZ.** Replaces `_tool_options_generate()` (WORLD's idle row becomes the context name and one hint, through `_tool_options_simple()`; two new constants `WORLD_IDLE_HINT_POINTER` and `WORLD_IDLE_HINT_TOUCH`); deletes `_tool_options_stale`, `_tool_options_bake`, `set_bake_shortcut()`, `bake_and_finalize()` and `_refresh_finalize()`'s push to the bar (EDITs T1-T6) | none; lands on its own |
| 90 | `90_pc_milestone_stubs.gd` | `shell/app.gd` | **not applied with the others.** The contract PC-1, PC-4 (with Ruling DY's module registry and arrangement), PC-5, PC-6 and PC-8 (the planner window, Ruling DY) implement; each milestone writes its methods' real bodies | its milestone |

Every row id the proposal renames is kept: a rebind (`DccSettings`, keyed by id), a quick-bar pin (`menu:<id>`) and
the assistant's window list (`llm_view.gd::WINDOWS`, by id) all follow a row from its old name to its new one.

## Forms: which row is built when

A row gets one of three treatments, decided once at build time (menus are built once per boot, so a milestone that
lands shows its rows from the next start):

1. **No dependency: live from the moment it is pasted.** The Records title and the tablet's short bar (Ruling DY);
   File ▸ Generate world (Ctrl+R) on the PC, the phone and the wide tablet; Ctrl+Shift+F on Factions…; Ctrl+Shift+V
   on Markdown vault…; the two signposts; Keyboard reference…; Import image removed; Open windows walking every
   window; Context card (Shift+F10) and Cartwheel at the pointer (Q), not on the phone; the tablet's World row
   titled Generate world (Ruling DX PQ11).
2. **A shipped row whose new form needs a milestone: the shipped form until then.** It works today, and a `_todo` in
   its place would take away a working command and its key (`MISTAKES.md`, "Delete a surface").
3. **A new row with no behaviour today: `_todo(p, label, reason)` until its milestone**, the shipped convention for a
   command the port cannot honour yet. The menu never promises what does not exist.

The test is `DccMenus._host_has(method)` for PC-1's methods, `DccMenus._context_tab_ready(tab)` for the Context
panel's tabs (`DccApp.context_tabs()`) and, since Ruling DY, `DccMenus._context_module_ready(key)` for its modules
(`DccApp.context_module_registry()`). `check_symbols.sh` section 6a reports which of them exist today.

| Row | Before its milestone | From its milestone | Milestone (gate) |
|---|---|---|---|
| Edit ▸ Previous / Next selection | `_todo`, `SELECTION_HISTORY_TODO` | live, Alt+← / Alt+→ (ignored while typing in a field, Ruling DX PQ12) | PC-1 (`select_back`, `select_forward`) |
| Edit ▸ Deselect | shipped tooltip | `DESELECT_ONE_TIP` (behaviour is `clear_selection()`'s) | PC-1 (`select_back`) |
| Records ▸ Factions… | last faction viewed (shipped) | current faction | PC-1 (`current_faction`) |
| Story ▸ Faction history… | last faction viewed (shipped) | current faction | PC-1 (`current_faction`) |
| Story ▸ Political history | "…of this place…", Place editor, settlement only | "…of the selection", Inspect | PC-4 (`context_tabs` has inspect) |
| Story ▸ note row | "Notes for this place", vault window | "Note for the selection", Note tab | PC-5 (note) |
| Story ▸ assistant row | "Assistant panel…", no key | "Ask the assistant about the selection", Ctrl+J | PC-6 (ask) |
| Window ▸ Right dock (region row) | "Right dock" | "Context panel" on every form factor; on the phone it shows or hides the selection sheet (Ruling DX PQ13) | PC-4 (inspect) |
| Window ▸ Context panel ▸ Inspect, Pin, Follow | `_todo`, each with its reason | live (Inspect: Ctrl+1); on the phone Inspect opens the selection sheet | PC-4 (inspect + the method) |
| Window ▸ Context panel ▸ Note / Ask | `_todo` | live, Ctrl+2 / Ctrl+3; on the phone Note opens the vault at the selection's note and Ask the assistant | PC-5 / PC-6 |
| Window ▸ Open windows' Context panel row | absent | listed while the panel is shown | PC-4 (inspect) |
| Window ▸ Context panel ▸ MODULES' ten rows (Ruling DY; desktop and tablet) | `_todo`, each naming where its content lives today (`CTX_MODULE_TODAY`) | a check row, ticked while the module is in the panel; Ctrl+1/2/3 stay Inspect, Note and Ask | PC-4 (`context_module_registry()` lists that module) |
| Window ▸ Context panel ▸ Reset panel (Ruling DY; desktop and tablet) | `_todo`, `CTX_RESET_TODO` | live: back to Inspect · Note · Ask | PC-4 (`reset_context_panel`) |
| Story ▸ Journey planner's tooltip | shipped (`STORY_PLANNER_TIP`) | "its own window" (`STORY_PLANNER_WINDOW_TIP`), the desktop only | PC-8 (`planner_in_window()` answers true) |

**`90_pc_milestone_stubs.gd` must not be pasted before its milestone.** Its bodies answer "nothing" honestly (`false`,
`-1`, an empty tab list), so an early paste lights no Context panel row; but it would switch Deselect's and the
Factions rows' tooltips to wording that is not yet true. Each milestone writes the real body and deletes the
`STUB(PC-n)` marker; `check_symbols.sh` fails while a marker is in the shipped tree.

## Accelerators: the collision check

Command (repository root), section 6b of its output:

```
bash cartalith-native/drafts/pc-shell/check_symbols.sh
```

Its core grep is `grep -n -E '(_live|_bind_accelerator)\(.*KEY_[A-Z0-9_]+' shell/menus.gd`, comment lines and
`_live`'s own signature dropped; then every other `KEY_` match of the new keys in every shipped script. Output at
`a38e0ac`, 2026-11-01:

```
  Ctrl+A         ID_SELECT_ALL                menus.gd:1422
  Ctrl+C         ID_COPY                      menus.gd:1388
  Ctrl+D         ID_DESELECT                  menus.gd:1440
  Ctrl+F         ID_FIND_ON_MAP               menus.gd:1453
  Ctrl+N         ID_NEW_WORLD                 menus.gd:804
  Ctrl+O         ID_OPEN_PROJECT              menus.gd:806
  Ctrl+R         ID_WORLD_RUN_PIPELINE        menus.gd:7426
  Ctrl+S         ID_SAVE                      menus.gd:838
  Ctrl+Shift+E   ID_EXPORT_PLAYER             menus.gd:3172
  Ctrl+Shift+M   ID_WIN_SESSION_MODE          menus.gd:6095
  Ctrl+Shift+N   ID_STORY_NOTES_HERE          menus.gd:3361
  Ctrl+Shift+P   ID_AP_EXPORT                 menus.gd:2766
  Ctrl+Shift+S   ID_SAVE_AS                   menus.gd:840
  Ctrl+Shift+T   ID_STORY_TIMELINE            menus.gd:3331
  Ctrl+Shift+Z   ID_REDO                      menus.gd:1242
  Ctrl+V         ID_PASTE                     menus.gd:1390
  Ctrl+W         ID_CLOSE                     menus.gd:896
  Ctrl+X         ID_CUT                       menus.gd:1386
  Ctrl+Z         ID_UNDO                      menus.gd:1228
  DELETE         ID_DELETE                    menus.gd:1402
  Shift+A        ID_ASSET_LIBRARY             menus.gd:2126
  Shift+D        ID_WIN_DIAG_OVERLAY          menus.gd:6111
  Shift+L        ID_TRAVEL_LIBRARY            menus.gd:3347
  (23 bindings)
  new chord      intended command              state
  Ctrl+R         ID_WORLD_RUN_PIPELINE        bound to its own command (expected once its row is pasted)
  Ctrl+Shift+F   ID_FACTIONS                  free
  Ctrl+Shift+V   ID_VAULT                     free
  Alt+LEFT       ID_EDIT_SELECT_BACK          free
  Alt+RIGHT      ID_EDIT_SELECT_FORWARD       free
  Ctrl+1         ID_WIN_CTX_INSPECT           free
  Ctrl+2         ID_WIN_CTX_NOTE              free
  Ctrl+3         ID_WIN_CTX_ASK               free
  Ctrl+J         ID_STORY_ASSISTANT           free
  Shift+F10      ID_EDIT_CONTEXT_CARD         free
  Q              ID_EDIT_CARTWHEEL            free
  every other match of those keys in a shipped script (not menus.gd's bindings, not comments):
    ./shell/context_card.gd:851:		KEY_RIGHT, KEY_LEFT:
    ./shell/context_card.gd:852:			var right := k.keycode == KEY_RIGHT
    ./shell/workspaces/world_workspace.gd:2978:			freehand_key.keycode = KEY_F
    ./shell/layers_popover.gd:869:		ev.physical_keycode = KEY_1 + i
    ./shell/app.gd:525:	if event is InputEventKey and event.keycode == KEY_Q and not event.pressed:
    ./shell/app.gd:542:	elif event.keycode == KEY_Q and not event.echo:
    ./shell/app.gd:584:	elif event.keycode == KEY_MENU or (event.keycode == KEY_F10 and event.shift_pressed \
    ./shell/app.gd:601:	elif event.keycode == KEY_J and event.shift_pressed \
  tool-button letters (exact-match Shortcuts, dcc_widgets.gd::_tool_shortcut): B I L M R S T V W ⇧R ⇧T + Freehand's F (world_workspace.gd)
  project.godot [input] actions: 0 (0 means only Godot's ui_* defaults and the Layers popover's runtime 1-8)
```

The 23 bindings are the proposal §1's list exactly. Reading the "other matches":
- `context_card.gd` Left/Right and the Layers popover's 1-8 act only while that surface is open. **While the Layers
  popover is open, Ctrl+1-3 pick a layer**: `layers_popover.gd::_input` calls `is_action_pressed()` with its default
  `exact_match = false`, so an unmodified action matches a Ctrl chord. Popover-local; left alone.
- Freehand's F and every tool letter are exact-match `Shortcut`s, so no modified chord fires them. No tool uses Q or J.
- `app.gd:601` is ⇧J, which excludes Ctrl, so it cannot meet Ctrl+J. Lines 525, 542 and 584 are the Q and Shift+F10
  code 09 removes (it keeps `KEY_MENU`).
- Alt+← in a focused `LineEdit`: macOS takes it as word-left first; Windows and Linux do not, so the press reaches the
  row. Ruling DX, PQ12: it must not move the selection, so `_on_edit_selection()` drops a press typed into a field
  (`_typed_into_field()`); a click on the row still acts.

After the drafts are applied, re-run it: each new chord should read "bound to its own command" once its row is live,
and nothing should read COLLISION. **Ctrl+R then has two code lines, both `ID_WORLD_RUN_PIPELINE`**: File's
(`_build_generate_world_row()`) and the tablet's World (`_world()`). That is one command on two bars, never two rows
on one bar (Ruling DY: the short tablet bar's File leaves its row out, `_generate_in_world`), so the table reads it as
bound to its own command.

## Search and the reference: the EXTRAS and UNLISTED rows

`command_index.gd::EXTRAS` (07) gains four rows with a new `alias` key, and one pointer row. An alias is skipped while
a live menu row still carries its exact title, so each appears only once its title has left the bar:

| Title | Blurb (abridged) | Appears |
|---|---|---|
| Import image | Assets ▸ ⧉ Asset library (Shift+A), then the library's own Import image… | now (06 removes the row) |
| Notes for this place | Now Story ▸ Note for the selection (Ctrl+Shift+N) | from PC-5 |
| Right dock | Now the Context panel: Window ▸ Context panel | from PC-4, on every form factor (Ruling DX PQ13) |
| Run pipeline | Now Generate world (Ctrl+R): File ▸ Generate world, or World ▸ Generate world on a tablet whose bar has a World menu | now (02's W5 renames the tablet's row, Ruling DX PQ11; W6 puts the PC's in File, Ruling DY) |
| Records (was World) | **a pointer row, not an alias** (Ruling DY): the menu called World is Records now, its rows listed, Generate world in File | now (02's W1 renames the menu) |

No alias for renames whose words still match the new title ("political history", "assistant", "keyboard"), or for
titles still on the bar elsewhere (Records' "Timeline" → Story ▸ Timeline; Story's "Places without notes" → Records'
row). Counted by TITLE, never by `search()`.

**Why the World pointer is not an alias titled "World".** The index files every menu row under its menu's name
(`group`) and `search()` matches groups, so after the rename the word "world" no longer reaches Factions…, Places…,
the vault or the data rows. An alias titled "World" would never appear: Window ▸ Workspace ▸ World is a live row with
exactly that title, and `_has_menu_title()` skips an alias while one exists. A pointer titled "World" would sit
beside that radio row and mean something else. "Records (was World)" is found by both words.

`shortcuts_dialog.gd::UNLISTED` (08) gains one row: `["Map view", "Menu", "Open the context card: the same as Edit ▸
Context card, whose key can be rebound", "the main window"]`. No §3.8 change moves an accelerator off a menu: Import
image, Records' (then World's) Timeline and Story's Places without notes had none. Q and Shift+F10 move ONTO menu rows, so the walk of
the live menus lists them and `Preferences ▸ Keyboard shortcuts…` can rebind them (finding M8). The Menu key cannot
follow, because one `PopupMenu` item holds one accelerator (the Ctrl+Y precedent).

## Probes to update in the same change

Each was read at its own lines; none was run (no Godot here).

| Probe | What changes | Draft |
|---|---|---|
| `_storymenus_probe.gd` | **Ruling DY:** `DESKTOP_TITLES`' "World" becomes "Records"; `TABLET_TITLES` becomes `["☰", "File", "World", "Records", "Story"]`. The rows check reads `_menu_button("Records")` on both form factors (so does the Travel library check near its end), and `WORLD_ROWS` is Records' whole list, with no head to slice: Generate world is not in it. On the tablet, World's rows must equal `TABLET_WORLD_HEAD` exactly, its last `"|"` dropped (World no longer runs on into the records rows). The submenu, Export and Places without notes… checks read Records. New: File's second row is "Generate world" on the desktop and absent from the tablet's File. **PC-9:** `WORLD_ROWS`' "Timeline" becomes "Timeline is in Story ▸ Timeline"; drop the `id_pressed.emit(DccMenus.ID_WORLD_TIMELINE)` check (no row carries it); `STORY_ROWS`' "Places without notes…" becomes "Places without notes is in Records"; drop the Story-popup press of `ID_PLACES_WITHOUT_NOTES` (Records' press stays). Its Ctrl+Shift+N and "Notes for this place" checks hold until PC-5 | 02, 04 |
| `_storymenus_probe.gd`, again for W5 | `TABLET_WORLD_HEAD`'s first entry "Run pipeline" becomes "Generate world" | 02 |
| `_sessionmode_probe.gd` | `GATED_DESKTOP` gains "File > Generate world" (Ruling DY; the phone run builds the same row); `GATED_TABLET`'s "World > Run pipeline" becomes "World > Generate world" (W5, Ruling DX PQ11), and its other World rows stay (the tablet's World is `_world()`). `_refresh_menus()`'s title list needs nothing: Records has no gated row | 02 |
| `_tabletparity_probe.gd` | Ruling DY: desktop titles `File,Edit,Assets,Records,Story,Preferences,Window,Help`; tablet titles `☰,File,World,Records,Story` (until MS-7; from it a tablet from 1 180 dp shows the desktop's). Its title census counts "Generate world" once on each bar (File on the desktop, World on the tablet), which Ruling DX PQ11 and DY together make true | 02 |
| `_factionhub_probe.gd` | `_find_menu_button(_app, "World")` becomes `"Records"`: Factions… is Records' row on every bar, the tablet's included (its comment says the opposite of the short bar now) | 02 |
| `_v3menu_probe.gd` | `_data_popup()` finds `"Records"`; its messages "World ▸ Markdown vault" read "Records ▸ Markdown vault" | 02 |
| `_vfyb13_probe.gd` | `MENUS`' "World" becomes "Records" (the phone's eight menus) | 02 |
| `_structmove_probe.gd`, `_vfy_batch0905_probe.gd` | `_menu("World")` becomes `_menu("Records")` with their messages; the latter's bar walk list gains "Records" (keep "World" for a tablet run) | 02 |
| `_adaudit_probe.gd`, `_adpress_probe.gd` | the `b.text == "World"` tests become `"Records"` (each walks the records popup) | 02 |
| `_phonemore_reach_probe.gd` | `SCREEN_COVERS["data"]` becomes `"Records"`, with EDIT W8; it retires with the `data` screen when `drafts/mobile-shell/05` lands | 02 |
| `_idxfind_probe.gd` | add `"Import image": 1, "Notes for this place": 1, "Right dock": 1, "Run pipeline": 1, "Records (was World)": 1` to `want` (it builds with no app, so every alias and pointer appears) | 07 |
| `_cmdindex_probe.gd` | no edit expected: its pinnable list files the records rows under Records and lists Generate world once per bar, so "0 duplicate pin_keys among pinnable rows" must stay green | 02 |
| `_menuwire_probe.gd` | no edit: every disabled row states a reason, the Context panel's sixteen `_todo` rows (five, ten modules, Reset panel) included | 05 |
| `_ctxring_probe.gd` (windowed, clean user dir) | no edit: its Q and M legs push Q, the Menu key and Shift+F10 through `_vp.push_input()`, which now reaches the rows' accelerators for Q and Shift+F10. They must stay green | 03, 09 |
| `_seedlayout_probe.gd`, `_appbar20_probe.gd`, `_llmplace_probe.gd`, `_llmpanel_probe.gd` | no edit now; each reads a label that changes only at its milestone (PC-4: "Right dock" on the desktop readout, and since Ruling DX PQ13 the phone's row in `_appbar20_probe.gd` too; PC-6: "Assistant panel…") | 04, 05 |
| `_vfy_r2_menus_probe.gd`, `_apcmdcheck_probe.gd` | census dumps; "Import image" now title-matches the alias row | 06, 07 |
| `_structmove_probe.gd` | **Ruling DZ:** "2b control: Bake ALL & finalize is still on the WORLD bar" becomes: the bar holds no such button, and the WORLD dock's Finalize foot holds one | 11 |
| `_vfy_batch0905_probe.gd` | **Ruling DZ:** "A2 control: Bake ALL & finalize is still on the WORLD tool-options bar", the same | 11 |
| `_ds03fit_probe.gd` | **Ruling DZ:** its band-height check was written against the five buttons WORLD's idle row held; arm a tool whose row has `DccWidgets.action()` buttons (Way, Route, Conflict, Territory or Lasso) before it measures, or it passes without exercising the case | 11 |

## Verifying, after pasting

1. `check_symbols.sh` before pasting (all pass), then the parse check after every edit, then `cargo test --workspace`
   (no Rust changes, so unaffected) and the parse check on every touched file plus `shell/app.gd` (`MISTAKES.md`,
   "Declare the work green").
2. The probes above, windowed where they read pixels or route input.
3. By hand, desktop (each draft's header has its own list): the bar reads File · Edit · Assets · Records · Story ·
   Preferences · Window · Help; File's second row is Generate world · Ctrl+R; Ctrl+Shift+F opens Factions and
   Ctrl+Shift+V the vault; Records' RECORDS band ends with the Timeline signpost; a search for "world" lists Records
   (was World); Edit shows Previous and Next selection dark with their reason; Shift+F10 and Q work over the map and do
   nothing while a settlement name is typed (and from PC-1 Alt+← does nothing while one is typed); Window shows Right
   dock, then Context panel ▸ with five dark rows, a MODULES band of ten dark rows (each naming where its content
   lives today) and a dark Reset panel; Help ▸ Keyboard reference… opens a
   read-only window titled Keyboard reference listing the two new Edit rows and the Menu key; Assets has no Import
   image row and search for "import image" finds the alias; a popped-out roster appears in Window ▸ Open windows.
4. Tablet (`--force-touch`): the bar reads ☰ · File · World · Records · Story (Ruling DY; measure it at 600, 768,
   1 024 and 1 179 dp, windowed: every title inside the bar, none clipped, none over the quick cluster); World is
   Generate world · Ctrl+R (W5), the two resets, Generation pipeline / Sculpt and Seed & size…, and nothing else;
   File has no Generate world; Records is the desktop's; the same Edit and Window rows under ☰ ▸ Edit and ☰ ▸ Window,
   the MODULES band included; no hover switching. Phone: MENU ▸ Edit has neither map row; MENU ▸ Window has Right dock
   (Context panel from PC-4) and the Context panel submenu with five dark rows and no MODULES band; MENU lists Records
   where it listed World, and MENU ▸ File's second row is Generate world.

## Not drafted, and why

- **§3.8's three-level flattening (finding M7).** Inventoried below, code left for later. The proposal names the five
  submenus but not where each third-level row goes, and the obvious mechanical answer (inline the third level as a
  labelled band in its parent, same ids) breaks the phone. `phone_menu.gd::_expandable()` draws a submenu as ONE chip
  row when it holds two or more radios and no nested submenu, so inlining "LOD levels" into "Tile size · LOD levels"
  would put twelve chips of two settings in one row, and `_stamp_pref_values()` (which reads the single checked row,
  `_sole_checked_text()`) would lose the group's readout. The phone's rule has to learn to split radio groups at
  separators first, and the owner has to say what each third level becomes.
- **§3.9's Escape** ("closes the topmost transient … With nothing open, it deselects"): a change to
  `app.gd::_escape_action()` and a transient stack, not a menu row, and the deselect half is PC-1's. `DESELECT_ONE_TIP`
  says Escape leaves the selection alone, which stays true until that lands.
- **§3.8's "Desktop search runs commands"** (finding S4): the search list's code (`dcc_shell.gd`'s command bands), not
  a menu.
- **CIVIL ▸ Open factions… on the current faction** (W6's third entry point): `civilization_workspace.gd`, not a menu.
  It should call the same `current_faction()`.
- **`assistant_chip.gd::open_panel` and the ring-dock assistant item → the Ask tab** (A4): not menus; PC-6's.
- **§3.1's S3, the card at the selection:** `app.gd::_open_context_card_from_keyboard()`'s body, at PC-1.
  `CONTEXT_CARD_TIP` says "at the pointer" and must change with it.
- **Ruling DY's PQ5 preference, "save the assistant conversation with the world"** (off by default, asked at first
  use, saved inside the world file). Not a menu row: the drafts carry no Preferences rows, and the assistant's
  settings are one row, Preferences ▸ Assistant… (`ID_PREF_ASSISTANT`, 900), which opens `LlmSettingsWindow`. The
  switch belongs in that window beside the consent and profile switches; the first-use question in the Ask tab (PC-6);
  and the conversation needs a save slot that nothing has built (Rust, `project_bridge.rs`, with a
  `SAVEFILE_COMPAT.md` row when it exists). Only the session half reaches the menus: `STORY_ASK_TIP` (04).
- **The Context panel's own module controls** (Ruling DY): the strip's +, dragging a tab, a tab's Remove and Open in a
  window. They are the panel's (PC-4), not a menu's; `90_pc_milestone_stubs.gd` states the store they share with
  the MODULES rows (`context_panel_modules()`, `set_context_module_shown()`).
- **The planner's window itself** (Ruling DY, PQ4, milestone PC-8): `90_pc_milestone_stubs.gd`'s PC-8 contract
  (an `AcceptDialog`, `exclusive = false`, its key in `PopoutHost.LIVE_KEYS` with the per-window Pop out button). The
  menus need only the tooltip switch (04).

### M7 inventory (read at the builders, `a38e0ac`)

| Submenu | Its rows, at each level | Refreshed by / consumers | Why it is not a mechanical flatten |
|---|---|---|---|
| Assets ▸ Landmark types ▸ family ▸ type (`_build_landmark_types_menu()`, `_build_landmark_family()`) | L2: one submenu per family in the engine's first-seen order (`landmark_kinds()`), "Placed by hand   N" (`ID_LM_HAND`), Landmark icons… (`ID_LM_ICONS`), Landmark label style… (`ID_LM_LABELS`). L3 per family: one state row per type (index ids: "Peak   3 max · 2 placed · …", "off · was N", "not buildable"), "Open *Family* in the dock" (`ID_LM_FAMILY_FIRST + i`), a signpost | `_refresh_landmark_menu()` rewrites every type row; `phone_menu.gd::_landmark_sub()`; `_landmark_probe.gd`, `_adpress_probe.gd`; `command_index.gd::_is_index_popup()` and `_is_dynamic_id()` (510 + 64) | Engine-sized and per-type state with no other menu home; the tablet's ☰ promotes it to its top level. Needs a decision: family rows with counts and the types in CIVIL ▸ Landmarks, or bands |
| Preferences ▸ Lighting rig defaults ▸ Azimuth / Elevation / Ambient / Multidirectional ▸ rung (`_build_lighting_menu()`, `_light_ladder()`) | L2: source readout, four ladders, Use the engine's own rig (`ID_LIGHT_RESET`). L3: 8 + 7 + 6 + 6 = 27 radio rungs (170-177, 190-196, 210-215, 230-235) | `_refresh_lighting_menu()`, `_light_ladders()` (each ladder addressed as its own popup), `_on_lighting()` | 27 radios in one popup is one chip row of four settings on the phone; the four ladder tooltips live on L2 rows a band cannot carry |
| Preferences ▸ Tile size · LOD levels ▸ LOD levels (`_build_atlas_tiles_menu()`) | L2: 256 / 512 / 1024 px radios (140-142), "LOD levels" with its tooltip. L3: 9 radios (150-158) | `_refresh_tile_size_menu()`, `_refresh_lod_levels_menu()`, `_on_tile_size()`, `_on_lod_levels()`; `_stamp_pref_values()` | merges two radio groups (phone chips, the stamped readout) |
| Preferences ▸ Atlas cache ▸ Size cap · GB (`_build_atlas_cache_menu()`, `_build_atlas_cap_menu()`) | L2: store readout, Size cap (or a `_todo` without `atlas_evict_to`), Refine detail…, Refine viewshed…, viewshed readout, Export / Import atlas…, Clear atlas cache now…. L3: 7 radios (630-636) | `_refresh_atlas_cap_menu()` indexes `_atlas_cap_popup`; AtlasCache's `about_to_popup` evicts (`_enforce_atlas_cap()`), so it is in `phone_menu.gd::NO_EXPAND` and `command_index.gd::UNSAFE_OPEN_HANDLERS` | feasible as a band, but every refresher and both safety lists address the L3 popup by name |
| Window ▸ 3D capability ▸ Force tier (`_build_view3d_rows()`) | L2: "3D tier" readout, Re-test 3D (801), Force tier. L3: Automatic (802) and four tiers (803-806) | one lambda sets L3 checks by index; `force.id_pressed` | the most mechanical of the five (a "FORCE TIER" band, the lambda by id); the phone would then draw the five as chips, which is acceptable. Left with the rest so M7 is one decision |

## Where the shipped code and `PC_SHELL_PROPOSAL.md` disagree (reported, not changed)

1. **"Q and Menu or Shift+F10 move into the rebindable table."** The table is the menu accelerators: `DccSettings`'
   only populated context is `SHORTCUT_CONTEXT_MENU`, filled by `menus.gd::_bind_accelerator`. So each key needs a menu
   row, which neither §3.8 nor the mockup placed (K1, answered by Ruling DX PQ10). And only Shift+F10 can move: one `PopupMenu` item holds one
   accelerator, so the Menu key stays in `app.gd`, fixed, listed in `UNLISTED`.
2. **"Generate world, Ctrl+R … The tablet's Run pipeline row is the same command."** Same id, drafted so. The labels
   differed by form factor, one command under two names (finding M2's own complaint about Import image). Ruling DX
   PQ11 settles it: "Generate world" everywhere (W5), with a "Run pipeline" search alias (07).
3. **"'Right dock' becomes 'Context panel', with a submenu."** A `PopupMenu` row cannot be both a check item and a
   submenu. The mockup draws two rows titled "Context panel" (the check, then the submenu); drafted that way.
4. **"…of the selection" for Political history.** The mockup drops the ellipsis (the row opens a panel tab, not a
   window); drafted per the mockup.
5. **Help's row is already read-only.** `ID_HELP_SHORTCUTS` opens `ShortcutsDialog.open()` today; M2 is the label, and
   the dialog's own title ("Keyboard shortcuts" in both modes), which 08 fixes.
6. **"Free" keys, with two scoped exceptions** the proposal does not mention: Ctrl+1-3 pick a layer while the Layers
   popover is open; Alt+←/→ reach the menu while a text field has focus on Windows and Linux, where the handler now
   drops them (Ruling DX PQ12).
7. **The phone gets "the menu rows" (§3.11)**, but its menu bar is a hidden model (`dcc_shell.gd`, `PhoneMenuModel`), so
   no accelerator fires there. Moving Q and Shift+F10 onto rows therefore drops them for a phone with a hardware
   keyboard (stated in 03 and 09); the Menu key and the phone's long-press card remain.
8. **"Open windows lists everything open."** While an exclusive dialog is open the menu bar takes no input, so today the
   list can only ever show pop-outs, which the shipped hand list of six omitted. The fix matters most after PC-7.
9. **"Desktop menus open as side-by-side columns."** Godot's submenus already open beside their row; only "moving
   across the bar switches menus" is missing (`switch_on_hover`, draft 10).
10. **PC-9's bar ("every row in §3.8") cannot be met before PC-1, PC-4, PC-5 and PC-6**: four of its rows are those
    milestones' behaviour. The drafts deliver it progressively, with `_todo` rows meanwhile.
11. **The rename reaches past `menus.gd`** (Ruling DY). Five user-visible strings in four other files say "World ▸
    Markdown vault" or "World ▸ Import" (02's W9-W13), File's own signpost says "World ▸ Import" (W7), and the
    phone's shipped MORE root finds the menu by its title (`phone_menu.gd::_fill_data()`, `_named_in_root()`: W8,
    until mobile draft 05 retires them). Two more strings were already wrong: they send the reader to "World ▸ Travel
    library", which moved to Story with SM-2 (W14).
12. **Search cannot alias the old menu name as "World"** (Ruling DY): Window ▸ Workspace ▸ World is a live row with
    that title, so `_has_menu_title()` would hide the alias for good. The pointer is titled "Records (was World)" (07).
13. **The planner's window must be an `AcceptDialog`** (Ruling DY, PQ4): `_visible_windows()` walks `AcceptDialog`s for
    Window ▸ Open windows, and `PopoutHost.add_toggle_button()` takes one. A plain `Window` would be missing from
    both.

## Questions the drafts raised, answered (Ruling DX, 2026-11-01)

The drafts' K1-K3 were asked as `PC_SHELL_PROPOSAL.md` PQ10-PQ12; the owner answered them with PQ13 after the menu
parity check (`cartalith-native/docs/evidence/menu-parity-2026-11-01/`). The drafts now carry each answer:

- **K1 / PQ10.** The card's and the cartwheel's rows: Edit, after Find on map… (Context card · Shift+F10, Cartwheel
  at the pointer · Q), on the desktop and the tablet, **not on the phone** (a stated exception to the phone's row for
  row). As drafted (03).
- **K2 / PQ11.** One label: **"Generate world" on every form factor.** The tablet's row is renamed (02, EDIT W5) and
  search keeps "Run pipeline" as an alias (07).
- **K3 / PQ12.** **No**: Alt+← and Alt+→ do not move the selection while a text field has focus (03,
  `_on_edit_selection()` with `_typed_into_field()`).
- **PQ13** (decided under the owner's delegation): the phone's Window menu follows the mockup, with the Context panel
  check and its submenu and no separate Right dock row; on the phone they reach the selection sheet, the vault and the
  assistant (05, and `90_pc_milestone_stubs.gd`'s phone contract).

## What Ruling DY changed (owner, 2026-11-01)

The owner answered PQ1-PQ9 and their follow-ups (`LARGE_ITEM_RULINGS.md`, Ruling DY). What the drafts carry:

- **PQ6, "Rename to 'Records'"**, then **Generate world in "File, after New world…"**, then the tablet's short bar
  **"File · World · Records · Story"**: 02 (W1, W6-W14; `_tablet_world()` deleted, nothing dropped), 08 for the
  whole-bar `build()` (`drafts/mobile-shell/`), 07 (the "Records (was World)" pointer; "Run pipeline" now names
  File), the Records wording in 04, 05 and 90, and the probe rows above. One labelled judgement: the short bar's File
  leaves Generate world out because its World holds it (`_generate_in_world`), so no bar lists the command twice.
- **PQ1, "1, but it should be modular"**, then **"Tabs you arrange"** with ten modules (History; Economy; Military;
  Faith, culture & relations; Territory; City layout; Here; Routes & journeys; Links; Recent & bookmarks): 05's
  MODULES band and Reset panel (desktop and tablet), the ids 920-930 (01), and 90's PC-4 registry and arrangement,
  which name each module's builder in its window today. Judgements: Ctrl+1/2/3 re-add a removed core tab
  (`open_context_tab()`'s contract), and the panel refuses to lose its last tab.
- **PQ4, "Planner should open in a separate window that doesn't block the map"**: 90's PC-8 contract and
  `planner_in_window()`, and 04's two-form Journey planner tooltip.
- **PQ5, one conversation per session, with an opt-in to save it** (off by default, asked at first use, inside the
  world file): 04's `STORY_ASK_TIP` states the session half; the rest is listed under "Not drafted".
- **VQ4 (the vault editor), "Open in Write"**: 05's `CTX_NOTE_TIP` says the Note tab opens in Write, a judgement that
  the Note tab follows the editor it embeds (`PC_SHELL_PROPOSAL.md` §3.4 says it is that editor).
- **PQ2, PQ3, PQ7, PQ8 and PQ9** took their defaults, which the drafts already assumed; nothing here contradicts
  them (PQ8's keys are the ones drafted).

## What Ruling DZ changed (owner, 2026-11-01)

The owner, verbatim: *"The buttons in the top rail new seed, generate and bake can be removed. Please see to only keep
the paint/sculpt brush settings here."* Asked about the rest, the owner chose "Tool settings only"
(`LARGE_ITEM_RULINGS.md`, Ruling DZ; `PC_SHELL_PROPOSAL.md` §3.12).

Draft 11 carries it:
- **WORLD's idle row** becomes the context name and one hint.
- **Generate world, New seed, Center landmasses and Bake ALL & finalize** leave the bar. Each keeps its WORLD dock
  home, and Generate world keeps File ▸ Generate world (draft 02).
- **The bake shortcut's pushed state goes with them:** `set_bake_shortcut()`, `bake_and_finalize()` and the push in
  `_refresh_finalize()`.
- **Other tools' rows are untouched:** their settings and their Commit and Discard stay.

The planner's row (`journey_planner_view.gd::_tool_options_journey()`) moves into its window with PC-8. That is
noted in draft 11, and stubbed in 90's `planner_in_window()`.

`check_symbols.sh` checks draft 11's names, its anchors and the fingerprints of the three blocks it replaces or
deletes.

## Stated drops (`MISTAKES.md`, "Delete a surface")

- A quick-bar pin on Assets ▸ Import image (`menu:32`) or World (now Records) ▸ Timeline (`menu:722`) reads "not in this build". Each
  did what ⧉ Asset library and Story ▸ Timeline do; pins on those keep working.
- Ctrl+Q and Shift+Q no longer open the cartwheel (the old test ignored modifiers; an accelerator does not).
- A phone with a hardware keyboard loses Q and Shift+F10 (item 7 above).
- None from Ruling DY: `_tablet_world()`'s rows each keep a home on the same bar (02's inventory), and Generate
  world keeps its id, key, gate and tooltip, so a rebind or a pin (`menu:710`) follows it into File.

## `MISTAKES.md` rows to carry into the PC session

Call a shell helper (the vocabulary is `_live` / `_todo` / `_readout` / `_signpost`; `check_symbols.sh` re-checks
every call) · Move a command off the menu bar (07 and 08 in the same change; assert by TITLE) · Change behaviour (each
header's PROSE list) · Delete a surface (the inventories in 06 and above) · Return, marshal or default a value that can
be absent (`current_faction()` answers -1, `open_faction_roster()`'s own "none"; `selection` is `{}`) · Dash a field
with a reason (every `_todo` reason was checked at its symbol; re-check it when the milestone is near) · Re-run the
parse check after EVERY edit · Run `--check-only` from the project root with `--script` · Run `godot --headless
--import` (none of these drafts adds a `class_name`) · Add a probe in the same commit (a probe must fail before the
change) · Declare the work green.
