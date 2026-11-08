# Menu parity: Godot against the mobile-shell mockup (2026-11-01)

Owner, verbatim: *"check the godot version of all menu's against the mockups in the artifact. Per platform and
option if they are actually the same."*

Evidence only. It records what differs and why. It is not a status record: how far any milestone has got is
`cartalith-native/docs/STATUS.md`'s answer.

## What was compared

- **Mockup:** `mockup_menus.json`. `mockup_menus.js` evaluates the mockup's own `menuTree()`, `tabletOverflow()`,
  `tabletFileMenu()`, `tabletWorldMenu()` and `ctxModuleMenuItems()` from `design/mobile-shell-2026-10-31/Shell.dc.html`.
  Both mockup files were regenerated on 2026-11-01, after Ruling DY, with `node mockup_menus.js <Shell.dc.html>` and
  `--md`.
- **Godot:** `godot_menus.json`, written by `godot_menus.py`. It covers the shipped shell at `a2200e5b`, with these
  drafts applied in memory: `drafts/pc-shell/` 01-06 and their EDITs (Ruling DY's W6 and W7 in 02 included);
  `drafts/mobile-shell/` 05, 08 and 09, with 09's EDIT G1. This session has no Godot binary, so `godot_menus.py` interprets the menu builders. Every assumption it makes
  is in its `CONDITIONS` and `RUNTIME` tables (a fresh install, 8 logical cores). The interpreter left **0** conditions
  unresolved on every platform, in both states.
- **Per platform:**
  - **desktop:** `build()`'s eight menus.
  - **tablet ≥ 1 180 dp:** draft 08's whole bar.
  - **tablet < 1 180 dp:** draft 08's short bar, `☰ · File · World · Records · Story` (Ruling DY, PQ6). It was
    the shipped `☰ · File · World · Story` until then.
  - **phone:** MENU, which is draft 05's root; since Ruling DX Q13 each of its eight rows opens that menu's own
    popup, row for row.
- **Two Godot states:**
  - **today:** PC-1, PC-4, PC-5 and PC-6 not built, so each row takes its drafted fallback.
  - **after:** those four milestones built.

  Rows are compared in the after state. The *Godot today* column is filled only where today's form differs.
- **Per row:** label, kind, accelerator (not on the phone, which draws none), disabled state and its reason,
  Session-mode gate, and submenu contents. A run-time list is compared by its pattern. On the phone, a radio submenu
  drawn as chips would be compared by its options, not its shape; since Q13 MENU draws none (`_fill_popup()` drills).

Commands behind every count below:

```
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/godot_menus.py   # rows per platform and state; unresolved 0
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/compare.py       # this file; exits 1 if a difference is unclassified
```

## Totals

*mockup rows* and *Godot rows* count each side's rows once, at every depth (on the phone, the root's eleven included). *rows* counts the tables' rows: a run-time Godot row paired with several of the mockup's sample rows counts once per sample. Each row that is not *same* counts once more, under its class. Class (b) has no rows: its fixes are already applied on the Godot side.

| Platform | mockup rows | Godot rows | rows | same | same after its milestone | differs | mockup-only | Godot-only | a | c | m | s | n |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Desktop | 447 | 364 | 495 | 288 | 10 | 20 | 129 | 48 | 0 | 78 | 20 | 5 | 0 |
| Tablet, 1 180 dp and wider | 450 | 368 | 503 | 287 | 10 | 20 | 133 | 53 | 0 | 78 | 20 | 5 | 4 |
| Tablet, below 1 180 dp | 464 | 382 | 517 | 301 | 10 | 20 | 133 | 53 | 0 | 78 | 20 | 5 | 4 |
| Phone, MENU | 445 | 361 | 494 | 294 | 10 | 10 | 131 | 49 | 0 | 78 | 10 | 5 | 2 |

Rows per rule, per platform:

| Rule | Desktop | Tablet, 1 180 dp and wider | Tablet, below 1 180 dp | Phone, MENU |
|---|---:|---:|---:|---:|
| EH-2 | 78 | 78 | 78 | 78 |
| S1 | 3 | 3 | 3 | 3 |
| S3 | 2 | 2 | 2 | 2 |
| N1 | 0 | 2 | 2 | 2 |
| N2 | 0 | 1 | 1 | 0 |
| N3 | 0 | 1 | 1 | 0 |
| PC-4 | 6 | 6 | 6 | 6 |
| PC-5 | 2 | 2 | 2 | 2 |
| PC-6 | 2 | 2 | 2 | 2 |
| PC-10 | 10 | 10 | 10 | 0 |
| ? | 104 | 109 | 109 | 105 |

## Run-time labels (same at run time)

- **R1. Landmark types: a kind the engine cannot place.** The static read sees the build-time label (the kind's name); `menus.gd::_refresh_landmark_menu()` appends `   not buildable` when the menu opens, which is what the mockup draws since A1's fix.
- **R2. Atlas cache: the store readout.** The static read sees the placeholder `… chunks · … · empty`; `_refresh_atlas_cache_menu()` writes the value (`%d chunk%s · <size> · to LOD n`), which is what the mockup draws since A3's fix.

## (a) The mockup is wrong: the fixes (all thirteen made in `Shell.dc.html` on 2026-11-01; the counts above are after them)

- **A1. Landmark types: the 18 kinds the engine cannot place.** The two sides agree on which kinds can be placed: 49 kinds, 18 not buildable, the same set (the mockup's `LANDMARK_FAMILIES` `*` marks against `cartalith-civ`'s vocabulary, read by `godot_menus.py::landmark_kinds()`). They draw them differently. The mockup draws each as a `todo`, a disabled row. Godot keeps it as an enabled row that reads `<kind>   not buildable` (`menus.gd::_refresh_landmark_menu()`). Pressing it opens CIVIL ▸ Landmarks at its family, as every leaf does: `_build_landmark_family()`'s signpost reads *Rows here read state; they do not arm a type.* **Fix:** in `menuTree()`'s `lmFamilies`, map a starred kind to `{ t: 'live', label: k.slice(0, -1) + '   not buildable', act: 'node', arg: 'civil/landmarks' }`.
- **A2. Preferences ▸ Try the GPU again is drawn enabled.** Godot builds the row disabled. Its tooltip is `GPU_RETRY_NONE_TIP` (*Nothing to clear: no GPU readback has failed this session…*), and `_refresh_gpu_retry_row()` enables it only while a failure is on record. **Fix:** give the row a `liveReason()` rule that returns *Nothing to clear: no GPU readback has failed this session* unless the mock state records a failure. This replaces the toast.
- **A3. Lighting rig defaults ▸ Source and Atlas cache ▸ Store have names Godot does not draw.** Godot's rows are value only. `_refresh_lighting_menu()` writes `Following the engine`, `All four stored here` or `n of 4 stored here`. `_refresh_atlas_cache_menu()` writes `%d chunk%s · <size> · to LOD n` (or `empty`). **Fix:** `{ t: 'readout', label: 'Following the engine', v: '' }` and `{ t: 'readout', label: '85 chunks · 1.2 GB · to LOD 3', v: '' }`.
- **A4. Tile size · LOD levels ▸ LOD levels: `LOD 0–0   1 tiles`.** Godot writes the singular (`menus.gd`, `"LOD 0–%d   %s tile%s"`). **Fix:** `1 tile`.
- **A5. Window ▸ Workspace ▸ World carries no Session-mode gate.** Godot gates it: `_session_gate_row()` runs on every domain in `DccShell.SESSION_HIDDEN_DOMAINS`, and `SESSION_MODE_TIP` names the row. **Fix:** add `sm: true` to the `World` radio. It applies on every platform, ☰ ▸ Window and the phone's MENU ▸ Window included.
- **A6. Window ▸ Open windows: the Context panel's row label.** Both sides list the panel while it is shown, so the list is the same. The labels differ. The mockup's `openWindowItems()` writes `Context panel · <tab> · <subject>`. Draft `pc-shell/05` writes `CONTEXT_PANEL_LABEL` (`Context panel`). No PC-4 contract method returns the panel's current tab: `90_pc_milestone_stubs.gd` has `context_tabs()`, the tabs it has, and `open_context_tab()`. **Fix:** the label `Context panel`. If the richer label is wanted, PC-4's contract needs a current-tab query first.
- **A7. Phone: Preferences ▸ Open panels in their own window, drawn disabled as DESKTOP ONLY.** `POPOUT_WINDOWS_SCOPE.md` PW-5 says *the setting is absent on phone and disabled-with-reason on tablet*. Godot omits it on the phone (`PopoutHost.supported()`). **Fix:** `menuRows()` skips a `desktopOnly` item on the phone and keeps the DESKTOP ONLY row on the tablet (N3).
- **A8. Tablet < 1 180: ☰ ▸ `Keyboard shortcuts…`.** The ☰ row and Help's row are one builder, `_build_help_shortcuts_row()`. Draft `pc-shell/06` titles it `Keyboard reference…`, which is also what the mockup's own Help says. **Fix:** in `tabletOverflow()`, `{ t: 'live', label: 'Keyboard reference…', act: 'win:shortcuts', arg: 'ref' }`.
- **A9. Tablet < 1 180: ☰'s submenus repeat the rows ☰ promotes.** Godot builds them with `full = false`: `_edit(p, false)`, `_assets(p, false)`, `_preferences(p, false, true)` and `_help(p, false)`, called from `_overflow()`. So each row exists once. Undo, Redo and Undo history, ⧉ Asset library, Landmark types, Units, Keyboard reference… and About are at ☰'s top. Reset generation parameters and Reset one stage are in World. **Fix:** in `tabletOverflow()`, filter those rows (and the separators that framed them) out of the Edit, Assets, Preferences and Help submenus.
- **A10. Tablet < 1 180: World ▸ Generation pipeline and Sculpt carry no Session-mode gate.** Godot gates both (`menus.gd::_world()`, `_session_gate_row(p, mode_pipe_idx)` and `mode_sculpt_idx`). **Fix:** add `sm: true` to both radios in `tabletWorldMenu()`.
- **A11. Tablet < 1 180: World ▸ Generate world under Run pipeline.** `tabletWorldMenu()` appends World's items whole, so the narrow World has two rows for one command: id 710, Ctrl+R, `ID_WORLD_RUN_PIPELINE`, which `_pc_world()` and `_world()` both use. Godot's narrow World is `_tablet_world()` = `_world()` + `_data()`, with no Generate world. **Fix:** `.concat(w.items.filter((x) => x.label !== 'Generate world'))`. One label for the command on every form factor is K2.
- **A12. Preferences ▸ Save the assistant conversation with the world (after Ruling DY).** The mockup drew the PQ5 switch twice: as a check row in the Preferences menu and in the assistant's settings window. `PC_SHELL_PROPOSAL.md` §3.5 puts it in Preferences ▸ Assistant (the settings window), and no draft builds a menu row for it. **Fix:** the menu row is removed; the switch stays in the window, and the first-use card still sets it.
- **A13. Window ▸ Context panel ▸ MODULES: a separator before the band (after Ruling DY).** The mockup drew a plain separator and then the MODULES band. In Godot a captioned separator is one row (`sub.add_separator("MODULES")`, `pc-shell/05`), and the mockup's other bands carry no separator before them. **Fix:** the separator is removed from `ctxModuleMenuItems()`.

## (b) Drafts this check corrected

- **B1. `drafts/mobile-shell/08_menus_tablet_bar.gd`, `build()`.** Its whole-bar World was `_tablet_world if DccTheme.is_tablet() else _data`. That dropped the PC drafts' Generate world from the desktop, and it repeated Edit's two Reset rows in the wide tablet's World. It is now `_pc_world` (Ruling DU Q7, *the eight menus whole*). The header lists where each narrow-World row lives instead; README row 8 adds verify steps. `pc-shell/02` EDIT W1 and its README row now say to skip W1 when 08 is applied.
- **B2. `drafts/mobile-shell/05_phone_menu_root.gd`.** Three fixes. (1) `story-root` drew the old MORE section, which opened with a `Story` head and ended with an empty `World` head; `_fill_story_section()` is replaced. (2) The `MENU_SCREENS` comment claimed File's own order; it is corrected. (3) `_fill_popup()` dropped a leading captioned separator, which lost FOR THE TABLE, SOURCES and ACTIVE PACK on the phone; it is replaced, with only that test changed. README row 5 is updated.
- **B3. `drafts/mobile-shell/03_dcc_shell_phone_tabs.gd`.** Draft 03 renames the tabs (MS-2) and names the documents to correct with them, but not the two strings the app draws. EDIT H1 is added: MENU ▸ Help's closing line becomes *…reachable through MAP · DOMAIN · STORY · MENU.*, the mockup's own About wording. H2 flags the bottom-bar coach mark and the two probes that pin it; its wording is MS-2's to write. `check_symbols.sh` gains an anchor check for both strings, and README row 3 is updated.
- **B4. `drafts/mobile-shell/check_symbols.sh`.** The need list gains `_fill_popup _band _popup_row _clean STORY_ROWS`: 93 calls, up from 88. The anchor section is new.
- **B5. Ruling DX Q13: the phone's MENU follows the mockup (`drafts/mobile-shell/05` rewritten, `09` new).** Each of MENU's eight rows now opens its menu's popup row for row (`_menu_root_row()`, `_fill_popup()`), instead of the File, Assets, World, Story, Preferences and Help screens. The README's "Where each phone-only row goes" gives each of those screens' rows a home or a stated drop. The one that needed a row is Help ▸ Gesture reference… on the phone (draft 09, the mockup's own row, `ID_HELP_GESTURES` 919). Draft 09 also makes the landmark rows open the left sheet on the phone. Fourteen screens are left unreachable; the README lists them and the sixteen probes that reach them, to delete in the same change. Draft 03's H1 is withdrawn with the Help screen; H2, the coach mark, stays.
- **B6. Ruling DX PQ10: the card's and the cartwheel's rows.** As drafted: Edit, after Find on map…, on the desktop and the tablet, not on the phone (`03_menus_edit.gd::_build_map_surface_rows()` returns there). The mockup has taken them (`notPhone`).
- **B7. Ruling DX PQ11: one label, Generate world.** `pc-shell/02` EDIT W5 renames the narrow tablet's `_world()` row; W4's Session-mode wording follows. `pc-shell/07` adds a "Run pipeline" `EXTRAS` alias. The README lists the probes that name the row: `_sessionmode_probe.gd` and `_storymenus_probe.gd` change, while `_tabletparity_probe.gd` has no pair left. `pc-shell/check_symbols.sh` anchors W5 (27 anchors).
- **B8. Ruling DX PQ12: no Alt+arrows while typing.** `pc-shell/03`'s `_on_edit_selection()` drops an Alt+← or Alt+→ press typed into a text field (`_typed_into_field()`), as it already did for Q and Shift+F10; a click on the row still acts. The guard's doc in 01 and `select_back()`'s in 90 say so.
- **B9. Ruling DX PQ13: the phone's Window menu.** `pc-shell/05` builds the Context panel check and its submenu on the phone too: "Right dock" until PC-4, then "Context panel". `90_pc_milestone_stubs.gd` states the phone contract: Inspect shows the selection sheet, Note opens the vault at the selection's note, Ask opens the assistant, and Pin/Follow act on the sheet, which is the right dock presented as a sheet.
- **B10. Ruling DY (PQ6, PQ1): Records, Generate world in File, the four-menu tablet bar, the module rows.** `pc-shell/02` titles the World menu Records and moves Generate world (id 710, Ctrl+R) into File after New world… (EDIT W6); File's signpost names Records (W7). Both EDITs are transcribed into `godot_menus.py`'s `PC_EDITS`, which had stopped at W5, so the first run after the ruling showed File ▸ Generate world as mockup-only. `mobile-shell/08` builds the narrow bar as `☰ · File · World · Records · Story`, with World holding only the WORLD domain's rows and File leaving Generate world out there. `pc-shell/05` adds the MODULES band, ten check rows and Reset panel (rule PC-10). `pc-shell/07` keeps the old name findable through a pointer row titled "Records (was World)".

## (c) Owner decisions: all answered by Rulings DX and DY (2026-11-01); none is open

- **EH-2. Ruling EH item 2 (2026-11-08): Assets ▸ Landmark types is a window, not a cascade.** *Answered: a browsable list window, one `Landmark types…` row (`menus.gd::_build_landmark_types_row()`, `landmark_types_window.gd`).* The mockup's `lmFamilies` cascade (six family submenus and their kinds, the fixed icons, label-style and hand rows) is the stale side. The window shows every row the cascade showed, proved against a baseline dump by `_lmtypes_probe.gd`; the two destination rows stay findable through `command_index.gd`'s `EXTRAS`. **Fix when the mockup is next edited:** replace the cascade with one live row that opens a window.
- **K1. PQ10: where the card's and the cartwheel's rows live.** *Answered: as drafted, not on the phone (B6).* As asked: the drafts put Edit ▸ Context card (Shift+F10) and Cartwheel at the pointer (Q) after Find on map… on the desktop and both tablets. The mockup had neither row. *Default: as drafted.* If it is taken, add both rows to the mockup's Edit.
- **K2. PQ11: one label for the regenerate command.** *Answered: one label, Generate world (B7).* As asked: the command is Generate world on the PC and the wide tablet, and Run pipeline on the narrow tablet; it is the same command, id 710. It bears on A11. *Default: keep two labels.*
- **K3. PQ12: Alt+← / Alt+→ while a text field has focus.** *Answered: no (B8).* As asked: it bears on Edit ▸ Previous / Next selection; the rows match. *Default: yes, as a browser does.*
- **P1. Phone: MENU's File, Assets, Preferences and Help open the phone screens, which reorder the rows and add some.** *Answered (Q13): the phone follows the mockup (B5).* As asked: Ruling DT Q2: *the eight desktop menus row for row, in the same order and under the same names, plus Quick and Status*. The mockup's MENU follows it, and its band reads *same names, same order*. Draft 05 keeps `_fill_project()`, `_fill_assets()`, `_fill_prefs()` and `_fill_help()`. These draw:
  - File: Save, Save as, New world and Open project first; Recent worlds inline under a head; Autosave with Interval chips (`Interval`, not `Autosave interval`).
  - Assets: the nine families and the PACK values first.
  - Preferences: Theme and Units first.
  - Help: Runtime, Engine and Gesture reference first, and the closing info line, which the mockup puts in the About window.
  - The screens keep their own titles (`SCREEN_TITLES`), so MENU ▸ File opens a screen titled *Project*, Assets one titled *Asset library*, and Help one titled *Help & about*. The row comparison cannot see a title.

  `_fill_world()` also appends an *On the phone* band (Civilization, Data manager, Travel library, Simulation) that DT Q2 does not list. *Default: DT Q2 as written.* Those menus would open their popups row for row, as Edit and Window already do. Each phone-only row then needs a named home first (`MISTAKES.md`, "Delete a surface"). The candidates:
  - Runtime, Engine and the info line: the About window, as the mockup draws it.
  - Families and PACK values: the asset library and Assets ▸ Asset pack.
  - Civilization: the CIVIL sheet (MS-3).
  - Data manager: World ▸ ⧉ Data manager, already one of World's rows.
  - Travel library: Story ▸ Travel library….
  - Simulation (`_fill_sim()`: year, speed, playback, layers): no home is named yet. The nearest is the STORY tab's TIME band (Ruling DT Q1).
- **P2. Phone: Window ▸ Context panel.** *Answered (PQ13, delegated): the phone follows the mockup (B9).* As asked: the mockup draws the Context panel check and its submenu (Inspect, Note, Ask, Pin, Follow) on the phone. `pc-shell/05` keeps `Right dock` and no submenu there. It reads `PC_SHELL_PROPOSAL.md` §3.11 (*the phone gets the model, not the panel*). The same section also gives the phone *the menu rows*. *Default: as drafted.* If it is taken, the mockup hides both Context panel rows on the phone and draws `Right dock`.

## Same in substance (drawing only)

- **S1. Disabled-row reasons: Anti-aliasing · anisotropy, 3D viewport defaults, Help ▸ Documentation.** The mockup's reasons are short forms of Godot's tooltips, and they make the same claim.
- **S2. Phone: plain separators inside a menu screen.** *Gone with the screens (B5).* `_rest_of()` skipped an unlabelled separator, and `_add()` drew a hairline between every row. The mockup's phone draws a gap. Since Q13 MENU draws every menu through `_fill_popup()`, which keeps them.
- **S3. Window ▸ Open windows: the run-time list.** Both sides fill it when the menu opens. The mockup's static tree shows its empty state (*No windows open*), and the Godot side shows the pattern its loop repeats. The Context panel's own row is PC-4's.

## Not drafted: the mockup's design, which no draft builds yet

- **N1. Preferences ▸ TOUCH · NEW ▸ Touch & gestures… (tablet and phone).** This is the proposal's Touch & gestures window. It holds Ruling DU Q5's switch, the hold durations and, per `MOBILE_SHELL_PROPOSAL.md`, the UI scale. `MOBILE_SHELL_MS1_SPEC.md` §3.6 specifies UI scale as its own `Preferences ▸ UI scale` submenu on touch builds, and leaves the panel to later work. The read-only half, the gesture reference, is the phone's Help ▸ Gesture reference… (B5).
- **N2. Tablet: Help ▸ Gesture reference….** The gestures screen exists only in `phone_menu.gd` (`_fill_gestures()`); the phone's row is draft 09's.
- **N3. Tablet: Preferences ▸ Open panels in their own window, disabled (DESKTOP ONLY).** PW-5 (*disabled-with-reason on tablet*) is unbuilt. Godot omits the row there today.
- **N4. Phone: File ▸ Show project on disk, which the mockup labels Share project….** *Not in the counts.* The mockup relabels the row on the phone through `liveLabel()` (its `phone:` key), which `mockup_menus.js` does not apply, so the tables show the row as the same. Godot's row keeps its label and does nothing on a touch build: `app.gd::reveal_on_disk()` returns at once when `DccTheme.is_touch()`. No share action exists. Found by reading the mockup's source, 2026-11-01.

## Same after its milestone

- **PC-1. Edit ▸ Previous / Next selection.** A `_todo` row today; live, with Alt+← / Alt+→, from PC-1.
- **PC-4. Story ▸ Political history of the selection; Window ▸ Context panel (the region row) and its Inspect, Pin and Follow rows.** The shipped label or a `_todo` today; the mockup's row from PC-4.
- **PC-5. Story ▸ Note for the selection; Window ▸ Context panel ▸ Note.** As above, from PC-5.
- **PC-6. Story ▸ Ask the assistant about the selection (Ctrl+J); Window ▸ Context panel ▸ Ask.** As above, from PC-6.
- **PC-10. Window ▸ Context panel ▸ MODULES: its ten check rows and Reset panel (Ruling DY, PQ1; not on the phone).** A `_todo` row with its reason today, and still in the "after" column, which models PC-1, PC-4, PC-5 and PC-6 only. Each row goes live when its own module is registered (PC-10).

## The tables

Columns: the mockup's row; Godot's row with PC-1, PC-4, PC-5 and PC-6 built; Godot today, shown only where it differs; the status; the class and rule. `›` marks one level of submenu. Marks: `[kind]`; `· Ctrl+…` the accelerator; `· dark` disabled when built, as Godot builds it; `· SM` disabled in Session mode; `· run-time list` a row repeated at run time (its text is the expression that makes it).

### Desktop

#### File

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | New world… · Ctrl+N · SM | New world… · Ctrl+N · SM |  | same |  |
| 2 | Generate world · Ctrl+R · SM | Generate world · Ctrl+R · SM |  | same |  |
| 3 | Open project… · Ctrl+O | Open project… · Ctrl+O |  | same |  |
| 4 | [submenu] Recent worlds | [submenu] Recent worlds |  | same |  |
| 5 | › VHAREN REACH — 129384 · 5 d ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 6 | › KESSA — 774201 · 3 w ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 7 | › THORNWOOD — 908133 · 2 mo ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 8 | New world from selection… · SM | New world from selection… · SM |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Save project · Ctrl+S | Save project · Ctrl+S |  | same |  |
| 11 | Save as… · Ctrl+Shift+S | Save as… · Ctrl+Shift+S |  | same |  |
| 12 | [check] Autosave | [check] Autosave |  | same |  |
| 13 | [submenu] Autosave interval | [submenu] Autosave interval |  | same |  |
| 14 | › [radio] Off | › [radio] Off |  | same |  |
| 15 | › [radio] Every 1 min | › [radio] Every 1 min |  | same |  |
| 16 | › [radio] Every 5 min | › [radio] Every 5 min |  | same |  |
| 17 | › [radio] Every 15 min | › [radio] Every 15 min |  | same |  |
| 18 | Revert to last save | Revert to last save |  | same |  |
| 19 | Close project · Ctrl+W | Close project · Ctrl+W |  | same |  |
| 20 | ——— | ——— |  | same |  |
| 21 | [submenu] Export | [submenu] Export |  | same |  |
| 22 | › **FOR THE TABLE** | › **FOR THE TABLE** |  | same |  |
| 23 | › Player map… · Ctrl+Shift+E | › Player map… · Ctrl+Shift+E |  | same |  |
| 24 | › DM / full map… | › DM / full map… |  | same |  |
| 25 | › **SHARE** | › **SHARE** |  | same |  |
| 26 | › Export map image… | › Export map image… |  | same |  |
| 27 | › [not built] Print sheet (PDF)… | › [not built] Print sheet (PDF)… |  | same |  |
| 28 | › [not built] VTT battle map (grid + walls)… | › [not built] VTT battle map (grid + walls)… |  | same |  |
| 29 | › **DATA** | › **DATA** |  | same |  |
| 30 | › Maps   tiles | › Maps   tiles |  | same |  |
| 31 | › GIS / GeoJSON   .geojson | › GIS / GeoJSON   .geojson |  | same |  |
| 32 | › World Data   map + atlas | › World Data   map + atlas |  | same |  |
| 33 | › Assets   .zip | › Assets   .zip |  | same |  |
| 34 | **STORAGE LOCATIONS** | **STORAGE LOCATIONS** |  | same |  |
| 35 | [readout] projects | [readout] projects   … |  | same |  |
| 36 | [readout] tile atlas cache | [readout] tile atlas cache   … |  | same |  |
| 37 | [readout] asset packs | [readout] asset packs   … |  | same |  |
| 38 | [readout] exports | [readout] exports   … |  | same |  |
| 39 | Change locations… | Change locations… |  | same |  |
| 40 | Show project on disk | Show project on disk |  | same |  |
| 41 | [signpost] imports live under Records ▸ Import | [signpost] imports live under Records ▸ Import |  | same |  |
| 42 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

#### Edit

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Undo · Ctrl+Z | Undo · Ctrl+Z |  | same |  |
| 2 | Redo · Ctrl+Shift+Z | Redo · Ctrl+Shift+Z |  | same |  |
| 3 | Undo history… | Undo history… |  | same |  |
| 4 | ——— | ——— |  | same |  |
| 5 | Cut · Ctrl+X | Cut · Ctrl+X |  | same |  |
| 6 | Copy · Ctrl+C | Copy · Ctrl+C |  | same |  |
| 7 | Paste · Ctrl+V | Paste · Ctrl+V |  | same |  |
| 8 | Delete · Delete | Delete · Delete |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Select all · Ctrl+A | Select all · Ctrl+A |  | same |  |
| 11 | Deselect · Ctrl+D | Deselect · Ctrl+D |  | same |  |
| 12 | Previous selection · Alt+← | Previous selection · Alt+← |  | same |  |
| 13 | Next selection · Alt+→ | Next selection · Alt+→ |  | same |  |
| 14 | ——— | ——— |  | same |  |
| 15 | Find on map… · Ctrl+F | Find on map… · Ctrl+F |  | same |  |
| 16 | Context card · Shift+F10 | Context card · Shift+F10 |  | same |  |
| 17 | Cartwheel at the pointer · Q | Cartwheel at the pointer · Q |  | same |  |
| 18 | ——— | ——— |  | same |  |
| 19 | Reset generation parameters · SM | Reset generation parameters · SM |  | same |  |
| 20 | [submenu] Reset one stage · SM | [submenu] Reset one stage · SM |  | same |  |
| 21 | › 01 Planet · SM | › 01 Planet |  | same |  |
| 22 | › 02 Extent & scale · SM | › 02 Extent & scale |  | same |  |
| 23 | › 03 World structure · SM | › 03 World structure |  | same |  |
| 24 | › 04 Tectonics · SM | › 04 Tectonics |  | same |  |
| 25 | › 05 Volcanism & impacts · SM | › 05 Volcanism & impacts |  | same |  |
| 26 | › 06 Erosion · SM | › 06 Erosion |  | same |  |
| 27 | › 07 Hydrology · SM | › 07 Hydrology |  | same |  |
| 28 | › 08 Climate · SM | › 08 Climate |  | same |  |
| 29 | › 09 Ecology & biomes · SM | › 09 Ecology & biomes |  | same |  |
| 30 | › 10 Resources & soils · SM | › 10 Resources & soils |  | same |  |

#### Assets

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Asset library · Shift+A | ⧉ Asset library · Shift+A |  | same |  |
| 2 | ⧉ Sprite sheet slicer (▦) | ⧉ Sprite sheet slicer (▦) |  | same |  |
| 3 | Import asset pack .zip… | Import asset pack .zip… |  | same |  |
| 4 | Open pack in library… | Open pack in library… |  | same |  |
| 5 | Pack report… | Pack report… |  | same |  |
| 6 | Import folder of art… | Import folder of art… |  | same |  |
| 7 | Compare pack with library… | Compare pack with library… |  | same |  |
| 8 | ——— | ——— |  | same |  |
| 9 | [submenu] Asset pack | [submenu] Asset pack |  | same |  |
| 10 | › **ACTIVE PACK** | › **ACTIVE PACK** |  | same |  |
| 11 | › [readout] NAME | › [readout] NAME   — loading — |  | same |  |
| 12 | › [readout] AUTHOR | › [readout] AUTHOR   — loading — |  | same |  |
| 13 | › [readout] LICENSE | › [readout] LICENSE   — loading — |  | same |  |
| 14 | › [readout] SCHEMA | › [readout] SCHEMA   — loading — |  | same |  |
| 15 | › [readout] FILLED | › [readout] FILLED   — loading — |  | same |  |
| 16 | › Pack metadata… | › Pack metadata… |  | same |  |
| 17 | › Validate pack | › Validate pack |  | same |  |
| 18 | › Export pack .zip… · Ctrl+Shift+P | › Export pack .zip… · Ctrl+Shift+P |  | same |  |
| 19 | [submenu] Icon families | [submenu] Icon families |  | same |  |
| 20 | › Feature icons   22/31 | › Feature icons |  | same |  |
| 21 | › Settlement pins   8/10 | › Settlement pins |  | same |  |
| 22 | › Settlement traits   5/12 | › Settlement traits |  | same |  |
| 23 | › Points of interest   14/21 | › Points of interest |  | same |  |
| 24 | › Sea marks   4/9 | › Sea marks |  | same |  |
| 25 | › Custom icons   0/6 | › Custom icons |  | same |  |
| 26 | [submenu] Texture sets | [submenu] Texture sets |  | same |  |
| 27 | › Splat channels   4/4 | › Splat channels |  | same |  |
| 28 | › Biome textures   9/12 | › Biome textures |  | same |  |
| 29 | › Terrain textures   6/8 | › Terrain textures |  | same |  |
| 30 | [submenu] Landmark types | Landmark types… |  | differs: kind submenu/command | (c) EH-2 |
| 31 | › [submenu] Physical |  |  | mockup-only | (c) EH-2 |
| 32 | › › Peak |  |  | mockup-only (inside) | (c) EH-2 |
| 33 | › › Ridge |  |  | mockup-only (inside) | (c) EH-2 |
| 34 | › › Saddle   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 35 | › › Cliff |  |  | mockup-only (inside) | (c) EH-2 |
| 36 | › › Gorge |  |  | mockup-only (inside) | (c) EH-2 |
| 37 | › › Cave |  |  | mockup-only (inside) | (c) EH-2 |
| 38 | › › Waterfall |  |  | mockup-only (inside) | (c) EH-2 |
| 39 | › › Spring |  |  | mockup-only (inside) | (c) EH-2 |
| 40 | › › Lake |  |  | mockup-only (inside) | (c) EH-2 |
| 41 | › › Delta   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 42 | › › River confluence |  |  | mockup-only (inside) | (c) EH-2 |
| 43 | › › Volcanic feature |  |  | mockup-only (inside) | (c) EH-2 |
| 44 | › › Rock formation |  |  | mockup-only (inside) | (c) EH-2 |
| 45 | › › Glacial feature   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 46 | › › Ancient forest   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 47 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 48 | › › Open Physical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 49 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 50 | › [submenu] Transport |  |  | mockup-only | (c) EH-2 |
| 51 | › › Mountain pass |  |  | mockup-only (inside) | (c) EH-2 |
| 52 | › › River crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 53 | › › Ford |  |  | mockup-only (inside) | (c) EH-2 |
| 54 | › › Bridge site |  |  | mockup-only (inside) | (c) EH-2 |
| 55 | › › Road junction |  |  | mockup-only (inside) | (c) EH-2 |
| 56 | › › Caravan station |  |  | mockup-only (inside) | (c) EH-2 |
| 57 | › › Portage |  |  | mockup-only (inside) | (c) EH-2 |
| 58 | › › Harbour |  |  | mockup-only (inside) | (c) EH-2 |
| 59 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 60 | › › Open Transport in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 61 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 62 | › [submenu] Economic |  |  | mockup-only | (c) EH-2 |
| 63 | › › Mine |  |  | mockup-only (inside) | (c) EH-2 |
| 64 | › › Quarry |  |  | mockup-only (inside) | (c) EH-2 |
| 65 | › › Salt works   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 66 | › › Resource extraction site |  |  | mockup-only (inside) | (c) EH-2 |
| 67 | › › Market site |  |  | mockup-only (inside) | (c) EH-2 |
| 68 | › › Trade depot |  |  | mockup-only (inside) | (c) EH-2 |
| 69 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 70 | › › Open Economic in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 71 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 72 | › [submenu] Military |  |  | mockup-only | (c) EH-2 |
| 73 | › › Fort |  |  | mockup-only (inside) | (c) EH-2 |
| 74 | › › Watchtower |  |  | mockup-only (inside) | (c) EH-2 |
| 75 | › › Fortified pass |  |  | mockup-only (inside) | (c) EH-2 |
| 76 | › › Fortified crossing |  |  | mockup-only (inside) | (c) EH-2 |
| 77 | › › Battlefield |  |  | mockup-only (inside) | (c) EH-2 |
| 78 | › › Border marker |  |  | mockup-only (inside) | (c) EH-2 |
| 79 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 80 | › › Open Military in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 81 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 82 | › [submenu] Cultural |  |  | mockup-only | (c) EH-2 |
| 83 | › › Shrine   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 84 | › › Temple   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 85 | › › Sacred grove   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 86 | › › Sacred mountain   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 87 | › › Pilgrimage site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 88 | › › Tomb   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 89 | › › Monument   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 90 | › › Ceremonial site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 91 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 92 | › › Open Cultural in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 93 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 94 | › [submenu] Historical |  |  | mockup-only | (c) EH-2 |
| 95 | › › Ruin   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 96 | › › Abandoned settlement |  |  | mockup-only (inside) | (c) EH-2 |
| 97 | › › Ancient road |  |  | mockup-only (inside) | (c) EH-2 |
| 98 | › › Historic battlefield   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 99 | › › Destroyed fortress   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 100 | › › Historic crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 101 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 102 | › › Open Historical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 103 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 104 | › Placed by hand   3 |  |  | mockup-only | (c) EH-2 |
| 105 | › ——— |  |  | mockup-only | (c) EH-2 |
| 106 | › Landmark icons… |  |  | mockup-only | (c) EH-2 |
| 107 | › Landmark label style… |  |  | mockup-only | (c) EH-2 |
| 108 | ——— | ——— |  | same |  |
| 109 | Apply library to map | Apply library to map |  | same |  |
| 110 | Clear library… | Clear library…   destructive |  | same |  |

#### Records

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 2 | **RECORDS** | **RECORDS** |  | same |  |
| 3 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 4 | Places… | Places… |  | same |  |
| 5 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 6 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 7 | Places without notes… | Places without notes… |  | same |  |
| 8 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 9 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 10 | World data tables… | World data tables… |  | same |  |
| 11 | [submenu] Import | [submenu] Import |  | same |  |
| 12 | › Maps   tiles | › Maps   tiles |  | same |  |
| 13 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 14 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 15 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 16 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 17 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 18 | › **SOURCES** | › **SOURCES** |  | same |  |
| 19 | › External Sources | › External Sources |  | same |  |
| 20 | › Connected Sources | › Connected Sources |  | same |  |
| 21 | › Source Registry | › Source Registry |  | same |  |
| 22 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 23 | › Definitions | › Definitions |  | same |  |
| 24 | › Check Data | › Check Data |  | same |  |
| 25 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 26 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

#### Story

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **TIME** | **TIME** |  | same |  |
| 2 | Timeline · Ctrl+Shift+T | Timeline · Ctrl+Shift+T |  | same |  |
| 3 | Political history of the selection | Political history of the selection | Political history of this place… | same after the milestone; today: label | m PC-4 |
| 4 | Faction history… | Faction history… |  | same |  |
| 5 | **JOURNEYS** | **JOURNEYS** |  | same |  |
| 6 | Journey planner | Journey planner |  | same |  |
| 7 | Saved journeys… | Saved journeys… |  | same |  |
| 8 | Travel library… · Shift+L | Travel library… · Shift+L |  | same |  |
| 9 | **CONFLICTS & CAMPAIGNS** | **CONFLICTS & CAMPAIGNS** |  | same |  |
| 10 | Conflicts… | Conflicts… |  | same |  |
| 11 | Draw a conflict | Draw a conflict |  | same |  |
| 12 | [readout] Campaigns this year | [readout] Campaigns this year |  | same |  |
| 13 | **NOTES** | **NOTES** |  | same |  |
| 14 | Note for the selection · Ctrl+Shift+N | Note for the selection · Ctrl+Shift+N | Notes for this place · Ctrl+Shift+N | same after the milestone; today: label | m PC-5 |
| 15 | [signpost] Places without notes… is in Records | [signpost] Places without notes is in Records |  | same |  |
| 16 | Ask the assistant about the selection · Ctrl+J | Ask the assistant about the selection · Ctrl+J | Assistant panel… | same after the milestone; today: label, key Ctrl+J/none | m PC-6 |

#### Preferences

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **PERFORMANCE** | **PERFORMANCE** |  | same |  |
| 2 | [check] GPU acceleration | [check] GPU acceleration |  | same |  |
| 3 | [submenu] Devices | [submenu] Devices |  | same |  |
| 4 | › [check] Automatic (highest-performance GPU) | › [check] Automatic (highest-performance GPU) |  | same |  |
| 5 | › ——— | › ——— |  | same |  |
| 6 | › [check] Adreno 630 · integrated · Vulkan | › [check] … · … · … · run-time list |  | same (run-time list) |  |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › [readout] Memory | › [readout] Memory: not measured yet |  | same |  |
| 9 | › ——— | › ——— |  | same |  |
| 10 | › Rescan devices… | › Rescan devices… |  | same |  |
| 11 | [submenu] Multi-GPU mode | [submenu] Multi-GPU mode |  | same |  |
| 12 | › [radio] Single device | › [radio] Single device |  | same |  |
| 13 | › [radio] Split tiles | › [radio] Split tiles |  | same |  |
| 14 | › [not built] Alternate frames | › [radio] Alternate frames · dark |  | same |  |
| 15 | [submenu] CPU worker threads | [submenu] CPU worker threads |  | same |  |
| 16 | › [radio] Automatic | › [radio] Automatic |  | same |  |
| 17 | › [radio] 1 — 1 thread | › [radio] 1 — 1 thread |  | same |  |
| 18 | › [radio] 2 — quarter | › [radio] 2 — quarter |  | same |  |
| 19 | › [radio] 4 — half | › [radio] 4 — half |  | same |  |
| 20 | › [radio] 8 — all cores | › [radio] 8 — all cores |  | same |  |
| 21 | › [readout] Running | › [readout] Running … workers now |  | same |  |
| 22 | [submenu] VRAM budget | [submenu] VRAM budget |  | same |  |
| 23 | › [radio] No cap | › [radio] No cap |  | same |  |
| 24 | › [radio] 1 GB | › [radio] 1 GB |  | same |  |
| 25 | › [radio] 2 GB | › [radio] 2 GB |  | same |  |
| 26 | › [radio] 4 GB | › [radio] 4 GB |  | same |  |
| 27 | › [radio] 8 GB | › [radio] 8 GB |  | same |  |
| 28 | › [radio] 12 GB | › [radio] 12 GB |  | same |  |
| 29 | › [radio] 16 GB | › [radio] 16 GB |  | same |  |
| 30 | › [radio] 24 GB | › [radio] 24 GB |  | same |  |
| 31 | › [readout] Current grid 2048×2048 needs about | › [readout] Current grid …x… needs about … MB |  | same |  |
| 32 | [submenu] Fallback when VRAM full | [submenu] Fallback when VRAM full |  | same |  |
| 33 | › [radio] CPU tile pass | › [radio] CPU tile pass |  | same |  |
| 34 | › [not built] Reduce working res | › [radio] Reduce working res · dark |  | same |  |
| 35 | › [radio] Fail with error | › [radio] Fail with error |  | same |  |
| 36 | Try the GPU again · dark | Try the GPU again · dark |  | same |  |
| 37 | **GRAPHICS** | **GRAPHICS** |  | same |  |
| 38 | [submenu] Render quality | [submenu] Render quality |  | same |  |
| 39 | › [radio] performance | › [radio] performance |  | same |  |
| 40 | › [radio] balanced | › [radio] balanced |  | same |  |
| 41 | › [radio] quality | › [radio] quality |  | same |  |
| 42 | › [radio] ultra | › [radio] ultra |  | same |  |
| 43 | [not built] Anti-aliasing · anisotropy | [not built] Anti-aliasing · anisotropy |  | differs: reason | s S1 |
| 44 | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management |  | same |  |
| 45 | [not built] 3D viewport defaults | [not built] 3D viewport defaults |  | differs: reason | s S1 |
| 46 | [submenu] Lighting rig defaults | [submenu] Lighting rig defaults |  | same |  |
| 47 | › [readout] Following the engine | › [readout] Following the engine |  | same |  |
| 48 | › [submenu] Azimuth | › **AZIMUTH** |  | differs: kind submenu/caption | ?  |
| 49 | › › [radio] 0 deg |  |  | mockup-only | ?  |
| 50 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 51 | › › [radio] 90 deg |  |  | mockup-only | ?  |
| 52 | › › [radio] 135 deg |  |  | mockup-only | ?  |
| 53 | › › [radio] 180 deg |  |  | mockup-only | ?  |
| 54 | › › [radio] 225 deg |  |  | mockup-only | ?  |
| 55 | › › [radio] 270 deg |  |  | mockup-only | ?  |
| 56 | › › [radio] 315 deg |  |  | mockup-only | ?  |
| 57 |  | › [radio] 0 deg |  | Godot-only | ?  |
| 58 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 59 |  | › [radio] 90 deg |  | Godot-only | ?  |
| 60 |  | › [radio] 135 deg |  | Godot-only | ?  |
| 61 |  | › [radio] 180 deg |  | Godot-only | ?  |
| 62 |  | › [radio] 225 deg |  | Godot-only | ?  |
| 63 |  | › [radio] 270 deg |  | Godot-only | ?  |
| 64 |  | › [radio] 315 deg |  | Godot-only | ?  |
| 65 | › [submenu] Elevation | › **ELEVATION** |  | differs: kind submenu/caption | ?  |
| 66 | › › [radio] 15 deg |  |  | mockup-only | ?  |
| 67 | › › [radio] 25 deg |  |  | mockup-only | ?  |
| 68 | › › [radio] 35 deg |  |  | mockup-only | ?  |
| 69 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 70 | › › [radio] 55 deg |  |  | mockup-only | ?  |
| 71 | › › [radio] 65 deg |  |  | mockup-only | ?  |
| 72 | › › [radio] 75 deg |  |  | mockup-only | ?  |
| 73 |  | › [radio] 15 deg |  | Godot-only | ?  |
| 74 |  | › [radio] 25 deg |  | Godot-only | ?  |
| 75 |  | › [radio] 35 deg |  | Godot-only | ?  |
| 76 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 77 |  | › [radio] 55 deg |  | Godot-only | ?  |
| 78 |  | › [radio] 65 deg |  | Godot-only | ?  |
| 79 |  | › [radio] 75 deg |  | Godot-only | ?  |
| 80 | › [submenu] Ambient | › **AMBIENT** |  | differs: kind submenu/caption | ?  |
| 81 | › › [radio] 0.00 |  |  | mockup-only | ?  |
| 82 | › › [radio] 0.15 |  |  | mockup-only | ?  |
| 83 | › › [radio] 0.25 |  |  | mockup-only | ?  |
| 84 | › › [radio] 0.35 |  |  | mockup-only | ?  |
| 85 | › › [radio] 0.50 |  |  | mockup-only | ?  |
| 86 | › › [radio] 0.70 |  |  | mockup-only | ?  |
| 87 |  | › [radio] 0.00 |  | Godot-only | ?  |
| 88 |  | › [radio] 0.15 |  | Godot-only | ?  |
| 89 |  | › [radio] 0.25 |  | Godot-only | ?  |
| 90 |  | › [radio] 0.35 |  | Godot-only | ?  |
| 91 |  | › [radio] 0.50 |  | Godot-only | ?  |
| 92 |  | › [radio] 0.70 |  | Godot-only | ?  |
| 93 | › [submenu] Multidirectional | › **MULTIDIRECTIONAL** |  | differs: kind submenu/caption | ?  |
| 94 | › › [radio] 1 light (single sun) |  |  | mockup-only | ?  |
| 95 | › › [radio] 2 lights |  |  | mockup-only | ?  |
| 96 | › › [radio] 4 lights |  |  | mockup-only | ?  |
| 97 | › › [radio] 6 lights |  |  | mockup-only | ?  |
| 98 | › › [radio] 8 lights |  |  | mockup-only | ?  |
| 99 | › › [radio] 12 lights |  |  | mockup-only | ?  |
| 100 |  | › [radio] 1 light (single sun) |  | Godot-only | ?  |
| 101 |  | › [radio] 2 lights |  | Godot-only | ?  |
| 102 |  | › [radio] 4 lights |  | Godot-only | ?  |
| 103 |  | › [radio] 6 lights |  | Godot-only | ?  |
| 104 |  | › [radio] 8 lights |  | Godot-only | ?  |
| 105 |  | › [radio] 12 lights |  | Godot-only | ?  |
| 106 | › ——— | › ——— |  | same |  |
| 107 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 108 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 109 | › [radio] 1x | › [radio] 1x |  | same |  |
| 110 | › [radio] 2x | › [radio] 2x |  | same |  |
| 111 | › [radio] 4x | › [radio] 4x |  | same |  |
| 112 | › ——— | › ——— |  | same |  |
| 113 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 114 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 115 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 116 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 117 | › [radio] Manual | › [radio] Manual |  | same |  |
| 118 | › ——— | › ——— |  | same |  |
| 119 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 120 | › Leave deep detail | › Leave deep detail |  | same |  |
| 121 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 122 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 123 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 124 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 125 | › ——— |  |  | mockup-only | ?  |
| 126 | › [submenu] LOD levels | › **LOD LEVELS** |  | differs: kind submenu/caption | ?  |
| 127 | › › [radio] LOD 0–0   1 tile |  |  | mockup-only | ?  |
| 128 | › › [radio] LOD 0–1   5 tiles |  |  | mockup-only | ?  |
| 129 | › › [radio] LOD 0–2   21 tiles |  |  | mockup-only | ?  |
| 130 | › › [radio] LOD 0–3   85 tiles |  |  | mockup-only | ?  |
| 131 | › › [radio] LOD 0–4   341 tiles |  |  | mockup-only | ?  |
| 132 | › › [radio] LOD 0–5   1 365 tiles |  |  | mockup-only | ?  |
| 133 | › › [radio] LOD 0–6   5 461 tiles |  |  | mockup-only | ?  |
| 134 | › › [radio] LOD 0–7   21 845 tiles |  |  | mockup-only | ?  |
| 135 | › › [radio] LOD 0–8   87 381 tiles |  |  | mockup-only | ?  |
| 136 |  | › [radio] LOD 0–0   1 tile |  | Godot-only | ?  |
| 137 |  | › [radio] LOD 0–1   5 tiles |  | Godot-only | ?  |
| 138 |  | › [radio] LOD 0–2   21 tiles |  | Godot-only | ?  |
| 139 |  | › [radio] LOD 0–3   85 tiles |  | Godot-only | ?  |
| 140 |  | › [radio] LOD 0–4   341 tiles |  | Godot-only | ?  |
| 141 |  | › [radio] LOD 0–5   1 365 tiles |  | Godot-only | ?  |
| 142 |  | › [radio] LOD 0–6   5 461 tiles |  | Godot-only | ?  |
| 143 |  | › [radio] LOD 0–7   21 845 tiles |  | Godot-only | ?  |
| 144 |  | › [radio] LOD 0–8   87 381 tiles |  | Godot-only | ?  |
| 145 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 146 | › [readout] 85 chunks · 1.2 GB · to LOD 3 |  |  | mockup-only | ?  |
| 147 |  | › [readout] … chunks · … · empty |  | Godot-only | ?  |
| 148 | › [submenu] Size cap · GB | › **SIZE CAP · GB** |  | differs: kind submenu/caption | ?  |
| 149 | › › [radio] No cap |  |  | mockup-only | ?  |
| 150 | › › [radio] 0.5 GB |  |  | mockup-only | ?  |
| 151 | › › [radio] 1 GB |  |  | mockup-only | ?  |
| 152 | › › [radio] 2 GB |  |  | mockup-only | ?  |
| 153 | › › [radio] 4 GB |  |  | mockup-only | ?  |
| 154 | › › [radio] 8 GB |  |  | mockup-only | ?  |
| 155 | › › [radio] 16 GB |  |  | mockup-only | ?  |
| 156 |  | › [radio] No cap |  | Godot-only | ?  |
| 157 |  | › [radio] 0.5 GB |  | Godot-only | ?  |
| 158 |  | › [radio] 1 GB |  | Godot-only | ?  |
| 159 |  | › [radio] 2 GB |  | Godot-only | ?  |
| 160 |  | › [radio] 4 GB |  | Godot-only | ?  |
| 161 |  | › [radio] 8 GB |  | Godot-only | ?  |
| 162 |  | › [radio] 16 GB |  | Godot-only | ?  |
| 163 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 164 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 165 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 166 | › ——— | › ——— |  | same |  |
| 167 | › Export atlas… | › Export atlas… |  | same |  |
| 168 | › Import atlas… | › Import atlas… |  | same |  |
| 169 | › ——— | › ——— |  | same |  |
| 170 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 171 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 172 | › [check] Grid | › [check] Grid |  | same |  |
| 173 | › [check] Colors | › [check] Colors |  | same |  |
| 174 | › [check] Labels | › [check] Labels |  | same |  |
| 175 | › ——— | › ——— |  | same |  |
| 176 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 177 | **MEMORY** | **MEMORY** |  | same |  |
| 178 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 179 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 180 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 181 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 182 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 183 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 184 | › ——— | › ——— |  | same |  |
| 185 | › Clear undo history now | › Clear undo history now |  | same |  |
| 186 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 187 | Clear caches… | Clear caches… |  | same |  |
| 188 | **APPLICATION** | **APPLICATION** |  | same |  |
| 189 | Storage locations… | Storage locations… |  | same |  |
| 190 | Assistant… | Assistant… |  | same |  |
| 191 | Quick bar… | Quick bar… |  | same |  |
| 192 | [check] Open panels in their own window | [check] Open panels in their own window |  | same |  |
| 193 | [submenu] Theme | [submenu] Theme |  | same |  |
| 194 | › [radio] Dark | › [radio] Dark |  | same |  |
| 195 | › [radio] Light | › [radio] Light |  | same |  |
| 196 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 197 | [submenu] Units | [submenu] Units |  | same |  |
| 198 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 199 | › [radio] Miles | › [radio] Miles |  | same |  |
| 200 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 201 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 202 | › [radio] Left | › [radio] Left |  | same |  |
| 203 | › [radio] Right | › [radio] Right |  | same |  |
| 204 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |

#### Window

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [check] Left dock | [check] Left dock |  | same |  |
| 2 | [check] Context panel | [check] Context panel | [check] Right dock | same after the milestone; today: label | m PC-4 |
| 3 | [submenu] Context panel | [submenu] Context panel |  | same |  |
| 4 | › Inspect · Ctrl+1 | › Inspect · Ctrl+1 | › [not built] Inspect | same after the milestone; today: kind command/disabled (not built), key Ctrl+1/none | m PC-4 |
| 5 | › Note · Ctrl+2 | › Note · Ctrl+2 | › [not built] Note | same after the milestone; today: kind command/disabled (not built), key Ctrl+2/none | m PC-5 |
| 6 | › Ask (assistant) · Ctrl+3 | › Ask (assistant) · Ctrl+3 | › [not built] Ask (assistant) | same after the milestone; today: kind command/disabled (not built), key Ctrl+3/none | m PC-6 |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › Pin to the current subject | › Pin to the current subject | › [not built] Pin to the current subject | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 9 | › Follow the selection | › Follow the selection | › [not built] Follow the selection | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 10 | › **MODULES** | › **MODULES** |  | same |  |
| 11 | › [check] History | › [not built] History |  | differs: kind check/disabled (not built) | m PC-10 |
| 12 | › [check] Economy | › [not built] Economy |  | differs: kind check/disabled (not built) | m PC-10 |
| 13 | › [check] Military | › [not built] Military |  | differs: kind check/disabled (not built) | m PC-10 |
| 14 | › [check] Faith, culture & relations | › [not built] Faith, culture & relations |  | differs: kind check/disabled (not built) | m PC-10 |
| 15 | › [check] Territory | › [not built] Territory |  | differs: kind check/disabled (not built) | m PC-10 |
| 16 | › [check] City layout | › [not built] City layout |  | differs: kind check/disabled (not built) | m PC-10 |
| 17 | › [check] Here | › [not built] Here |  | differs: kind check/disabled (not built) | m PC-10 |
| 18 | › [check] Routes & journeys | › [not built] Routes & journeys |  | differs: kind check/disabled (not built) | m PC-10 |
| 19 | › [check] Links | › [not built] Links |  | differs: kind check/disabled (not built) | m PC-10 |
| 20 | › [check] Recent & bookmarks | › [not built] Recent & bookmarks |  | differs: kind check/disabled (not built) | m PC-10 |
| 21 | › ——— | › ——— |  | same |  |
| 22 | › Reset panel | › Reset panel |  | same |  |
| 23 | [check] Timeline | [check] Timeline |  | same |  |
| 24 | [check] Status bar | [check] Status bar |  | same |  |
| 25 | [check] Domain rail | [check] Domain rail |  | same |  |
| 26 | ——— | ——— |  | same |  |
| 27 | [check] Session mode · Ctrl+Shift+M | [check] Session mode · Ctrl+Shift+M |  | same |  |
| 28 | [check] Diagnostics overlay · Shift+D | [check] Diagnostics overlay · Shift+D |  | same |  |
| 29 | [check] 3D view | [check] 3D view |  | same |  |
| 30 | [submenu] 3D capability | [submenu] 3D capability |  | same |  |
| 31 | › [readout] 3D tier | › [readout] 3D tier: not tested |  | same |  |
| 32 | › Re-test 3D | › Re-test 3D |  | same |  |
| 33 | › [submenu] Force tier |  |  | mockup-only | ?  |
| 34 | › › [radio] Automatic (as measured) |  |  | mockup-only (inside) | ?  |
| 35 | › › [radio] Off |  |  | mockup-only (inside) | ?  |
| 36 | › › [radio] Basic |  |  | mockup-only (inside) | ?  |
| 37 | › › [radio] Standard |  |  | mockup-only (inside) | ?  |
| 38 | › › [radio] Full |  |  | mockup-only (inside) | ?  |
| 39 |  | › **FORCE TIER** |  | Godot-only | ?  |
| 40 |  | › [radio] Automatic (as measured) |  | Godot-only | ?  |
| 41 |  | › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | Godot-only | ?  |
| 42 | ——— | ——— |  | same |  |
| 43 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 44 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 45 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 46 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 47 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 48 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 49 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 50 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 51 | ——— | ——— |  | same |  |
| 52 | Reset layout | Reset layout |  | same |  |
| 53 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 54 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 55 | › Generate | › Generate |  | same |  |
| 56 | › Sculpt | › Sculpt |  | same |  |
| 57 | › Cartography | › Cartography |  | same |  |
| 58 | › Journey | › Journey |  | same |  |
| 59 | › ——— | › ——— |  | same |  |
| 60 | › Save layout as… | › Save layout as… |  | same |  |
| 61 | › Forget layout… | › Forget layout… |  | same |  |

#### Help

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [not built] Documentation | [not built] Documentation |  | differs: reason | s S1 |
| 2 | Keyboard reference… | Keyboard reference… |  | same |  |
| 3 | Credits & academic principles | Credits & academic principles |  | same |  |
| 4 | Generation info… | Generation info… |  | same |  |
| 5 | Save diagnostic report… | Save diagnostic report… |  | same |  |
| 6 | About | About |  | same |  |

### Tablet, 1 180 dp and wider

#### File

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | New world… · Ctrl+N · SM | New world… · Ctrl+N · SM |  | same |  |
| 2 | Generate world · Ctrl+R · SM | Generate world · Ctrl+R · SM |  | same |  |
| 3 | Open project… · Ctrl+O | Open project… · Ctrl+O |  | same |  |
| 4 | [submenu] Recent worlds | [submenu] Recent worlds |  | same |  |
| 5 | › VHAREN REACH — 129384 · 5 d ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 6 | › KESSA — 774201 · 3 w ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 7 | › THORNWOOD — 908133 · 2 mo ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 8 | New world from selection… · SM | New world from selection… · SM |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Save project · Ctrl+S | Save project · Ctrl+S |  | same |  |
| 11 | Save as… · Ctrl+Shift+S | Save as… · Ctrl+Shift+S |  | same |  |
| 12 | [check] Autosave | [check] Autosave |  | same |  |
| 13 | [submenu] Autosave interval | [submenu] Autosave interval |  | same |  |
| 14 | › [radio] Off | › [radio] Off |  | same |  |
| 15 | › [radio] Every 1 min | › [radio] Every 1 min |  | same |  |
| 16 | › [radio] Every 5 min | › [radio] Every 5 min |  | same |  |
| 17 | › [radio] Every 15 min | › [radio] Every 15 min |  | same |  |
| 18 | Revert to last save | Revert to last save |  | same |  |
| 19 | Close project · Ctrl+W | Close project · Ctrl+W |  | same |  |
| 20 | ——— | ——— |  | same |  |
| 21 | [submenu] Export | [submenu] Export |  | same |  |
| 22 | › **FOR THE TABLE** | › **FOR THE TABLE** |  | same |  |
| 23 | › Player map… · Ctrl+Shift+E | › Player map… · Ctrl+Shift+E |  | same |  |
| 24 | › DM / full map… | › DM / full map… |  | same |  |
| 25 | › **SHARE** | › **SHARE** |  | same |  |
| 26 | › Export map image… | › Export map image… |  | same |  |
| 27 | › [not built] Print sheet (PDF)… | › [not built] Print sheet (PDF)… |  | same |  |
| 28 | › [not built] VTT battle map (grid + walls)… | › [not built] VTT battle map (grid + walls)… |  | same |  |
| 29 | › **DATA** | › **DATA** |  | same |  |
| 30 | › Maps   tiles | › Maps   tiles |  | same |  |
| 31 | › GIS / GeoJSON   .geojson | › GIS / GeoJSON   .geojson |  | same |  |
| 32 | › World Data   map + atlas | › World Data   map + atlas |  | same |  |
| 33 | › Assets   .zip | › Assets   .zip |  | same |  |
| 34 | **STORAGE LOCATIONS** | **STORAGE LOCATIONS** |  | same |  |
| 35 | [readout] projects | [readout] projects   … |  | same |  |
| 36 | [readout] tile atlas cache | [readout] tile atlas cache   … |  | same |  |
| 37 | [readout] asset packs | [readout] asset packs   … |  | same |  |
| 38 | [readout] exports | [readout] exports   … |  | same |  |
| 39 | Change locations… | Change locations… |  | same |  |
| 40 | Show project on disk | Show project on disk |  | same |  |
| 41 | [signpost] imports live under Records ▸ Import | [signpost] imports live under Records ▸ Import |  | same |  |
| 42 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

#### Edit

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Undo · Ctrl+Z | Undo · Ctrl+Z |  | same |  |
| 2 | Redo · Ctrl+Shift+Z | Redo · Ctrl+Shift+Z |  | same |  |
| 3 | Undo history… | Undo history… |  | same |  |
| 4 | ——— | ——— |  | same |  |
| 5 | Cut · Ctrl+X | Cut · Ctrl+X |  | same |  |
| 6 | Copy · Ctrl+C | Copy · Ctrl+C |  | same |  |
| 7 | Paste · Ctrl+V | Paste · Ctrl+V |  | same |  |
| 8 | Delete · Delete | Delete · Delete |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Select all · Ctrl+A | Select all · Ctrl+A |  | same |  |
| 11 | Deselect · Ctrl+D | Deselect · Ctrl+D |  | same |  |
| 12 | Previous selection · Alt+← | Previous selection · Alt+← |  | same |  |
| 13 | Next selection · Alt+→ | Next selection · Alt+→ |  | same |  |
| 14 | ——— | ——— |  | same |  |
| 15 | Find on map… · Ctrl+F | Find on map… · Ctrl+F |  | same |  |
| 16 | Context card · Shift+F10 | Context card · Shift+F10 |  | same |  |
| 17 | Cartwheel at the pointer · Q | Cartwheel at the pointer · Q |  | same |  |
| 18 | ——— | ——— |  | same |  |
| 19 | Reset generation parameters · SM | Reset generation parameters · SM |  | same |  |
| 20 | [submenu] Reset one stage · SM | [submenu] Reset one stage · SM |  | same |  |
| 21 | › 01 Planet · SM | › 01 Planet |  | same |  |
| 22 | › 02 Extent & scale · SM | › 02 Extent & scale |  | same |  |
| 23 | › 03 World structure · SM | › 03 World structure |  | same |  |
| 24 | › 04 Tectonics · SM | › 04 Tectonics |  | same |  |
| 25 | › 05 Volcanism & impacts · SM | › 05 Volcanism & impacts |  | same |  |
| 26 | › 06 Erosion · SM | › 06 Erosion |  | same |  |
| 27 | › 07 Hydrology · SM | › 07 Hydrology |  | same |  |
| 28 | › 08 Climate · SM | › 08 Climate |  | same |  |
| 29 | › 09 Ecology & biomes · SM | › 09 Ecology & biomes |  | same |  |
| 30 | › 10 Resources & soils · SM | › 10 Resources & soils |  | same |  |

#### Assets

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Asset library · Shift+A | ⧉ Asset library · Shift+A |  | same |  |
| 2 | ⧉ Sprite sheet slicer (▦) | ⧉ Sprite sheet slicer (▦) |  | same |  |
| 3 | Import asset pack .zip… | Import asset pack .zip… |  | same |  |
| 4 | Open pack in library… | Open pack in library… |  | same |  |
| 5 | Pack report… | Pack report… |  | same |  |
| 6 | Import folder of art… | Import folder of art… |  | same |  |
| 7 | Compare pack with library… | Compare pack with library… |  | same |  |
| 8 | ——— | ——— |  | same |  |
| 9 | [submenu] Asset pack | [submenu] Asset pack |  | same |  |
| 10 | › **ACTIVE PACK** | › **ACTIVE PACK** |  | same |  |
| 11 | › [readout] NAME | › [readout] NAME   — loading — |  | same |  |
| 12 | › [readout] AUTHOR | › [readout] AUTHOR   — loading — |  | same |  |
| 13 | › [readout] LICENSE | › [readout] LICENSE   — loading — |  | same |  |
| 14 | › [readout] SCHEMA | › [readout] SCHEMA   — loading — |  | same |  |
| 15 | › [readout] FILLED | › [readout] FILLED   — loading — |  | same |  |
| 16 | › Pack metadata… | › Pack metadata… |  | same |  |
| 17 | › Validate pack | › Validate pack |  | same |  |
| 18 | › Export pack .zip… · Ctrl+Shift+P | › Export pack .zip… · Ctrl+Shift+P |  | same |  |
| 19 | [submenu] Icon families | [submenu] Icon families |  | same |  |
| 20 | › Feature icons   22/31 | › Feature icons |  | same |  |
| 21 | › Settlement pins   8/10 | › Settlement pins |  | same |  |
| 22 | › Settlement traits   5/12 | › Settlement traits |  | same |  |
| 23 | › Points of interest   14/21 | › Points of interest |  | same |  |
| 24 | › Sea marks   4/9 | › Sea marks |  | same |  |
| 25 | › Custom icons   0/6 | › Custom icons |  | same |  |
| 26 | [submenu] Texture sets | [submenu] Texture sets |  | same |  |
| 27 | › Splat channels   4/4 | › Splat channels |  | same |  |
| 28 | › Biome textures   9/12 | › Biome textures |  | same |  |
| 29 | › Terrain textures   6/8 | › Terrain textures |  | same |  |
| 30 | [submenu] Landmark types | Landmark types… |  | differs: kind submenu/command | (c) EH-2 |
| 31 | › [submenu] Physical |  |  | mockup-only | (c) EH-2 |
| 32 | › › Peak |  |  | mockup-only (inside) | (c) EH-2 |
| 33 | › › Ridge |  |  | mockup-only (inside) | (c) EH-2 |
| 34 | › › Saddle   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 35 | › › Cliff |  |  | mockup-only (inside) | (c) EH-2 |
| 36 | › › Gorge |  |  | mockup-only (inside) | (c) EH-2 |
| 37 | › › Cave |  |  | mockup-only (inside) | (c) EH-2 |
| 38 | › › Waterfall |  |  | mockup-only (inside) | (c) EH-2 |
| 39 | › › Spring |  |  | mockup-only (inside) | (c) EH-2 |
| 40 | › › Lake |  |  | mockup-only (inside) | (c) EH-2 |
| 41 | › › Delta   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 42 | › › River confluence |  |  | mockup-only (inside) | (c) EH-2 |
| 43 | › › Volcanic feature |  |  | mockup-only (inside) | (c) EH-2 |
| 44 | › › Rock formation |  |  | mockup-only (inside) | (c) EH-2 |
| 45 | › › Glacial feature   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 46 | › › Ancient forest   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 47 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 48 | › › Open Physical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 49 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 50 | › [submenu] Transport |  |  | mockup-only | (c) EH-2 |
| 51 | › › Mountain pass |  |  | mockup-only (inside) | (c) EH-2 |
| 52 | › › River crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 53 | › › Ford |  |  | mockup-only (inside) | (c) EH-2 |
| 54 | › › Bridge site |  |  | mockup-only (inside) | (c) EH-2 |
| 55 | › › Road junction |  |  | mockup-only (inside) | (c) EH-2 |
| 56 | › › Caravan station |  |  | mockup-only (inside) | (c) EH-2 |
| 57 | › › Portage |  |  | mockup-only (inside) | (c) EH-2 |
| 58 | › › Harbour |  |  | mockup-only (inside) | (c) EH-2 |
| 59 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 60 | › › Open Transport in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 61 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 62 | › [submenu] Economic |  |  | mockup-only | (c) EH-2 |
| 63 | › › Mine |  |  | mockup-only (inside) | (c) EH-2 |
| 64 | › › Quarry |  |  | mockup-only (inside) | (c) EH-2 |
| 65 | › › Salt works   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 66 | › › Resource extraction site |  |  | mockup-only (inside) | (c) EH-2 |
| 67 | › › Market site |  |  | mockup-only (inside) | (c) EH-2 |
| 68 | › › Trade depot |  |  | mockup-only (inside) | (c) EH-2 |
| 69 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 70 | › › Open Economic in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 71 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 72 | › [submenu] Military |  |  | mockup-only | (c) EH-2 |
| 73 | › › Fort |  |  | mockup-only (inside) | (c) EH-2 |
| 74 | › › Watchtower |  |  | mockup-only (inside) | (c) EH-2 |
| 75 | › › Fortified pass |  |  | mockup-only (inside) | (c) EH-2 |
| 76 | › › Fortified crossing |  |  | mockup-only (inside) | (c) EH-2 |
| 77 | › › Battlefield |  |  | mockup-only (inside) | (c) EH-2 |
| 78 | › › Border marker |  |  | mockup-only (inside) | (c) EH-2 |
| 79 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 80 | › › Open Military in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 81 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 82 | › [submenu] Cultural |  |  | mockup-only | (c) EH-2 |
| 83 | › › Shrine   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 84 | › › Temple   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 85 | › › Sacred grove   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 86 | › › Sacred mountain   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 87 | › › Pilgrimage site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 88 | › › Tomb   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 89 | › › Monument   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 90 | › › Ceremonial site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 91 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 92 | › › Open Cultural in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 93 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 94 | › [submenu] Historical |  |  | mockup-only | (c) EH-2 |
| 95 | › › Ruin   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 96 | › › Abandoned settlement |  |  | mockup-only (inside) | (c) EH-2 |
| 97 | › › Ancient road |  |  | mockup-only (inside) | (c) EH-2 |
| 98 | › › Historic battlefield   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 99 | › › Destroyed fortress   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 100 | › › Historic crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 101 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 102 | › › Open Historical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 103 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 104 | › Placed by hand   3 |  |  | mockup-only | (c) EH-2 |
| 105 | › ——— |  |  | mockup-only | (c) EH-2 |
| 106 | › Landmark icons… |  |  | mockup-only | (c) EH-2 |
| 107 | › Landmark label style… |  |  | mockup-only | (c) EH-2 |
| 108 | ——— | ——— |  | same |  |
| 109 | Apply library to map | Apply library to map |  | same |  |
| 110 | Clear library… | Clear library…   destructive |  | same |  |

#### Records

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 2 | **RECORDS** | **RECORDS** |  | same |  |
| 3 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 4 | Places… | Places… |  | same |  |
| 5 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 6 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 7 | Places without notes… | Places without notes… |  | same |  |
| 8 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 9 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 10 | World data tables… | World data tables… |  | same |  |
| 11 | [submenu] Import | [submenu] Import |  | same |  |
| 12 | › Maps   tiles | › Maps   tiles |  | same |  |
| 13 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 14 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 15 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 16 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 17 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 18 | › **SOURCES** | › **SOURCES** |  | same |  |
| 19 | › External Sources | › External Sources |  | same |  |
| 20 | › Connected Sources | › Connected Sources |  | same |  |
| 21 | › Source Registry | › Source Registry |  | same |  |
| 22 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 23 | › Definitions | › Definitions |  | same |  |
| 24 | › Check Data | › Check Data |  | same |  |
| 25 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 26 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

#### Story

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **TIME** | **TIME** |  | same |  |
| 2 | Timeline · Ctrl+Shift+T | Timeline · Ctrl+Shift+T |  | same |  |
| 3 | Political history of the selection | Political history of the selection | Political history of this place… | same after the milestone; today: label | m PC-4 |
| 4 | Faction history… | Faction history… |  | same |  |
| 5 | **JOURNEYS** | **JOURNEYS** |  | same |  |
| 6 | Journey planner | Journey planner |  | same |  |
| 7 | Saved journeys… | Saved journeys… |  | same |  |
| 8 | Travel library… · Shift+L | Travel library… · Shift+L |  | same |  |
| 9 | **CONFLICTS & CAMPAIGNS** | **CONFLICTS & CAMPAIGNS** |  | same |  |
| 10 | Conflicts… | Conflicts… |  | same |  |
| 11 | Draw a conflict | Draw a conflict |  | same |  |
| 12 | [readout] Campaigns this year | [readout] Campaigns this year |  | same |  |
| 13 | **NOTES** | **NOTES** |  | same |  |
| 14 | Note for the selection · Ctrl+Shift+N | Note for the selection · Ctrl+Shift+N | Notes for this place · Ctrl+Shift+N | same after the milestone; today: label | m PC-5 |
| 15 | [signpost] Places without notes… is in Records | [signpost] Places without notes is in Records |  | same |  |
| 16 | Ask the assistant about the selection · Ctrl+J | Ask the assistant about the selection · Ctrl+J | Assistant panel… | same after the milestone; today: label, key Ctrl+J/none | m PC-6 |

#### Preferences

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **PERFORMANCE** | **PERFORMANCE** |  | same |  |
| 2 | [check] GPU acceleration | [check] GPU acceleration |  | same |  |
| 3 | [submenu] Devices | [submenu] Devices |  | same |  |
| 4 | › [check] Automatic (highest-performance GPU) | › [check] Automatic (highest-performance GPU) |  | same |  |
| 5 | › ——— | › ——— |  | same |  |
| 6 | › [check] Adreno 630 · integrated · Vulkan | › [check] … · … · … · run-time list |  | same (run-time list) |  |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › [readout] Memory | › [readout] Memory: not measured yet |  | same |  |
| 9 | › ——— | › ——— |  | same |  |
| 10 | › Rescan devices… | › Rescan devices… |  | same |  |
| 11 | [submenu] Multi-GPU mode | [submenu] Multi-GPU mode |  | same |  |
| 12 | › [radio] Single device | › [radio] Single device |  | same |  |
| 13 | › [radio] Split tiles | › [radio] Split tiles |  | same |  |
| 14 | › [not built] Alternate frames | › [radio] Alternate frames · dark |  | same |  |
| 15 | [submenu] CPU worker threads | [submenu] CPU worker threads |  | same |  |
| 16 | › [radio] Automatic | › [radio] Automatic |  | same |  |
| 17 | › [radio] 1 — 1 thread | › [radio] 1 — 1 thread |  | same |  |
| 18 | › [radio] 2 — quarter | › [radio] 2 — quarter |  | same |  |
| 19 | › [radio] 4 — half | › [radio] 4 — half |  | same |  |
| 20 | › [radio] 8 — all cores | › [radio] 8 — all cores |  | same |  |
| 21 | › [readout] Running | › [readout] Running … workers now |  | same |  |
| 22 | [submenu] VRAM budget | [submenu] VRAM budget |  | same |  |
| 23 | › [radio] No cap | › [radio] No cap |  | same |  |
| 24 | › [radio] 1 GB | › [radio] 1 GB |  | same |  |
| 25 | › [radio] 2 GB | › [radio] 2 GB |  | same |  |
| 26 | › [radio] 4 GB | › [radio] 4 GB |  | same |  |
| 27 | › [radio] 8 GB | › [radio] 8 GB |  | same |  |
| 28 | › [radio] 12 GB | › [radio] 12 GB |  | same |  |
| 29 | › [radio] 16 GB | › [radio] 16 GB |  | same |  |
| 30 | › [radio] 24 GB | › [radio] 24 GB |  | same |  |
| 31 | › [readout] Current grid 2048×2048 needs about | › [readout] Current grid …x… needs about … MB |  | same |  |
| 32 | [submenu] Fallback when VRAM full | [submenu] Fallback when VRAM full |  | same |  |
| 33 | › [radio] CPU tile pass | › [radio] CPU tile pass |  | same |  |
| 34 | › [not built] Reduce working res | › [radio] Reduce working res · dark |  | same |  |
| 35 | › [radio] Fail with error | › [radio] Fail with error |  | same |  |
| 36 | Try the GPU again · dark | Try the GPU again · dark |  | same |  |
| 37 | **GRAPHICS** | **GRAPHICS** |  | same |  |
| 38 | [submenu] Render quality | [submenu] Render quality |  | same |  |
| 39 | › [radio] performance | › [radio] performance |  | same |  |
| 40 | › [radio] balanced | › [radio] balanced |  | same |  |
| 41 | › [radio] quality | › [radio] quality |  | same |  |
| 42 | › [radio] ultra | › [radio] ultra |  | same |  |
| 43 | [not built] Anti-aliasing · anisotropy | [not built] Anti-aliasing · anisotropy |  | differs: reason | s S1 |
| 44 | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management |  | same |  |
| 45 | [not built] 3D viewport defaults | [not built] 3D viewport defaults |  | differs: reason | s S1 |
| 46 | [submenu] Lighting rig defaults | [submenu] Lighting rig defaults |  | same |  |
| 47 | › [readout] Following the engine | › [readout] Following the engine |  | same |  |
| 48 | › [submenu] Azimuth | › **AZIMUTH** |  | differs: kind submenu/caption | ?  |
| 49 | › › [radio] 0 deg |  |  | mockup-only | ?  |
| 50 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 51 | › › [radio] 90 deg |  |  | mockup-only | ?  |
| 52 | › › [radio] 135 deg |  |  | mockup-only | ?  |
| 53 | › › [radio] 180 deg |  |  | mockup-only | ?  |
| 54 | › › [radio] 225 deg |  |  | mockup-only | ?  |
| 55 | › › [radio] 270 deg |  |  | mockup-only | ?  |
| 56 | › › [radio] 315 deg |  |  | mockup-only | ?  |
| 57 |  | › [radio] 0 deg |  | Godot-only | ?  |
| 58 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 59 |  | › [radio] 90 deg |  | Godot-only | ?  |
| 60 |  | › [radio] 135 deg |  | Godot-only | ?  |
| 61 |  | › [radio] 180 deg |  | Godot-only | ?  |
| 62 |  | › [radio] 225 deg |  | Godot-only | ?  |
| 63 |  | › [radio] 270 deg |  | Godot-only | ?  |
| 64 |  | › [radio] 315 deg |  | Godot-only | ?  |
| 65 | › [submenu] Elevation | › **ELEVATION** |  | differs: kind submenu/caption | ?  |
| 66 | › › [radio] 15 deg |  |  | mockup-only | ?  |
| 67 | › › [radio] 25 deg |  |  | mockup-only | ?  |
| 68 | › › [radio] 35 deg |  |  | mockup-only | ?  |
| 69 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 70 | › › [radio] 55 deg |  |  | mockup-only | ?  |
| 71 | › › [radio] 65 deg |  |  | mockup-only | ?  |
| 72 | › › [radio] 75 deg |  |  | mockup-only | ?  |
| 73 |  | › [radio] 15 deg |  | Godot-only | ?  |
| 74 |  | › [radio] 25 deg |  | Godot-only | ?  |
| 75 |  | › [radio] 35 deg |  | Godot-only | ?  |
| 76 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 77 |  | › [radio] 55 deg |  | Godot-only | ?  |
| 78 |  | › [radio] 65 deg |  | Godot-only | ?  |
| 79 |  | › [radio] 75 deg |  | Godot-only | ?  |
| 80 | › [submenu] Ambient | › **AMBIENT** |  | differs: kind submenu/caption | ?  |
| 81 | › › [radio] 0.00 |  |  | mockup-only | ?  |
| 82 | › › [radio] 0.15 |  |  | mockup-only | ?  |
| 83 | › › [radio] 0.25 |  |  | mockup-only | ?  |
| 84 | › › [radio] 0.35 |  |  | mockup-only | ?  |
| 85 | › › [radio] 0.50 |  |  | mockup-only | ?  |
| 86 | › › [radio] 0.70 |  |  | mockup-only | ?  |
| 87 |  | › [radio] 0.00 |  | Godot-only | ?  |
| 88 |  | › [radio] 0.15 |  | Godot-only | ?  |
| 89 |  | › [radio] 0.25 |  | Godot-only | ?  |
| 90 |  | › [radio] 0.35 |  | Godot-only | ?  |
| 91 |  | › [radio] 0.50 |  | Godot-only | ?  |
| 92 |  | › [radio] 0.70 |  | Godot-only | ?  |
| 93 | › [submenu] Multidirectional | › **MULTIDIRECTIONAL** |  | differs: kind submenu/caption | ?  |
| 94 | › › [radio] 1 light (single sun) |  |  | mockup-only | ?  |
| 95 | › › [radio] 2 lights |  |  | mockup-only | ?  |
| 96 | › › [radio] 4 lights |  |  | mockup-only | ?  |
| 97 | › › [radio] 6 lights |  |  | mockup-only | ?  |
| 98 | › › [radio] 8 lights |  |  | mockup-only | ?  |
| 99 | › › [radio] 12 lights |  |  | mockup-only | ?  |
| 100 |  | › [radio] 1 light (single sun) |  | Godot-only | ?  |
| 101 |  | › [radio] 2 lights |  | Godot-only | ?  |
| 102 |  | › [radio] 4 lights |  | Godot-only | ?  |
| 103 |  | › [radio] 6 lights |  | Godot-only | ?  |
| 104 |  | › [radio] 8 lights |  | Godot-only | ?  |
| 105 |  | › [radio] 12 lights |  | Godot-only | ?  |
| 106 | › ——— | › ——— |  | same |  |
| 107 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 108 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 109 | › [radio] 1x | › [radio] 1x |  | same |  |
| 110 | › [radio] 2x | › [radio] 2x |  | same |  |
| 111 | › [radio] 4x | › [radio] 4x |  | same |  |
| 112 | › ——— | › ——— |  | same |  |
| 113 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 114 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 115 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 116 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 117 | › [radio] Manual | › [radio] Manual |  | same |  |
| 118 | › ——— | › ——— |  | same |  |
| 119 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 120 | › Leave deep detail | › Leave deep detail |  | same |  |
| 121 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 122 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 123 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 124 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 125 | › ——— |  |  | mockup-only | ?  |
| 126 | › [submenu] LOD levels | › **LOD LEVELS** |  | differs: kind submenu/caption | ?  |
| 127 | › › [radio] LOD 0–0   1 tile |  |  | mockup-only | ?  |
| 128 | › › [radio] LOD 0–1   5 tiles |  |  | mockup-only | ?  |
| 129 | › › [radio] LOD 0–2   21 tiles |  |  | mockup-only | ?  |
| 130 | › › [radio] LOD 0–3   85 tiles |  |  | mockup-only | ?  |
| 131 | › › [radio] LOD 0–4   341 tiles |  |  | mockup-only | ?  |
| 132 | › › [radio] LOD 0–5   1 365 tiles |  |  | mockup-only | ?  |
| 133 | › › [radio] LOD 0–6   5 461 tiles |  |  | mockup-only | ?  |
| 134 | › › [radio] LOD 0–7   21 845 tiles |  |  | mockup-only | ?  |
| 135 | › › [radio] LOD 0–8   87 381 tiles |  |  | mockup-only | ?  |
| 136 |  | › [radio] LOD 0–0   1 tile |  | Godot-only | ?  |
| 137 |  | › [radio] LOD 0–1   5 tiles |  | Godot-only | ?  |
| 138 |  | › [radio] LOD 0–2   21 tiles |  | Godot-only | ?  |
| 139 |  | › [radio] LOD 0–3   85 tiles |  | Godot-only | ?  |
| 140 |  | › [radio] LOD 0–4   341 tiles |  | Godot-only | ?  |
| 141 |  | › [radio] LOD 0–5   1 365 tiles |  | Godot-only | ?  |
| 142 |  | › [radio] LOD 0–6   5 461 tiles |  | Godot-only | ?  |
| 143 |  | › [radio] LOD 0–7   21 845 tiles |  | Godot-only | ?  |
| 144 |  | › [radio] LOD 0–8   87 381 tiles |  | Godot-only | ?  |
| 145 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 146 | › [readout] 85 chunks · 1.2 GB · to LOD 3 |  |  | mockup-only | ?  |
| 147 |  | › [readout] … chunks · … · empty |  | Godot-only | ?  |
| 148 | › [submenu] Size cap · GB | › **SIZE CAP · GB** |  | differs: kind submenu/caption | ?  |
| 149 | › › [radio] No cap |  |  | mockup-only | ?  |
| 150 | › › [radio] 0.5 GB |  |  | mockup-only | ?  |
| 151 | › › [radio] 1 GB |  |  | mockup-only | ?  |
| 152 | › › [radio] 2 GB |  |  | mockup-only | ?  |
| 153 | › › [radio] 4 GB |  |  | mockup-only | ?  |
| 154 | › › [radio] 8 GB |  |  | mockup-only | ?  |
| 155 | › › [radio] 16 GB |  |  | mockup-only | ?  |
| 156 |  | › [radio] No cap |  | Godot-only | ?  |
| 157 |  | › [radio] 0.5 GB |  | Godot-only | ?  |
| 158 |  | › [radio] 1 GB |  | Godot-only | ?  |
| 159 |  | › [radio] 2 GB |  | Godot-only | ?  |
| 160 |  | › [radio] 4 GB |  | Godot-only | ?  |
| 161 |  | › [radio] 8 GB |  | Godot-only | ?  |
| 162 |  | › [radio] 16 GB |  | Godot-only | ?  |
| 163 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 164 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 165 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 166 | › ——— | › ——— |  | same |  |
| 167 | › Export atlas… | › Export atlas… |  | same |  |
| 168 | › Import atlas… | › Import atlas… |  | same |  |
| 169 | › ——— | › ——— |  | same |  |
| 170 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 171 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 172 | › [check] Grid | › [check] Grid |  | same |  |
| 173 | › [check] Colors | › [check] Colors |  | same |  |
| 174 | › [check] Labels | › [check] Labels |  | same |  |
| 175 | › ——— | › ——— |  | same |  |
| 176 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 177 | **MEMORY** | **MEMORY** |  | same |  |
| 178 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 179 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 180 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 181 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 182 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 183 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 184 | › ——— | › ——— |  | same |  |
| 185 | › Clear undo history now | › Clear undo history now |  | same |  |
| 186 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 187 | Clear caches… | Clear caches… |  | same |  |
| 188 | **APPLICATION** | **APPLICATION** |  | same |  |
| 189 | Storage locations… | Storage locations… |  | same |  |
| 190 | Assistant… | Assistant… |  | same |  |
| 191 | Quick bar… | Quick bar… |  | same |  |
| 192 | [DESKTOP ONLY] Open panels in their own window |  |  | mockup-only | n N3 |
| 193 | [submenu] Theme | [submenu] Theme |  | same |  |
| 194 | › [radio] Dark | › [radio] Dark |  | same |  |
| 195 | › [radio] Light | › [radio] Light |  | same |  |
| 196 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 197 | [submenu] Units | [submenu] Units |  | same |  |
| 198 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 199 | › [radio] Miles | › [radio] Miles |  | same |  |
| 200 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 201 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 202 | › [radio] Left | › [radio] Left |  | same |  |
| 203 | › [radio] Right | › [radio] Right |  | same |  |
| 204 |  | [submenu] UI scale |  | Godot-only | ?  |
| 205 |  | › [radio] 100 % |  | Godot-only (inside) | ?  |
| 206 |  | › [radio] 125 % |  | Godot-only (inside) | ?  |
| 207 |  | › [radio] 150 % |  | Godot-only (inside) | ?  |
| 208 |  | › [radio] 200 % |  | Godot-only (inside) | ?  |
| 209 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |
| 210 | **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 211 | Touch & gestures… |  |  | mockup-only | n N1 |

#### Window

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [check] Left dock | [check] Left dock |  | same |  |
| 2 | [check] Context panel | [check] Context panel | [check] Right dock | same after the milestone; today: label | m PC-4 |
| 3 | [submenu] Context panel | [submenu] Context panel |  | same |  |
| 4 | › Inspect · Ctrl+1 | › Inspect · Ctrl+1 | › [not built] Inspect | same after the milestone; today: kind command/disabled (not built), key Ctrl+1/none | m PC-4 |
| 5 | › Note · Ctrl+2 | › Note · Ctrl+2 | › [not built] Note | same after the milestone; today: kind command/disabled (not built), key Ctrl+2/none | m PC-5 |
| 6 | › Ask (assistant) · Ctrl+3 | › Ask (assistant) · Ctrl+3 | › [not built] Ask (assistant) | same after the milestone; today: kind command/disabled (not built), key Ctrl+3/none | m PC-6 |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › Pin to the current subject | › Pin to the current subject | › [not built] Pin to the current subject | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 9 | › Follow the selection | › Follow the selection | › [not built] Follow the selection | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 10 | › **MODULES** | › **MODULES** |  | same |  |
| 11 | › [check] History | › [not built] History |  | differs: kind check/disabled (not built) | m PC-10 |
| 12 | › [check] Economy | › [not built] Economy |  | differs: kind check/disabled (not built) | m PC-10 |
| 13 | › [check] Military | › [not built] Military |  | differs: kind check/disabled (not built) | m PC-10 |
| 14 | › [check] Faith, culture & relations | › [not built] Faith, culture & relations |  | differs: kind check/disabled (not built) | m PC-10 |
| 15 | › [check] Territory | › [not built] Territory |  | differs: kind check/disabled (not built) | m PC-10 |
| 16 | › [check] City layout | › [not built] City layout |  | differs: kind check/disabled (not built) | m PC-10 |
| 17 | › [check] Here | › [not built] Here |  | differs: kind check/disabled (not built) | m PC-10 |
| 18 | › [check] Routes & journeys | › [not built] Routes & journeys |  | differs: kind check/disabled (not built) | m PC-10 |
| 19 | › [check] Links | › [not built] Links |  | differs: kind check/disabled (not built) | m PC-10 |
| 20 | › [check] Recent & bookmarks | › [not built] Recent & bookmarks |  | differs: kind check/disabled (not built) | m PC-10 |
| 21 | › ——— | › ——— |  | same |  |
| 22 | › Reset panel | › Reset panel |  | same |  |
| 23 | [check] Timeline | [check] Timeline |  | same |  |
| 24 | [check] Status bar | [check] Status bar |  | same |  |
| 25 | [check] Domain rail | [check] Domain rail |  | same |  |
| 26 | ——— | ——— |  | same |  |
| 27 | [check] Session mode · Ctrl+Shift+M | [check] Session mode · Ctrl+Shift+M |  | same |  |
| 28 | [check] Diagnostics overlay · Shift+D | [check] Diagnostics overlay · Shift+D |  | same |  |
| 29 | [check] 3D view | [check] 3D view |  | same |  |
| 30 | [submenu] 3D capability | [submenu] 3D capability |  | same |  |
| 31 | › [readout] 3D tier | › [readout] 3D tier: not tested |  | same |  |
| 32 | › Re-test 3D | › Re-test 3D |  | same |  |
| 33 | › [submenu] Force tier |  |  | mockup-only | ?  |
| 34 | › › [radio] Automatic (as measured) |  |  | mockup-only (inside) | ?  |
| 35 | › › [radio] Off |  |  | mockup-only (inside) | ?  |
| 36 | › › [radio] Basic |  |  | mockup-only (inside) | ?  |
| 37 | › › [radio] Standard |  |  | mockup-only (inside) | ?  |
| 38 | › › [radio] Full |  |  | mockup-only (inside) | ?  |
| 39 |  | › **FORCE TIER** |  | Godot-only | ?  |
| 40 |  | › [radio] Automatic (as measured) |  | Godot-only | ?  |
| 41 |  | › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | Godot-only | ?  |
| 42 | ——— | ——— |  | same |  |
| 43 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 44 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 45 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 46 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 47 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 48 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 49 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 50 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 51 | ——— | ——— |  | same |  |
| 52 | Reset layout | Reset layout |  | same |  |
| 53 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 54 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 55 | › Generate | › Generate |  | same |  |
| 56 | › Sculpt | › Sculpt |  | same |  |
| 57 | › Cartography | › Cartography |  | same |  |
| 58 | › Journey | › Journey |  | same |  |
| 59 | › ——— | › ——— |  | same |  |
| 60 | › Save layout as… | › Save layout as… |  | same |  |
| 61 | › Forget layout… | › Forget layout… |  | same |  |

#### Help

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [not built] Documentation | [not built] Documentation |  | differs: reason | s S1 |
| 2 | Keyboard reference… | Keyboard reference… |  | same |  |
| 3 | Gesture reference… |  |  | mockup-only | n N2 |
| 4 | Credits & academic principles | Credits & academic principles |  | same |  |
| 5 | Generation info… | Generation info… |  | same |  |
| 6 | Save diagnostic report… | Save diagnostic report… |  | same |  |
| 7 | About | About |  | same |  |

### Tablet, below 1 180 dp

#### ☰

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Undo · Ctrl+Z | Undo · Ctrl+Z |  | same |  |
| 2 | Redo · Ctrl+Shift+Z | Redo · Ctrl+Shift+Z |  | same |  |
| 3 | Undo history… | Undo history… |  | same |  |
| 4 | ——— | ——— |  | same |  |
| 5 | ⧉ Asset library · Shift+A | ⧉ Asset library · Shift+A |  | same |  |
| 6 | [submenu] Landmark types | Landmark types… |  | differs: kind submenu/command | (c) EH-2 |
| 7 | › [submenu] Physical |  |  | mockup-only | (c) EH-2 |
| 8 | › › Peak |  |  | mockup-only (inside) | (c) EH-2 |
| 9 | › › Ridge |  |  | mockup-only (inside) | (c) EH-2 |
| 10 | › › Saddle   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 11 | › › Cliff |  |  | mockup-only (inside) | (c) EH-2 |
| 12 | › › Gorge |  |  | mockup-only (inside) | (c) EH-2 |
| 13 | › › Cave |  |  | mockup-only (inside) | (c) EH-2 |
| 14 | › › Waterfall |  |  | mockup-only (inside) | (c) EH-2 |
| 15 | › › Spring |  |  | mockup-only (inside) | (c) EH-2 |
| 16 | › › Lake |  |  | mockup-only (inside) | (c) EH-2 |
| 17 | › › Delta   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 18 | › › River confluence |  |  | mockup-only (inside) | (c) EH-2 |
| 19 | › › Volcanic feature |  |  | mockup-only (inside) | (c) EH-2 |
| 20 | › › Rock formation |  |  | mockup-only (inside) | (c) EH-2 |
| 21 | › › Glacial feature   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 22 | › › Ancient forest   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 23 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 24 | › › Open Physical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 25 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 26 | › [submenu] Transport |  |  | mockup-only | (c) EH-2 |
| 27 | › › Mountain pass |  |  | mockup-only (inside) | (c) EH-2 |
| 28 | › › River crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 29 | › › Ford |  |  | mockup-only (inside) | (c) EH-2 |
| 30 | › › Bridge site |  |  | mockup-only (inside) | (c) EH-2 |
| 31 | › › Road junction |  |  | mockup-only (inside) | (c) EH-2 |
| 32 | › › Caravan station |  |  | mockup-only (inside) | (c) EH-2 |
| 33 | › › Portage |  |  | mockup-only (inside) | (c) EH-2 |
| 34 | › › Harbour |  |  | mockup-only (inside) | (c) EH-2 |
| 35 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 36 | › › Open Transport in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 37 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 38 | › [submenu] Economic |  |  | mockup-only | (c) EH-2 |
| 39 | › › Mine |  |  | mockup-only (inside) | (c) EH-2 |
| 40 | › › Quarry |  |  | mockup-only (inside) | (c) EH-2 |
| 41 | › › Salt works   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 42 | › › Resource extraction site |  |  | mockup-only (inside) | (c) EH-2 |
| 43 | › › Market site |  |  | mockup-only (inside) | (c) EH-2 |
| 44 | › › Trade depot |  |  | mockup-only (inside) | (c) EH-2 |
| 45 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 46 | › › Open Economic in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 47 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 48 | › [submenu] Military |  |  | mockup-only | (c) EH-2 |
| 49 | › › Fort |  |  | mockup-only (inside) | (c) EH-2 |
| 50 | › › Watchtower |  |  | mockup-only (inside) | (c) EH-2 |
| 51 | › › Fortified pass |  |  | mockup-only (inside) | (c) EH-2 |
| 52 | › › Fortified crossing |  |  | mockup-only (inside) | (c) EH-2 |
| 53 | › › Battlefield |  |  | mockup-only (inside) | (c) EH-2 |
| 54 | › › Border marker |  |  | mockup-only (inside) | (c) EH-2 |
| 55 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 56 | › › Open Military in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 57 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 58 | › [submenu] Cultural |  |  | mockup-only | (c) EH-2 |
| 59 | › › Shrine   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 60 | › › Temple   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 61 | › › Sacred grove   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 62 | › › Sacred mountain   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 63 | › › Pilgrimage site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 64 | › › Tomb   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 65 | › › Monument   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 66 | › › Ceremonial site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 67 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 68 | › › Open Cultural in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 69 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 70 | › [submenu] Historical |  |  | mockup-only | (c) EH-2 |
| 71 | › › Ruin   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 72 | › › Abandoned settlement |  |  | mockup-only (inside) | (c) EH-2 |
| 73 | › › Ancient road |  |  | mockup-only (inside) | (c) EH-2 |
| 74 | › › Historic battlefield   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 75 | › › Destroyed fortress   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 76 | › › Historic crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 77 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 78 | › › Open Historical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 79 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 80 | › Placed by hand   3 |  |  | mockup-only | (c) EH-2 |
| 81 | › ——— |  |  | mockup-only | (c) EH-2 |
| 82 | › Landmark icons… |  |  | mockup-only | (c) EH-2 |
| 83 | › Landmark label style… |  |  | mockup-only | (c) EH-2 |
| 84 | ——— | ——— |  | same |  |
| 85 | Toggle theme | Toggle theme |  | same |  |
| 86 | [submenu] Units | [submenu] Units |  | same |  |
| 87 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 88 | › [radio] Miles | › [radio] Miles |  | same |  |
| 89 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 90 | ——— | ——— |  | same |  |
| 91 | Keyboard reference… | Keyboard reference… |  | same |  |
| 92 | About | About |  | same |  |
| 93 | ——— | ——— |  | same |  |
| 94 | [submenu] Edit | [submenu] Edit |  | same |  |
| 95 | › Cut · Ctrl+X | › Cut · Ctrl+X |  | same |  |
| 96 | › Copy · Ctrl+C | › Copy · Ctrl+C |  | same |  |
| 97 | › Paste · Ctrl+V | › Paste · Ctrl+V |  | same |  |
| 98 | › Delete · Delete | › Delete · Delete |  | same |  |
| 99 | › ——— | › ——— |  | same |  |
| 100 | › Select all · Ctrl+A | › Select all · Ctrl+A |  | same |  |
| 101 | › Deselect · Ctrl+D | › Deselect · Ctrl+D |  | same |  |
| 102 | › Previous selection · Alt+← | › Previous selection · Alt+← |  | same |  |
| 103 | › Next selection · Alt+→ | › Next selection · Alt+→ |  | same |  |
| 104 | › ——— | › ——— |  | same |  |
| 105 | › Find on map… · Ctrl+F | › Find on map… · Ctrl+F |  | same |  |
| 106 | › Context card · Shift+F10 | › Context card · Shift+F10 |  | same |  |
| 107 | › Cartwheel at the pointer · Q | › Cartwheel at the pointer · Q |  | same |  |
| 108 | [submenu] Assets | [submenu] Assets |  | same |  |
| 109 | › ⧉ Sprite sheet slicer (▦) | › ⧉ Sprite sheet slicer (▦) |  | same |  |
| 110 | › Import asset pack .zip… | › Import asset pack .zip… |  | same |  |
| 111 | › Open pack in library… | › Open pack in library… |  | same |  |
| 112 | › Pack report… | › Pack report… |  | same |  |
| 113 | › Import folder of art… | › Import folder of art… |  | same |  |
| 114 | › Compare pack with library… | › Compare pack with library… |  | same |  |
| 115 | › ——— | › ——— |  | same |  |
| 116 | › [submenu] Asset pack | › [submenu] Asset pack |  | same |  |
| 117 | › › **ACTIVE PACK** | › › **ACTIVE PACK** |  | same |  |
| 118 | › › [readout] NAME | › › [readout] NAME   — loading — |  | same |  |
| 119 | › › [readout] AUTHOR | › › [readout] AUTHOR   — loading — |  | same |  |
| 120 | › › [readout] LICENSE | › › [readout] LICENSE   — loading — |  | same |  |
| 121 | › › [readout] SCHEMA | › › [readout] SCHEMA   — loading — |  | same |  |
| 122 | › › [readout] FILLED | › › [readout] FILLED   — loading — |  | same |  |
| 123 | › › Pack metadata… | › › Pack metadata… |  | same |  |
| 124 | › › Validate pack | › › Validate pack |  | same |  |
| 125 | › › Export pack .zip… · Ctrl+Shift+P | › › Export pack .zip… · Ctrl+Shift+P |  | same |  |
| 126 | › [submenu] Icon families | › [submenu] Icon families |  | same |  |
| 127 | › › Feature icons   22/31 | › › Feature icons |  | same |  |
| 128 | › › Settlement pins   8/10 | › › Settlement pins |  | same |  |
| 129 | › › Settlement traits   5/12 | › › Settlement traits |  | same |  |
| 130 | › › Points of interest   14/21 | › › Points of interest |  | same |  |
| 131 | › › Sea marks   4/9 | › › Sea marks |  | same |  |
| 132 | › › Custom icons   0/6 | › › Custom icons |  | same |  |
| 133 | › [submenu] Texture sets | › [submenu] Texture sets |  | same |  |
| 134 | › › Splat channels   4/4 | › › Splat channels |  | same |  |
| 135 | › › Biome textures   9/12 | › › Biome textures |  | same |  |
| 136 | › › Terrain textures   6/8 | › › Terrain textures |  | same |  |
| 137 | › ——— | › ——— |  | same |  |
| 138 | › Apply library to map | › Apply library to map |  | same |  |
| 139 | › Clear library… | › Clear library…   destructive |  | same |  |
| 140 | [submenu] Preferences | [submenu] Preferences |  | same |  |
| 141 | › **PERFORMANCE** | › **PERFORMANCE** |  | same |  |
| 142 | › [check] GPU acceleration | › [check] GPU acceleration |  | same |  |
| 143 | › [submenu] Devices | › [submenu] Devices |  | same |  |
| 144 | › › [check] Automatic (highest-performance GPU) | › › [check] Automatic (highest-performance GPU) |  | same |  |
| 145 | › › ——— | › › ——— |  | same |  |
| 146 | › › [check] Adreno 630 · integrated · Vulkan | › › [check] … · … · … · run-time list |  | same (run-time list) |  |
| 147 | › › ——— | › › ——— |  | same |  |
| 148 | › › [readout] Memory | › › [readout] Memory: not measured yet |  | same |  |
| 149 | › › ——— | › › ——— |  | same |  |
| 150 | › › Rescan devices… | › › Rescan devices… |  | same |  |
| 151 | › [submenu] Multi-GPU mode | › [submenu] Multi-GPU mode |  | same |  |
| 152 | › › [radio] Single device | › › [radio] Single device |  | same |  |
| 153 | › › [radio] Split tiles | › › [radio] Split tiles |  | same |  |
| 154 | › › [not built] Alternate frames | › › [radio] Alternate frames · dark |  | same |  |
| 155 | › [submenu] CPU worker threads | › [submenu] CPU worker threads |  | same |  |
| 156 | › › [radio] Automatic | › › [radio] Automatic |  | same |  |
| 157 | › › [radio] 1 — 1 thread | › › [radio] 1 — 1 thread |  | same |  |
| 158 | › › [radio] 2 — quarter | › › [radio] 2 — quarter |  | same |  |
| 159 | › › [radio] 4 — half | › › [radio] 4 — half |  | same |  |
| 160 | › › [radio] 8 — all cores | › › [radio] 8 — all cores |  | same |  |
| 161 | › › [readout] Running | › › [readout] Running … workers now |  | same |  |
| 162 | › [submenu] VRAM budget | › [submenu] VRAM budget |  | same |  |
| 163 | › › [radio] No cap | › › [radio] No cap |  | same |  |
| 164 | › › [radio] 1 GB | › › [radio] 1 GB |  | same |  |
| 165 | › › [radio] 2 GB | › › [radio] 2 GB |  | same |  |
| 166 | › › [radio] 4 GB | › › [radio] 4 GB |  | same |  |
| 167 | › › [radio] 8 GB | › › [radio] 8 GB |  | same |  |
| 168 | › › [radio] 12 GB | › › [radio] 12 GB |  | same |  |
| 169 | › › [radio] 16 GB | › › [radio] 16 GB |  | same |  |
| 170 | › › [radio] 24 GB | › › [radio] 24 GB |  | same |  |
| 171 | › › [readout] Current grid 2048×2048 needs about | › › [readout] Current grid …x… needs about … MB |  | same |  |
| 172 | › [submenu] Fallback when VRAM full | › [submenu] Fallback when VRAM full |  | same |  |
| 173 | › › [radio] CPU tile pass | › › [radio] CPU tile pass |  | same |  |
| 174 | › › [not built] Reduce working res | › › [radio] Reduce working res · dark |  | same |  |
| 175 | › › [radio] Fail with error | › › [radio] Fail with error |  | same |  |
| 176 | › Try the GPU again · dark | › Try the GPU again · dark |  | same |  |
| 177 | › **GRAPHICS** | › **GRAPHICS** |  | same |  |
| 178 | › [submenu] Render quality | › [submenu] Render quality |  | same |  |
| 179 | › › [radio] performance | › › [radio] performance |  | same |  |
| 180 | › › [radio] balanced | › › [radio] balanced |  | same |  |
| 181 | › › [radio] quality | › › [radio] quality |  | same |  |
| 182 | › › [radio] ultra | › › [radio] ultra |  | same |  |
| 183 | › [not built] Anti-aliasing · anisotropy | › [not built] Anti-aliasing · anisotropy |  | differs: reason | s S1 |
| 184 | › [signpost] Colour management — CARTO ▸ Colours ▸ Colour management | › [signpost] Colour management — CARTO ▸ Colours ▸ Colour management |  | same |  |
| 185 | › [not built] 3D viewport defaults | › [not built] 3D viewport defaults |  | differs: reason | s S1 |
| 186 | › [submenu] Lighting rig defaults | › [submenu] Lighting rig defaults |  | same |  |
| 187 | › › [readout] Following the engine | › › [readout] Following the engine |  | same |  |
| 188 | › › [submenu] Azimuth | › › **AZIMUTH** |  | differs: kind submenu/caption | ?  |
| 189 | › › › [radio] 0 deg |  |  | mockup-only | ?  |
| 190 | › › › [radio] 45 deg |  |  | mockup-only | ?  |
| 191 | › › › [radio] 90 deg |  |  | mockup-only | ?  |
| 192 | › › › [radio] 135 deg |  |  | mockup-only | ?  |
| 193 | › › › [radio] 180 deg |  |  | mockup-only | ?  |
| 194 | › › › [radio] 225 deg |  |  | mockup-only | ?  |
| 195 | › › › [radio] 270 deg |  |  | mockup-only | ?  |
| 196 | › › › [radio] 315 deg |  |  | mockup-only | ?  |
| 197 |  | › › [radio] 0 deg |  | Godot-only | ?  |
| 198 |  | › › [radio] 45 deg |  | Godot-only | ?  |
| 199 |  | › › [radio] 90 deg |  | Godot-only | ?  |
| 200 |  | › › [radio] 135 deg |  | Godot-only | ?  |
| 201 |  | › › [radio] 180 deg |  | Godot-only | ?  |
| 202 |  | › › [radio] 225 deg |  | Godot-only | ?  |
| 203 |  | › › [radio] 270 deg |  | Godot-only | ?  |
| 204 |  | › › [radio] 315 deg |  | Godot-only | ?  |
| 205 | › › [submenu] Elevation | › › **ELEVATION** |  | differs: kind submenu/caption | ?  |
| 206 | › › › [radio] 15 deg |  |  | mockup-only | ?  |
| 207 | › › › [radio] 25 deg |  |  | mockup-only | ?  |
| 208 | › › › [radio] 35 deg |  |  | mockup-only | ?  |
| 209 | › › › [radio] 45 deg |  |  | mockup-only | ?  |
| 210 | › › › [radio] 55 deg |  |  | mockup-only | ?  |
| 211 | › › › [radio] 65 deg |  |  | mockup-only | ?  |
| 212 | › › › [radio] 75 deg |  |  | mockup-only | ?  |
| 213 |  | › › [radio] 15 deg |  | Godot-only | ?  |
| 214 |  | › › [radio] 25 deg |  | Godot-only | ?  |
| 215 |  | › › [radio] 35 deg |  | Godot-only | ?  |
| 216 |  | › › [radio] 45 deg |  | Godot-only | ?  |
| 217 |  | › › [radio] 55 deg |  | Godot-only | ?  |
| 218 |  | › › [radio] 65 deg |  | Godot-only | ?  |
| 219 |  | › › [radio] 75 deg |  | Godot-only | ?  |
| 220 | › › [submenu] Ambient | › › **AMBIENT** |  | differs: kind submenu/caption | ?  |
| 221 | › › › [radio] 0.00 |  |  | mockup-only | ?  |
| 222 | › › › [radio] 0.15 |  |  | mockup-only | ?  |
| 223 | › › › [radio] 0.25 |  |  | mockup-only | ?  |
| 224 | › › › [radio] 0.35 |  |  | mockup-only | ?  |
| 225 | › › › [radio] 0.50 |  |  | mockup-only | ?  |
| 226 | › › › [radio] 0.70 |  |  | mockup-only | ?  |
| 227 |  | › › [radio] 0.00 |  | Godot-only | ?  |
| 228 |  | › › [radio] 0.15 |  | Godot-only | ?  |
| 229 |  | › › [radio] 0.25 |  | Godot-only | ?  |
| 230 |  | › › [radio] 0.35 |  | Godot-only | ?  |
| 231 |  | › › [radio] 0.50 |  | Godot-only | ?  |
| 232 |  | › › [radio] 0.70 |  | Godot-only | ?  |
| 233 | › › [submenu] Multidirectional | › › **MULTIDIRECTIONAL** |  | differs: kind submenu/caption | ?  |
| 234 | › › › [radio] 1 light (single sun) |  |  | mockup-only | ?  |
| 235 | › › › [radio] 2 lights |  |  | mockup-only | ?  |
| 236 | › › › [radio] 4 lights |  |  | mockup-only | ?  |
| 237 | › › › [radio] 6 lights |  |  | mockup-only | ?  |
| 238 | › › › [radio] 8 lights |  |  | mockup-only | ?  |
| 239 | › › › [radio] 12 lights |  |  | mockup-only | ?  |
| 240 |  | › › [radio] 1 light (single sun) |  | Godot-only | ?  |
| 241 |  | › › [radio] 2 lights |  | Godot-only | ?  |
| 242 |  | › › [radio] 4 lights |  | Godot-only | ?  |
| 243 |  | › › [radio] 6 lights |  | Godot-only | ?  |
| 244 |  | › › [radio] 8 lights |  | Godot-only | ?  |
| 245 |  | › › [radio] 12 lights |  | Godot-only | ?  |
| 246 | › › ——— | › › ——— |  | same |  |
| 247 | › › Use the engine’s own rig | › › Use the engine's own rig |  | same |  |
| 248 | › [submenu] Relief exaggeration | › [submenu] Relief exaggeration |  | same |  |
| 249 | › › [radio] 1x | › › [radio] 1x |  | same |  |
| 250 | › › [radio] 2x | › › [radio] 2x |  | same |  |
| 251 | › › [radio] 4x | › › [radio] 4x |  | same |  |
| 252 | › › ——— | › › ——— |  | same |  |
| 253 | › › Use the engine’s own 3.4x | › › Use the engine's own 3.4x |  | same |  |
| 254 | › **TILES & LOD** | › **TILES & LOD** |  | same |  |
| 255 | › [submenu] Tiled LOD | › [submenu] Tiled LOD |  | same |  |
| 256 | › › [radio] Auto on zoom | › › [radio] Auto on zoom |  | same |  |
| 257 | › › [radio] Manual | › › [radio] Manual |  | same |  |
| 258 | › › ——— | › › ——— |  | same |  |
| 259 | › › Enter deep detail now | › › Enter deep detail now |  | same |  |
| 260 | › › Leave deep detail | › › Leave deep detail |  | same |  |
| 261 | › [submenu] Tile size · LOD levels | › [submenu] Tile size · LOD levels |  | same |  |
| 262 | › › [radio] 256 px | › › [radio] 256 px |  | same |  |
| 263 | › › [radio] 512 px | › › [radio] 512 px |  | same |  |
| 264 | › › [radio] 1024 px | › › [radio] 1024 px |  | same |  |
| 265 | › › ——— |  |  | mockup-only | ?  |
| 266 | › › [submenu] LOD levels | › › **LOD LEVELS** |  | differs: kind submenu/caption | ?  |
| 267 | › › › [radio] LOD 0–0   1 tile |  |  | mockup-only | ?  |
| 268 | › › › [radio] LOD 0–1   5 tiles |  |  | mockup-only | ?  |
| 269 | › › › [radio] LOD 0–2   21 tiles |  |  | mockup-only | ?  |
| 270 | › › › [radio] LOD 0–3   85 tiles |  |  | mockup-only | ?  |
| 271 | › › › [radio] LOD 0–4   341 tiles |  |  | mockup-only | ?  |
| 272 | › › › [radio] LOD 0–5   1 365 tiles |  |  | mockup-only | ?  |
| 273 | › › › [radio] LOD 0–6   5 461 tiles |  |  | mockup-only | ?  |
| 274 | › › › [radio] LOD 0–7   21 845 tiles |  |  | mockup-only | ?  |
| 275 | › › › [radio] LOD 0–8   87 381 tiles |  |  | mockup-only | ?  |
| 276 |  | › › [radio] LOD 0–0   1 tile |  | Godot-only | ?  |
| 277 |  | › › [radio] LOD 0–1   5 tiles |  | Godot-only | ?  |
| 278 |  | › › [radio] LOD 0–2   21 tiles |  | Godot-only | ?  |
| 279 |  | › › [radio] LOD 0–3   85 tiles |  | Godot-only | ?  |
| 280 |  | › › [radio] LOD 0–4   341 tiles |  | Godot-only | ?  |
| 281 |  | › › [radio] LOD 0–5   1 365 tiles |  | Godot-only | ?  |
| 282 |  | › › [radio] LOD 0–6   5 461 tiles |  | Godot-only | ?  |
| 283 |  | › › [radio] LOD 0–7   21 845 tiles |  | Godot-only | ?  |
| 284 |  | › › [radio] LOD 0–8   87 381 tiles |  | Godot-only | ?  |
| 285 | › [submenu] Atlas cache | › [submenu] Atlas cache |  | same |  |
| 286 | › › [readout] 85 chunks · 1.2 GB · to LOD 3 |  |  | mockup-only | ?  |
| 287 |  | › › [readout] … chunks · … · empty |  | Godot-only | ?  |
| 288 | › › [submenu] Size cap · GB | › › **SIZE CAP · GB** |  | differs: kind submenu/caption | ?  |
| 289 | › › › [radio] No cap |  |  | mockup-only | ?  |
| 290 | › › › [radio] 0.5 GB |  |  | mockup-only | ?  |
| 291 | › › › [radio] 1 GB |  |  | mockup-only | ?  |
| 292 | › › › [radio] 2 GB |  |  | mockup-only | ?  |
| 293 | › › › [radio] 4 GB |  |  | mockup-only | ?  |
| 294 | › › › [radio] 8 GB |  |  | mockup-only | ?  |
| 295 | › › › [radio] 16 GB |  |  | mockup-only | ?  |
| 296 |  | › › [radio] No cap |  | Godot-only | ?  |
| 297 |  | › › [radio] 0.5 GB |  | Godot-only | ?  |
| 298 |  | › › [radio] 1 GB |  | Godot-only | ?  |
| 299 |  | › › [radio] 2 GB |  | Godot-only | ?  |
| 300 |  | › › [radio] 4 GB |  | Godot-only | ?  |
| 301 |  | › › [radio] 8 GB |  | Godot-only | ?  |
| 302 |  | › › [radio] 16 GB |  | Godot-only | ?  |
| 303 | › › Refine detail for the current view | › › Refine detail for the current view |  | same |  |
| 304 | › › Refine viewshed for the current view | › › Refine viewshed for the current view |  | same |  |
| 305 | › › [readout] Viewshed | › › [readout] Viewshed: never refined |  | same |  |
| 306 | › › ——— | › › ——— |  | same |  |
| 307 | › › Export atlas… | › › Export atlas… |  | same |  |
| 308 | › › Import atlas… | › › Import atlas… |  | same |  |
| 309 | › › ——— | › › ——— |  | same |  |
| 310 | › › Clear atlas cache now… | › › Clear atlas cache now… |  | same |  |
| 311 | › [submenu] Chunk debug overlay | › [submenu] Chunk debug overlay |  | same |  |
| 312 | › › [check] Grid | › › [check] Grid |  | same |  |
| 313 | › › [check] Colors | › › [check] Colors |  | same |  |
| 314 | › › [check] Labels | › › [check] Labels |  | same |  |
| 315 | › › ——— | › › ——— |  | same |  |
| 316 | › › [check] Show tile borders on the map | › › [check] Show tile borders on the map |  | same |  |
| 317 | › **MEMORY** | › **MEMORY** |  | same |  |
| 318 | › [submenu] Undo history | › [submenu] Undo history |  | same |  |
| 319 | › › [radio] 64 MB | › › [radio] 64 MB |  | same |  |
| 320 | › › [radio] 128 MB | › › [radio] 128 MB |  | same |  |
| 321 | › › [radio] 256 MB | › › [radio] 256 MB |  | same |  |
| 322 | › › [radio] 512 MB | › › [radio] 512 MB |  | same |  |
| 323 | › › [radio] 1024 MB | › › [radio] 1024 MB |  | same |  |
| 324 | › › ——— | › › ——— |  | same |  |
| 325 | › › Clear undo history now | › › Clear undo history now |  | same |  |
| 326 | › [readout] Working set | › [readout] Working set   … GB of … GB |  | same |  |
| 327 | › Clear caches… | › Clear caches… |  | same |  |
| 328 | › **APPLICATION** | › **APPLICATION** |  | same |  |
| 329 | › Storage locations… | › Storage locations… |  | same |  |
| 330 | › Assistant… | › Assistant… |  | same |  |
| 331 | › Quick bar… | › Quick bar… |  | same |  |
| 332 | › [DESKTOP ONLY] Open panels in their own window |  |  | mockup-only | n N3 |
| 333 | › [submenu] Theme | › [submenu] Theme |  | same |  |
| 334 | › › [radio] Dark | › › [radio] Dark |  | same |  |
| 335 | › › [radio] Light | › › [radio] Light |  | same |  |
| 336 | › › [radio] Follow system | › › [radio] Follow system |  | same |  |
| 337 | › [submenu] Handedness | › [submenu] Handedness |  | same |  |
| 338 | › › [radio] Left | › › [radio] Left |  | same |  |
| 339 | › › [radio] Right | › › [radio] Right |  | same |  |
| 340 |  | › [submenu] UI scale |  | Godot-only | ?  |
| 341 |  | › › [radio] 100 % |  | Godot-only (inside) | ?  |
| 342 |  | › › [radio] 125 % |  | Godot-only (inside) | ?  |
| 343 |  | › › [radio] 150 % |  | Godot-only (inside) | ?  |
| 344 |  | › › [radio] 200 % |  | Godot-only (inside) | ?  |
| 345 | › Keyboard shortcuts… | › Keyboard shortcuts… |  | same |  |
| 346 | › **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 347 | › Touch & gestures… |  |  | mockup-only | n N1 |
| 348 | [submenu] Window | [submenu] Window |  | same |  |
| 349 | › [check] Left dock | › [check] Left dock |  | same |  |
| 350 | › [check] Context panel | › [check] Context panel | › [check] Right dock | same after the milestone; today: label | m PC-4 |
| 351 | › [submenu] Context panel | › [submenu] Context panel |  | same |  |
| 352 | › › Inspect · Ctrl+1 | › › Inspect · Ctrl+1 | › › [not built] Inspect | same after the milestone; today: kind command/disabled (not built), key Ctrl+1/none | m PC-4 |
| 353 | › › Note · Ctrl+2 | › › Note · Ctrl+2 | › › [not built] Note | same after the milestone; today: kind command/disabled (not built), key Ctrl+2/none | m PC-5 |
| 354 | › › Ask (assistant) · Ctrl+3 | › › Ask (assistant) · Ctrl+3 | › › [not built] Ask (assistant) | same after the milestone; today: kind command/disabled (not built), key Ctrl+3/none | m PC-6 |
| 355 | › › ——— | › › ——— |  | same |  |
| 356 | › › Pin to the current subject | › › Pin to the current subject | › › [not built] Pin to the current subject | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 357 | › › Follow the selection | › › Follow the selection | › › [not built] Follow the selection | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 358 | › › **MODULES** | › › **MODULES** |  | same |  |
| 359 | › › [check] History | › › [not built] History |  | differs: kind check/disabled (not built) | m PC-10 |
| 360 | › › [check] Economy | › › [not built] Economy |  | differs: kind check/disabled (not built) | m PC-10 |
| 361 | › › [check] Military | › › [not built] Military |  | differs: kind check/disabled (not built) | m PC-10 |
| 362 | › › [check] Faith, culture & relations | › › [not built] Faith, culture & relations |  | differs: kind check/disabled (not built) | m PC-10 |
| 363 | › › [check] Territory | › › [not built] Territory |  | differs: kind check/disabled (not built) | m PC-10 |
| 364 | › › [check] City layout | › › [not built] City layout |  | differs: kind check/disabled (not built) | m PC-10 |
| 365 | › › [check] Here | › › [not built] Here |  | differs: kind check/disabled (not built) | m PC-10 |
| 366 | › › [check] Routes & journeys | › › [not built] Routes & journeys |  | differs: kind check/disabled (not built) | m PC-10 |
| 367 | › › [check] Links | › › [not built] Links |  | differs: kind check/disabled (not built) | m PC-10 |
| 368 | › › [check] Recent & bookmarks | › › [not built] Recent & bookmarks |  | differs: kind check/disabled (not built) | m PC-10 |
| 369 | › › ——— | › › ——— |  | same |  |
| 370 | › › Reset panel | › › Reset panel |  | same |  |
| 371 | › [check] Timeline | › [check] Timeline |  | same |  |
| 372 | › [check] Status bar | › [check] Status bar |  | same |  |
| 373 | › [check] Domain rail | › [check] Domain rail |  | same |  |
| 374 | › ——— | › ——— |  | same |  |
| 375 | › [check] Session mode · Ctrl+Shift+M | › [check] Session mode · Ctrl+Shift+M |  | same |  |
| 376 | › [check] Diagnostics overlay · Shift+D | › [check] Diagnostics overlay · Shift+D |  | same |  |
| 377 | › [check] 3D view | › [check] 3D view |  | same |  |
| 378 | › [submenu] 3D capability | › [submenu] 3D capability |  | same |  |
| 379 | › › [readout] 3D tier | › › [readout] 3D tier: not tested |  | same |  |
| 380 | › › Re-test 3D | › › Re-test 3D |  | same |  |
| 381 | › › [submenu] Force tier |  |  | mockup-only | ?  |
| 382 | › › › [radio] Automatic (as measured) |  |  | mockup-only (inside) | ?  |
| 383 | › › › [radio] Off |  |  | mockup-only (inside) | ?  |
| 384 | › › › [radio] Basic |  |  | mockup-only (inside) | ?  |
| 385 | › › › [radio] Standard |  |  | mockup-only (inside) | ?  |
| 386 | › › › [radio] Full |  |  | mockup-only (inside) | ?  |
| 387 |  | › › **FORCE TIER** |  | Godot-only | ?  |
| 388 |  | › › [radio] Automatic (as measured) |  | Godot-only | ?  |
| 389 |  | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | Godot-only | ?  |
| 390 | › ——— | › ——— |  | same |  |
| 391 | › [submenu] Workspace | › [submenu] Workspace |  | same |  |
| 392 | › › [radio] World · SM | › › [radio] World · SM |  | same |  |
| 393 | › › [radio] Civilization | › › [radio] Civilization |  | same |  |
| 394 | › › [radio] Cartography | › › [radio] Cartography |  | same |  |
| 395 | › [submenu] Open windows | › [submenu] Open windows |  | same |  |
| 396 | › › [readout] No windows open |  |  | mockup-only | s S3 |
| 397 | › › [check] Context panel | › › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 398 |  | › › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 399 | › ——— | › ——— |  | same |  |
| 400 | › Reset layout | › Reset layout |  | same |  |
| 401 | › [submenu] Layouts | › [submenu] Layouts |  | same |  |
| 402 | › › Default (reset layout) | › › Default (reset layout) |  | same |  |
| 403 | › › Generate | › › Generate |  | same |  |
| 404 | › › Sculpt | › › Sculpt |  | same |  |
| 405 | › › Cartography | › › Cartography |  | same |  |
| 406 | › › Journey | › › Journey |  | same |  |
| 407 | › › ——— | › › ——— |  | same |  |
| 408 | › › Save layout as… | › › Save layout as… |  | same |  |
| 409 | › › Forget layout… | › › Forget layout… |  | same |  |
| 410 | [submenu] Help | [submenu] Help |  | same |  |
| 411 | › [not built] Documentation | › [not built] Documentation |  | differs: reason | s S1 |
| 412 | › Gesture reference… |  |  | mockup-only | n N2 |
| 413 | › Credits & academic principles | › Credits & academic principles |  | same |  |
| 414 | › Generation info… | › Generation info… |  | same |  |
| 415 | › Save diagnostic report… | › Save diagnostic report… |  | same |  |

#### File

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | New world… · Ctrl+N · SM | New world… · Ctrl+N · SM |  | same |  |
| 2 | Open project… · Ctrl+O | Open project… · Ctrl+O |  | same |  |
| 3 | [submenu] Recent worlds | [submenu] Recent worlds |  | same |  |
| 4 | › VHAREN REACH — 129384 · 5 d ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 5 | › KESSA — 774201 · 3 w ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 6 | › THORNWOOD — 908133 · 2 mo ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 7 | New world from selection… · SM | New world from selection… · SM |  | same |  |
| 8 | ——— | ——— |  | same |  |
| 9 | Save project · Ctrl+S | Save project · Ctrl+S |  | same |  |
| 10 | Save as… · Ctrl+Shift+S | Save as… · Ctrl+Shift+S |  | same |  |
| 11 | [check] Autosave | [check] Autosave |  | same |  |
| 12 | [submenu] Autosave interval | [submenu] Autosave interval |  | same |  |
| 13 | › [radio] Off | › [radio] Off |  | same |  |
| 14 | › [radio] Every 1 min | › [radio] Every 1 min |  | same |  |
| 15 | › [radio] Every 5 min | › [radio] Every 5 min |  | same |  |
| 16 | › [radio] Every 15 min | › [radio] Every 15 min |  | same |  |
| 17 | Revert to last save | Revert to last save |  | same |  |
| 18 | Close project · Ctrl+W | Close project · Ctrl+W |  | same |  |
| 19 | ——— | ——— |  | same |  |
| 20 | [submenu] Export | [submenu] Export |  | same |  |
| 21 | › **FOR THE TABLE** | › **FOR THE TABLE** |  | same |  |
| 22 | › Player map… · Ctrl+Shift+E | › Player map… · Ctrl+Shift+E |  | same |  |
| 23 | › DM / full map… | › DM / full map… |  | same |  |
| 24 | › **SHARE** | › **SHARE** |  | same |  |
| 25 | › Export map image… | › Export map image… |  | same |  |
| 26 | › [not built] Print sheet (PDF)… | › [not built] Print sheet (PDF)… |  | same |  |
| 27 | › [not built] VTT battle map (grid + walls)… | › [not built] VTT battle map (grid + walls)… |  | same |  |
| 28 | › **DATA** | › **DATA** |  | same |  |
| 29 | › Maps   tiles | › Maps   tiles |  | same |  |
| 30 | › GIS / GeoJSON   .geojson | › GIS / GeoJSON   .geojson |  | same |  |
| 31 | › World Data   map + atlas | › World Data   map + atlas |  | same |  |
| 32 | › Assets   .zip | › Assets   .zip |  | same |  |
| 33 | **STORAGE LOCATIONS** | **STORAGE LOCATIONS** |  | same |  |
| 34 | [readout] projects | [readout] projects   … |  | same |  |
| 35 | [readout] tile atlas cache | [readout] tile atlas cache   … |  | same |  |
| 36 | [readout] asset packs | [readout] asset packs   … |  | same |  |
| 37 | [readout] exports | [readout] exports   … |  | same |  |
| 38 | Change locations… | Change locations… |  | same |  |
| 39 | Show project on disk | Show project on disk |  | same |  |
| 40 | [signpost] imports live under Records ▸ Import | [signpost] imports live under Records ▸ Import |  | same |  |
| 41 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

#### World

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Generate world · Ctrl+R · SM | Generate world · Ctrl+R · SM |  | same |  |
| 2 | ——— | ——— |  | same |  |
| 3 | Reset generation parameters · SM | Reset generation parameters · SM |  | same |  |
| 4 | [submenu] Reset one stage · SM | [submenu] Reset one stage · SM |  | same |  |
| 5 | › 01 Planet · SM | › 01 Planet |  | same |  |
| 6 | › 02 Extent & scale · SM | › 02 Extent & scale |  | same |  |
| 7 | › 03 World structure · SM | › 03 World structure |  | same |  |
| 8 | › 04 Tectonics · SM | › 04 Tectonics |  | same |  |
| 9 | › 05 Volcanism & impacts · SM | › 05 Volcanism & impacts |  | same |  |
| 10 | › 06 Erosion · SM | › 06 Erosion |  | same |  |
| 11 | › 07 Hydrology · SM | › 07 Hydrology |  | same |  |
| 12 | › 08 Climate · SM | › 08 Climate |  | same |  |
| 13 | › 09 Ecology & biomes · SM | › 09 Ecology & biomes |  | same |  |
| 14 | › 10 Resources & soils · SM | › 10 Resources & soils |  | same |  |
| 15 | ——— | ——— |  | same |  |
| 16 | [radio] Generation pipeline · SM | [radio] Generation pipeline · SM |  | same |  |
| 17 | [radio] Sculpt · SM | [radio] Sculpt · SM |  | same |  |
| 18 | ——— | ——— |  | same |  |
| 19 | Seed & size… · SM | Seed & size… · SM |  | same |  |

#### Records

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 2 | **RECORDS** | **RECORDS** |  | same |  |
| 3 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 4 | Places… | Places… |  | same |  |
| 5 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 6 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 7 | Places without notes… | Places without notes… |  | same |  |
| 8 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 9 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 10 | World data tables… | World data tables… |  | same |  |
| 11 | [submenu] Import | [submenu] Import |  | same |  |
| 12 | › Maps   tiles | › Maps   tiles |  | same |  |
| 13 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 14 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 15 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 16 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 17 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 18 | › **SOURCES** | › **SOURCES** |  | same |  |
| 19 | › External Sources | › External Sources |  | same |  |
| 20 | › Connected Sources | › Connected Sources |  | same |  |
| 21 | › Source Registry | › Source Registry |  | same |  |
| 22 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 23 | › Definitions | › Definitions |  | same |  |
| 24 | › Check Data | › Check Data |  | same |  |
| 25 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 26 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

#### Story

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **TIME** | **TIME** |  | same |  |
| 2 | Timeline · Ctrl+Shift+T | Timeline · Ctrl+Shift+T |  | same |  |
| 3 | Political history of the selection | Political history of the selection | Political history of this place… | same after the milestone; today: label | m PC-4 |
| 4 | Faction history… | Faction history… |  | same |  |
| 5 | **JOURNEYS** | **JOURNEYS** |  | same |  |
| 6 | Journey planner | Journey planner |  | same |  |
| 7 | Saved journeys… | Saved journeys… |  | same |  |
| 8 | Travel library… · Shift+L | Travel library… · Shift+L |  | same |  |
| 9 | **CONFLICTS & CAMPAIGNS** | **CONFLICTS & CAMPAIGNS** |  | same |  |
| 10 | Conflicts… | Conflicts… |  | same |  |
| 11 | Draw a conflict | Draw a conflict |  | same |  |
| 12 | [readout] Campaigns this year | [readout] Campaigns this year |  | same |  |
| 13 | **NOTES** | **NOTES** |  | same |  |
| 14 | Note for the selection · Ctrl+Shift+N | Note for the selection · Ctrl+Shift+N | Notes for this place · Ctrl+Shift+N | same after the milestone; today: label | m PC-5 |
| 15 | [signpost] Places without notes… is in Records | [signpost] Places without notes is in Records |  | same |  |
| 16 | Ask the assistant about the selection · Ctrl+J | Ask the assistant about the selection · Ctrl+J | Assistant panel… | same after the milestone; today: label, key Ctrl+J/none | m PC-6 |

### Phone, MENU

#### MENU root

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [quick] QUICK (pinned commands) | [quick] QUICK (pinned commands) |  | same |  |
| 2 | **MENUS** | **MENUS** |  | same |  |
| 3 | [submenu] File | [submenu] File |  | same |  |
| 4 | [submenu] Edit | [submenu] Edit |  | same |  |
| 5 | [submenu] Assets | [submenu] Assets |  | same |  |
| 6 | [submenu] Records | [submenu] Records |  | same |  |
| 7 | [submenu] Story | [submenu] Story |  | same |  |
| 8 | [submenu] Preferences | [submenu] Preferences |  | same |  |
| 9 | [submenu] Window | [submenu] Window |  | same |  |
| 10 | [submenu] Help | [submenu] Help |  | same |  |
| 11 | [status] STATUS (world, state, last autosave) | [status] STATUS (world, state, last autosave) |  | same |  |

#### File

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | New world… · SM | New world… · SM |  | same |  |
| 2 | Generate world · SM | Generate world · SM |  | same |  |
| 3 | Open project… | Open project… |  | same |  |
| 4 | [submenu] Recent worlds | [submenu] Recent worlds |  | same |  |
| 5 | › VHAREN REACH — 129384 · 5 d ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 6 | › KESSA — 774201 · 3 w ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 7 | › THORNWOOD — 908133 · 2 mo ago | › {meta.get('name', '')} · run-time list |  | same (run-time list) |  |
| 8 | New world from selection… · SM | New world from selection… · SM |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Save project | Save project |  | same |  |
| 11 | Save as… | Save as… |  | same |  |
| 12 | [check] Autosave | [check] Autosave |  | same |  |
| 13 | [submenu] Autosave interval | [submenu] Autosave interval |  | same |  |
| 14 | › [radio] Off | › [radio] Off |  | same |  |
| 15 | › [radio] Every 1 min | › [radio] Every 1 min |  | same |  |
| 16 | › [radio] Every 5 min | › [radio] Every 5 min |  | same |  |
| 17 | › [radio] Every 15 min | › [radio] Every 15 min |  | same |  |
| 18 | Revert to last save | Revert to last save |  | same |  |
| 19 | Close project | Close project |  | same |  |
| 20 | ——— | ——— |  | same |  |
| 21 | [submenu] Export | [submenu] Export |  | same |  |
| 22 | › **FOR THE TABLE** | › **FOR THE TABLE** |  | same |  |
| 23 | › Player map… | › Player map… |  | same |  |
| 24 | › DM / full map… | › DM / full map… |  | same |  |
| 25 | › **SHARE** | › **SHARE** |  | same |  |
| 26 | › Export map image… | › Export map image… |  | same |  |
| 27 | › [not built] Print sheet (PDF)… | › [not built] Print sheet (PDF)… |  | same |  |
| 28 | › [not built] VTT battle map (grid + walls)… | › [not built] VTT battle map (grid + walls)… |  | same |  |
| 29 | › **DATA** | › **DATA** |  | same |  |
| 30 | › Maps   tiles | › Maps   tiles |  | same |  |
| 31 | › GIS / GeoJSON   .geojson | › GIS / GeoJSON   .geojson |  | same |  |
| 32 | › World Data   map + atlas | › World Data   map + atlas |  | same |  |
| 33 | › Assets   .zip | › Assets   .zip |  | same |  |
| 34 | **STORAGE LOCATIONS** | **STORAGE LOCATIONS** |  | same |  |
| 35 | [readout] projects | [readout] projects   … |  | same |  |
| 36 | [readout] tile atlas cache | [readout] tile atlas cache   … |  | same |  |
| 37 | [readout] asset packs | [readout] asset packs   … |  | same |  |
| 38 | [readout] exports | [readout] exports   … |  | same |  |
| 39 | Change locations… | Change locations… |  | same |  |
| 40 | Show project on disk | Show project on disk |  | same |  |
| 41 | [signpost] imports live under Records ▸ Import | [signpost] imports live under Records ▸ Import |  | same |  |
| 42 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

#### Edit

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Undo | Undo |  | same |  |
| 2 | Redo | Redo |  | same |  |
| 3 | Undo history… | Undo history… |  | same |  |
| 4 | ——— | ——— |  | same |  |
| 5 | Cut | Cut |  | same |  |
| 6 | Copy | Copy |  | same |  |
| 7 | Paste | Paste |  | same |  |
| 8 | Delete | Delete |  | same |  |
| 9 | ——— | ——— |  | same |  |
| 10 | Select all | Select all |  | same |  |
| 11 | Deselect | Deselect |  | same |  |
| 12 | Previous selection | Previous selection |  | same |  |
| 13 | Next selection | Next selection |  | same |  |
| 14 | ——— | ——— |  | same |  |
| 15 | Find on map… | Find on map… |  | same |  |
| 16 | ——— | ——— |  | same |  |
| 17 | Reset generation parameters · SM | Reset generation parameters · SM |  | same |  |
| 18 | [submenu] Reset one stage · SM | [submenu] Reset one stage · SM |  | same |  |
| 19 | › 01 Planet · SM | › 01 Planet |  | same |  |
| 20 | › 02 Extent & scale · SM | › 02 Extent & scale |  | same |  |
| 21 | › 03 World structure · SM | › 03 World structure |  | same |  |
| 22 | › 04 Tectonics · SM | › 04 Tectonics |  | same |  |
| 23 | › 05 Volcanism & impacts · SM | › 05 Volcanism & impacts |  | same |  |
| 24 | › 06 Erosion · SM | › 06 Erosion |  | same |  |
| 25 | › 07 Hydrology · SM | › 07 Hydrology |  | same |  |
| 26 | › 08 Climate · SM | › 08 Climate |  | same |  |
| 27 | › 09 Ecology & biomes · SM | › 09 Ecology & biomes |  | same |  |
| 28 | › 10 Resources & soils · SM | › 10 Resources & soils |  | same |  |

#### Assets

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Asset library | ⧉ Asset library |  | same |  |
| 2 | ⧉ Sprite sheet slicer (▦) | ⧉ Sprite sheet slicer (▦) |  | same |  |
| 3 | Import asset pack .zip… | Import asset pack .zip… |  | same |  |
| 4 | Open pack in library… | Open pack in library… |  | same |  |
| 5 | Pack report… | Pack report… |  | same |  |
| 6 | Import folder of art… | Import folder of art… |  | same |  |
| 7 | Compare pack with library… | Compare pack with library… |  | same |  |
| 8 | ——— | ——— |  | same |  |
| 9 | [submenu] Asset pack | [submenu] Asset pack |  | same |  |
| 10 | › **ACTIVE PACK** | › **ACTIVE PACK** |  | same |  |
| 11 | › [readout] NAME | › [readout] NAME   — loading — |  | same |  |
| 12 | › [readout] AUTHOR | › [readout] AUTHOR   — loading — |  | same |  |
| 13 | › [readout] LICENSE | › [readout] LICENSE   — loading — |  | same |  |
| 14 | › [readout] SCHEMA | › [readout] SCHEMA   — loading — |  | same |  |
| 15 | › [readout] FILLED | › [readout] FILLED   — loading — |  | same |  |
| 16 | › Pack metadata… | › Pack metadata… |  | same |  |
| 17 | › Validate pack | › Validate pack |  | same |  |
| 18 | › Export pack .zip… | › Export pack .zip… |  | same |  |
| 19 | [submenu] Icon families | [submenu] Icon families |  | same |  |
| 20 | › Feature icons   22/31 | › Feature icons |  | same |  |
| 21 | › Settlement pins   8/10 | › Settlement pins |  | same |  |
| 22 | › Settlement traits   5/12 | › Settlement traits |  | same |  |
| 23 | › Points of interest   14/21 | › Points of interest |  | same |  |
| 24 | › Sea marks   4/9 | › Sea marks |  | same |  |
| 25 | › Custom icons   0/6 | › Custom icons |  | same |  |
| 26 | [submenu] Texture sets | [submenu] Texture sets |  | same |  |
| 27 | › Splat channels   4/4 | › Splat channels |  | same |  |
| 28 | › Biome textures   9/12 | › Biome textures |  | same |  |
| 29 | › Terrain textures   6/8 | › Terrain textures |  | same |  |
| 30 | [submenu] Landmark types | Landmark types… |  | differs: kind submenu/command | (c) EH-2 |
| 31 | › [submenu] Physical |  |  | mockup-only | (c) EH-2 |
| 32 | › › Peak |  |  | mockup-only (inside) | (c) EH-2 |
| 33 | › › Ridge |  |  | mockup-only (inside) | (c) EH-2 |
| 34 | › › Saddle   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 35 | › › Cliff |  |  | mockup-only (inside) | (c) EH-2 |
| 36 | › › Gorge |  |  | mockup-only (inside) | (c) EH-2 |
| 37 | › › Cave |  |  | mockup-only (inside) | (c) EH-2 |
| 38 | › › Waterfall |  |  | mockup-only (inside) | (c) EH-2 |
| 39 | › › Spring |  |  | mockup-only (inside) | (c) EH-2 |
| 40 | › › Lake |  |  | mockup-only (inside) | (c) EH-2 |
| 41 | › › Delta   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 42 | › › River confluence |  |  | mockup-only (inside) | (c) EH-2 |
| 43 | › › Volcanic feature |  |  | mockup-only (inside) | (c) EH-2 |
| 44 | › › Rock formation |  |  | mockup-only (inside) | (c) EH-2 |
| 45 | › › Glacial feature   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 46 | › › Ancient forest   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 47 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 48 | › › Open Physical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 49 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 50 | › [submenu] Transport |  |  | mockup-only | (c) EH-2 |
| 51 | › › Mountain pass |  |  | mockup-only (inside) | (c) EH-2 |
| 52 | › › River crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 53 | › › Ford |  |  | mockup-only (inside) | (c) EH-2 |
| 54 | › › Bridge site |  |  | mockup-only (inside) | (c) EH-2 |
| 55 | › › Road junction |  |  | mockup-only (inside) | (c) EH-2 |
| 56 | › › Caravan station |  |  | mockup-only (inside) | (c) EH-2 |
| 57 | › › Portage |  |  | mockup-only (inside) | (c) EH-2 |
| 58 | › › Harbour |  |  | mockup-only (inside) | (c) EH-2 |
| 59 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 60 | › › Open Transport in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 61 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 62 | › [submenu] Economic |  |  | mockup-only | (c) EH-2 |
| 63 | › › Mine |  |  | mockup-only (inside) | (c) EH-2 |
| 64 | › › Quarry |  |  | mockup-only (inside) | (c) EH-2 |
| 65 | › › Salt works   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 66 | › › Resource extraction site |  |  | mockup-only (inside) | (c) EH-2 |
| 67 | › › Market site |  |  | mockup-only (inside) | (c) EH-2 |
| 68 | › › Trade depot |  |  | mockup-only (inside) | (c) EH-2 |
| 69 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 70 | › › Open Economic in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 71 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 72 | › [submenu] Military |  |  | mockup-only | (c) EH-2 |
| 73 | › › Fort |  |  | mockup-only (inside) | (c) EH-2 |
| 74 | › › Watchtower |  |  | mockup-only (inside) | (c) EH-2 |
| 75 | › › Fortified pass |  |  | mockup-only (inside) | (c) EH-2 |
| 76 | › › Fortified crossing |  |  | mockup-only (inside) | (c) EH-2 |
| 77 | › › Battlefield |  |  | mockup-only (inside) | (c) EH-2 |
| 78 | › › Border marker |  |  | mockup-only (inside) | (c) EH-2 |
| 79 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 80 | › › Open Military in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 81 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 82 | › [submenu] Cultural |  |  | mockup-only | (c) EH-2 |
| 83 | › › Shrine   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 84 | › › Temple   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 85 | › › Sacred grove   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 86 | › › Sacred mountain   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 87 | › › Pilgrimage site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 88 | › › Tomb   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 89 | › › Monument   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 90 | › › Ceremonial site   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 91 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 92 | › › Open Cultural in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 93 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 94 | › [submenu] Historical |  |  | mockup-only | (c) EH-2 |
| 95 | › › Ruin   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 96 | › › Abandoned settlement |  |  | mockup-only (inside) | (c) EH-2 |
| 97 | › › Ancient road |  |  | mockup-only (inside) | (c) EH-2 |
| 98 | › › Historic battlefield   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 99 | › › Destroyed fortress   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 100 | › › Historic crossing   not buildable |  |  | mockup-only (inside) | (c) EH-2 |
| 101 | › › ——— |  |  | mockup-only (inside) | (c) EH-2 |
| 102 | › › Open Historical in the dock |  |  | mockup-only (inside) | (c) EH-2 |
| 103 | › › [signpost] Rows here read state; they do not arm a type. |  |  | mockup-only (inside) | (c) EH-2 |
| 104 | › Placed by hand   3 |  |  | mockup-only | (c) EH-2 |
| 105 | › ——— |  |  | mockup-only | (c) EH-2 |
| 106 | › Landmark icons… |  |  | mockup-only | (c) EH-2 |
| 107 | › Landmark label style… |  |  | mockup-only | (c) EH-2 |
| 108 | ——— | ——— |  | same |  |
| 109 | Apply library to map | Apply library to map |  | same |  |
| 110 | Clear library… | Clear library…   destructive |  | same |  |

#### Records

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 2 | **RECORDS** | **RECORDS** |  | same |  |
| 3 | Factions… | Factions… |  | same |  |
| 4 | Places… | Places… |  | same |  |
| 5 | Markdown vault… | Markdown vault… |  | same |  |
| 6 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 7 | Places without notes… | Places without notes… |  | same |  |
| 8 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 9 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 10 | World data tables… | World data tables… |  | same |  |
| 11 | [submenu] Import | [submenu] Import |  | same |  |
| 12 | › Maps   tiles | › Maps   tiles |  | same |  |
| 13 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 14 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 15 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 16 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 17 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 18 | › **SOURCES** | › **SOURCES** |  | same |  |
| 19 | › External Sources | › External Sources |  | same |  |
| 20 | › Connected Sources | › Connected Sources |  | same |  |
| 21 | › Source Registry | › Source Registry |  | same |  |
| 22 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 23 | › Definitions | › Definitions |  | same |  |
| 24 | › Check Data | › Check Data |  | same |  |
| 25 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 26 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

#### Story

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **TIME** | **TIME** |  | same |  |
| 2 | Timeline | Timeline |  | same |  |
| 3 | Political history of the selection | Political history of the selection | Political history of this place… | same after the milestone; today: label | m PC-4 |
| 4 | Faction history… | Faction history… |  | same |  |
| 5 | **JOURNEYS** | **JOURNEYS** |  | same |  |
| 6 | Journey planner | Journey planner |  | same |  |
| 7 | Saved journeys… | Saved journeys… |  | same |  |
| 8 | Travel library… | Travel library… |  | same |  |
| 9 | **CONFLICTS & CAMPAIGNS** | **CONFLICTS & CAMPAIGNS** |  | same |  |
| 10 | Conflicts… | Conflicts… |  | same |  |
| 11 | Draw a conflict | Draw a conflict |  | same |  |
| 12 | [readout] Campaigns this year | [readout] Campaigns this year |  | same |  |
| 13 | **NOTES** | **NOTES** |  | same |  |
| 14 | Note for the selection | Note for the selection | Notes for this place | same after the milestone; today: label | m PC-5 |
| 15 | [signpost] Places without notes… is in Records | [signpost] Places without notes is in Records |  | same |  |
| 16 | Ask the assistant about the selection | Ask the assistant about the selection | Assistant panel… | same after the milestone; today: label | m PC-6 |

#### Preferences

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | **PERFORMANCE** | **PERFORMANCE** |  | same |  |
| 2 | [check] GPU acceleration | [check] GPU acceleration |  | same |  |
| 3 | [submenu] Devices | [submenu] Devices |  | same |  |
| 4 | › [check] Automatic (highest-performance GPU) | › [check] Automatic (highest-performance GPU) |  | same |  |
| 5 | › ——— | › ——— |  | same |  |
| 6 | › [check] Adreno 630 · integrated · Vulkan | › [check] … · … · … · run-time list |  | same (run-time list) |  |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › [readout] Memory | › [readout] Memory: not measured yet |  | same |  |
| 9 | › ——— | › ——— |  | same |  |
| 10 | › Rescan devices… | › Rescan devices… |  | same |  |
| 11 | [submenu] Multi-GPU mode | [submenu] Multi-GPU mode |  | same |  |
| 12 | › [radio] Single device | › [radio] Single device |  | same |  |
| 13 | › [radio] Split tiles | › [radio] Split tiles |  | same |  |
| 14 | › [not built] Alternate frames | › [radio] Alternate frames · dark |  | same |  |
| 15 | [submenu] CPU worker threads | [submenu] CPU worker threads |  | same |  |
| 16 | › [radio] Automatic | › [radio] Automatic |  | same |  |
| 17 | › [radio] 1 — 1 thread | › [radio] 1 — 1 thread |  | same |  |
| 18 | › [radio] 2 — quarter | › [radio] 2 — quarter |  | same |  |
| 19 | › [radio] 4 — half | › [radio] 4 — half |  | same |  |
| 20 | › [radio] 8 — all cores | › [radio] 8 — all cores |  | same |  |
| 21 | › [readout] Running | › [readout] Running … workers now |  | same |  |
| 22 | [submenu] VRAM budget | [submenu] VRAM budget |  | same |  |
| 23 | › [radio] No cap | › [radio] No cap |  | same |  |
| 24 | › [radio] 1 GB | › [radio] 1 GB |  | same |  |
| 25 | › [radio] 2 GB | › [radio] 2 GB |  | same |  |
| 26 | › [radio] 4 GB | › [radio] 4 GB |  | same |  |
| 27 | › [radio] 8 GB | › [radio] 8 GB |  | same |  |
| 28 | › [radio] 12 GB | › [radio] 12 GB |  | same |  |
| 29 | › [radio] 16 GB | › [radio] 16 GB |  | same |  |
| 30 | › [radio] 24 GB | › [radio] 24 GB |  | same |  |
| 31 | › [readout] Current grid 2048×2048 needs about | › [readout] Current grid …x… needs about … MB |  | same |  |
| 32 | [submenu] Fallback when VRAM full | [submenu] Fallback when VRAM full |  | same |  |
| 33 | › [radio] CPU tile pass | › [radio] CPU tile pass |  | same |  |
| 34 | › [not built] Reduce working res | › [radio] Reduce working res · dark |  | same |  |
| 35 | › [radio] Fail with error | › [radio] Fail with error |  | same |  |
| 36 | Try the GPU again · dark | Try the GPU again · dark |  | same |  |
| 37 | **GRAPHICS** | **GRAPHICS** |  | same |  |
| 38 | [submenu] Render quality | [submenu] Render quality |  | same |  |
| 39 | › [radio] performance | › [radio] performance |  | same |  |
| 40 | › [radio] balanced | › [radio] balanced |  | same |  |
| 41 | › [radio] quality | › [radio] quality |  | same |  |
| 42 | › [radio] ultra | › [radio] ultra |  | same |  |
| 43 | [not built] Anti-aliasing · anisotropy | [not built] Anti-aliasing · anisotropy |  | differs: reason | s S1 |
| 44 | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management |  | same |  |
| 45 | [not built] 3D viewport defaults | [not built] 3D viewport defaults |  | differs: reason | s S1 |
| 46 | [submenu] Lighting rig defaults | [submenu] Lighting rig defaults |  | same |  |
| 47 | › [readout] Following the engine | › [readout] Following the engine |  | same |  |
| 48 | › [submenu] Azimuth | › **AZIMUTH** |  | differs: kind submenu/caption | ?  |
| 49 | › › [radio] 0 deg |  |  | mockup-only | ?  |
| 50 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 51 | › › [radio] 90 deg |  |  | mockup-only | ?  |
| 52 | › › [radio] 135 deg |  |  | mockup-only | ?  |
| 53 | › › [radio] 180 deg |  |  | mockup-only | ?  |
| 54 | › › [radio] 225 deg |  |  | mockup-only | ?  |
| 55 | › › [radio] 270 deg |  |  | mockup-only | ?  |
| 56 | › › [radio] 315 deg |  |  | mockup-only | ?  |
| 57 |  | › [radio] 0 deg |  | Godot-only | ?  |
| 58 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 59 |  | › [radio] 90 deg |  | Godot-only | ?  |
| 60 |  | › [radio] 135 deg |  | Godot-only | ?  |
| 61 |  | › [radio] 180 deg |  | Godot-only | ?  |
| 62 |  | › [radio] 225 deg |  | Godot-only | ?  |
| 63 |  | › [radio] 270 deg |  | Godot-only | ?  |
| 64 |  | › [radio] 315 deg |  | Godot-only | ?  |
| 65 | › [submenu] Elevation | › **ELEVATION** |  | differs: kind submenu/caption | ?  |
| 66 | › › [radio] 15 deg |  |  | mockup-only | ?  |
| 67 | › › [radio] 25 deg |  |  | mockup-only | ?  |
| 68 | › › [radio] 35 deg |  |  | mockup-only | ?  |
| 69 | › › [radio] 45 deg |  |  | mockup-only | ?  |
| 70 | › › [radio] 55 deg |  |  | mockup-only | ?  |
| 71 | › › [radio] 65 deg |  |  | mockup-only | ?  |
| 72 | › › [radio] 75 deg |  |  | mockup-only | ?  |
| 73 |  | › [radio] 15 deg |  | Godot-only | ?  |
| 74 |  | › [radio] 25 deg |  | Godot-only | ?  |
| 75 |  | › [radio] 35 deg |  | Godot-only | ?  |
| 76 |  | › [radio] 45 deg |  | Godot-only | ?  |
| 77 |  | › [radio] 55 deg |  | Godot-only | ?  |
| 78 |  | › [radio] 65 deg |  | Godot-only | ?  |
| 79 |  | › [radio] 75 deg |  | Godot-only | ?  |
| 80 | › [submenu] Ambient | › **AMBIENT** |  | differs: kind submenu/caption | ?  |
| 81 | › › [radio] 0.00 |  |  | mockup-only | ?  |
| 82 | › › [radio] 0.15 |  |  | mockup-only | ?  |
| 83 | › › [radio] 0.25 |  |  | mockup-only | ?  |
| 84 | › › [radio] 0.35 |  |  | mockup-only | ?  |
| 85 | › › [radio] 0.50 |  |  | mockup-only | ?  |
| 86 | › › [radio] 0.70 |  |  | mockup-only | ?  |
| 87 |  | › [radio] 0.00 |  | Godot-only | ?  |
| 88 |  | › [radio] 0.15 |  | Godot-only | ?  |
| 89 |  | › [radio] 0.25 |  | Godot-only | ?  |
| 90 |  | › [radio] 0.35 |  | Godot-only | ?  |
| 91 |  | › [radio] 0.50 |  | Godot-only | ?  |
| 92 |  | › [radio] 0.70 |  | Godot-only | ?  |
| 93 | › [submenu] Multidirectional | › **MULTIDIRECTIONAL** |  | differs: kind submenu/caption | ?  |
| 94 | › › [radio] 1 light (single sun) |  |  | mockup-only | ?  |
| 95 | › › [radio] 2 lights |  |  | mockup-only | ?  |
| 96 | › › [radio] 4 lights |  |  | mockup-only | ?  |
| 97 | › › [radio] 6 lights |  |  | mockup-only | ?  |
| 98 | › › [radio] 8 lights |  |  | mockup-only | ?  |
| 99 | › › [radio] 12 lights |  |  | mockup-only | ?  |
| 100 |  | › [radio] 1 light (single sun) |  | Godot-only | ?  |
| 101 |  | › [radio] 2 lights |  | Godot-only | ?  |
| 102 |  | › [radio] 4 lights |  | Godot-only | ?  |
| 103 |  | › [radio] 6 lights |  | Godot-only | ?  |
| 104 |  | › [radio] 8 lights |  | Godot-only | ?  |
| 105 |  | › [radio] 12 lights |  | Godot-only | ?  |
| 106 | › ——— | › ——— |  | same |  |
| 107 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 108 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 109 | › [radio] 1x | › [radio] 1x |  | same |  |
| 110 | › [radio] 2x | › [radio] 2x |  | same |  |
| 111 | › [radio] 4x | › [radio] 4x |  | same |  |
| 112 | › ——— | › ——— |  | same |  |
| 113 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 114 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 115 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 116 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 117 | › [radio] Manual | › [radio] Manual |  | same |  |
| 118 | › ——— | › ——— |  | same |  |
| 119 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 120 | › Leave deep detail | › Leave deep detail |  | same |  |
| 121 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 122 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 123 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 124 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 125 | › ——— |  |  | mockup-only | ?  |
| 126 | › [submenu] LOD levels | › **LOD LEVELS** |  | differs: kind submenu/caption | ?  |
| 127 | › › [radio] LOD 0–0   1 tile |  |  | mockup-only | ?  |
| 128 | › › [radio] LOD 0–1   5 tiles |  |  | mockup-only | ?  |
| 129 | › › [radio] LOD 0–2   21 tiles |  |  | mockup-only | ?  |
| 130 | › › [radio] LOD 0–3   85 tiles |  |  | mockup-only | ?  |
| 131 | › › [radio] LOD 0–4   341 tiles |  |  | mockup-only | ?  |
| 132 | › › [radio] LOD 0–5   1 365 tiles |  |  | mockup-only | ?  |
| 133 | › › [radio] LOD 0–6   5 461 tiles |  |  | mockup-only | ?  |
| 134 | › › [radio] LOD 0–7   21 845 tiles |  |  | mockup-only | ?  |
| 135 | › › [radio] LOD 0–8   87 381 tiles |  |  | mockup-only | ?  |
| 136 |  | › [radio] LOD 0–0   1 tile |  | Godot-only | ?  |
| 137 |  | › [radio] LOD 0–1   5 tiles |  | Godot-only | ?  |
| 138 |  | › [radio] LOD 0–2   21 tiles |  | Godot-only | ?  |
| 139 |  | › [radio] LOD 0–3   85 tiles |  | Godot-only | ?  |
| 140 |  | › [radio] LOD 0–4   341 tiles |  | Godot-only | ?  |
| 141 |  | › [radio] LOD 0–5   1 365 tiles |  | Godot-only | ?  |
| 142 |  | › [radio] LOD 0–6   5 461 tiles |  | Godot-only | ?  |
| 143 |  | › [radio] LOD 0–7   21 845 tiles |  | Godot-only | ?  |
| 144 |  | › [radio] LOD 0–8   87 381 tiles |  | Godot-only | ?  |
| 145 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 146 | › [readout] 85 chunks · 1.2 GB · to LOD 3 |  |  | mockup-only | ?  |
| 147 |  | › [readout] … chunks · … · empty |  | Godot-only | ?  |
| 148 | › [submenu] Size cap · GB | › **SIZE CAP · GB** |  | differs: kind submenu/caption | ?  |
| 149 | › › [radio] No cap |  |  | mockup-only | ?  |
| 150 | › › [radio] 0.5 GB |  |  | mockup-only | ?  |
| 151 | › › [radio] 1 GB |  |  | mockup-only | ?  |
| 152 | › › [radio] 2 GB |  |  | mockup-only | ?  |
| 153 | › › [radio] 4 GB |  |  | mockup-only | ?  |
| 154 | › › [radio] 8 GB |  |  | mockup-only | ?  |
| 155 | › › [radio] 16 GB |  |  | mockup-only | ?  |
| 156 |  | › [radio] No cap |  | Godot-only | ?  |
| 157 |  | › [radio] 0.5 GB |  | Godot-only | ?  |
| 158 |  | › [radio] 1 GB |  | Godot-only | ?  |
| 159 |  | › [radio] 2 GB |  | Godot-only | ?  |
| 160 |  | › [radio] 4 GB |  | Godot-only | ?  |
| 161 |  | › [radio] 8 GB |  | Godot-only | ?  |
| 162 |  | › [radio] 16 GB |  | Godot-only | ?  |
| 163 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 164 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 165 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 166 | › ——— | › ——— |  | same |  |
| 167 | › Export atlas… | › Export atlas… |  | same |  |
| 168 | › Import atlas… | › Import atlas… |  | same |  |
| 169 | › ——— | › ——— |  | same |  |
| 170 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 171 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 172 | › [check] Grid | › [check] Grid |  | same |  |
| 173 | › [check] Colors | › [check] Colors |  | same |  |
| 174 | › [check] Labels | › [check] Labels |  | same |  |
| 175 | › ——— | › ——— |  | same |  |
| 176 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 177 | **MEMORY** | **MEMORY** |  | same |  |
| 178 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 179 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 180 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 181 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 182 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 183 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 184 | › ——— | › ——— |  | same |  |
| 185 | › Clear undo history now | › Clear undo history now |  | same |  |
| 186 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 187 | Clear caches… | Clear caches… |  | same |  |
| 188 | **APPLICATION** | **APPLICATION** |  | same |  |
| 189 | Storage locations… | Storage locations… |  | same |  |
| 190 | Assistant… | Assistant… |  | same |  |
| 191 | Quick bar… | Quick bar… |  | same |  |
| 192 | [submenu] Theme | [submenu] Theme |  | same |  |
| 193 | › [radio] Dark | › [radio] Dark |  | same |  |
| 194 | › [radio] Light | › [radio] Light |  | same |  |
| 195 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 196 | [submenu] Units | [submenu] Units |  | same |  |
| 197 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 198 | › [radio] Miles | › [radio] Miles |  | same |  |
| 199 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 200 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 201 | › [radio] Left | › [radio] Left |  | same |  |
| 202 | › [radio] Right | › [radio] Right |  | same |  |
| 203 |  | UI scale · dark |  | Godot-only | ?  |
| 204 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |
| 205 | **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 206 | Touch & gestures… |  |  | mockup-only | n N1 |

#### Window

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [check] Left dock | [check] Left dock |  | same |  |
| 2 | [check] Context panel | [check] Context panel | [check] Right dock | same after the milestone; today: label | m PC-4 |
| 3 | [submenu] Context panel | [submenu] Context panel |  | same |  |
| 4 | › Inspect | › Inspect | › [not built] Inspect | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 5 | › Note | › Note | › [not built] Note | same after the milestone; today: kind command/disabled (not built) | m PC-5 |
| 6 | › Ask (assistant) | › Ask (assistant) | › [not built] Ask (assistant) | same after the milestone; today: kind command/disabled (not built) | m PC-6 |
| 7 | › ——— | › ——— |  | same |  |
| 8 | › Pin to the current subject | › Pin to the current subject | › [not built] Pin to the current subject | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 9 | › Follow the selection | › Follow the selection | › [not built] Follow the selection | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 10 | [check] Timeline | [check] Timeline |  | same |  |
| 11 | [check] Status bar | [check] Status bar |  | same |  |
| 12 | [check] Domain rail | [check] Domain rail |  | same |  |
| 13 | ——— | ——— |  | same |  |
| 14 | [check] Session mode | [check] Session mode |  | same |  |
| 15 | [check] Diagnostics overlay | [check] Diagnostics overlay |  | same |  |
| 16 | [check] 3D view | [check] 3D view |  | same |  |
| 17 | [submenu] 3D capability | [submenu] 3D capability |  | same |  |
| 18 | › [readout] 3D tier | › [readout] 3D tier: not tested |  | same |  |
| 19 | › Re-test 3D | › Re-test 3D |  | same |  |
| 20 | › [submenu] Force tier |  |  | mockup-only | ?  |
| 21 | › › [radio] Automatic (as measured) |  |  | mockup-only (inside) | ?  |
| 22 | › › [radio] Off |  |  | mockup-only (inside) | ?  |
| 23 | › › [radio] Basic |  |  | mockup-only (inside) | ?  |
| 24 | › › [radio] Standard |  |  | mockup-only (inside) | ?  |
| 25 | › › [radio] Full |  |  | mockup-only (inside) | ?  |
| 26 |  | › **FORCE TIER** |  | Godot-only | ?  |
| 27 |  | › [radio] Automatic (as measured) |  | Godot-only | ?  |
| 28 |  | › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | Godot-only | ?  |
| 29 | ——— | ——— |  | same |  |
| 30 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 31 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 32 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 33 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 34 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 35 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 36 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 37 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 38 | ——— | ——— |  | same |  |
| 39 | Reset layout | Reset layout |  | same |  |
| 40 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 41 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 42 | › Generate | › Generate |  | same |  |
| 43 | › Sculpt | › Sculpt |  | same |  |
| 44 | › Cartography | › Cartography |  | same |  |
| 45 | › Journey | › Journey |  | same |  |
| 46 | › ——— | › ——— |  | same |  |
| 47 | › Save layout as… | › Save layout as… |  | same |  |
| 48 | › Forget layout… | › Forget layout… |  | same |  |

#### Help

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [not built] Documentation | [not built] Documentation |  | differs: reason | s S1 |
| 2 | Keyboard reference… | Keyboard reference… |  | same |  |
| 3 | Gesture reference… | Gesture reference… |  | same |  |
| 4 | Credits & academic principles | Credits & academic principles |  | same |  |
| 5 | Generation info… | Generation info… |  | same |  |
| 6 | Save diagnostic report… | Save diagnostic report… |  | same |  |
| 7 | About | About |  | same |  |
