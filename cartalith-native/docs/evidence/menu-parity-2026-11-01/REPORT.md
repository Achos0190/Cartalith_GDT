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
- **Godot:** `godot_menus.json`, written by `godot_menus.py`. It covers the shipped shell at `d2363949`, with these
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
| Desktop | 447 | 442 | 448 | 401 | 12 | 33 | 1 | 1 | 0 | 0 | 23 | 24 | 0 |
| Tablet, 1 180 dp and wider | 450 | 441 | 451 | 400 | 12 | 33 | 5 | 1 | 0 | 0 | 23 | 24 | 4 |
| Tablet, below 1 180 dp | 464 | 455 | 465 | 414 | 12 | 33 | 5 | 1 | 0 | 0 | 23 | 24 | 4 |
| Phone, MENU | 445 | 438 | 446 | 408 | 12 | 22 | 3 | 1 | 0 | 0 | 12 | 24 | 2 |

Rows per rule, per platform:

| Rule | Desktop | Tablet, 1 180 dp and wider | Tablet, below 1 180 dp | Phone, MENU |
|---|---:|---:|---:|---:|
| R1 | 18 | 18 | 18 | 18 |
| R2 | 1 | 1 | 1 | 1 |
| S1 | 3 | 3 | 3 | 3 |
| S3 | 2 | 2 | 2 | 2 |
| N1 | 0 | 2 | 2 | 2 |
| N2 | 0 | 1 | 1 | 0 |
| N3 | 0 | 1 | 1 | 0 |
| PC-1 | 2 | 2 | 2 | 2 |
| PC-4 | 6 | 6 | 6 | 6 |
| PC-5 | 2 | 2 | 2 | 2 |
| PC-6 | 2 | 2 | 2 | 2 |
| PC-10 | 11 | 11 | 11 | 0 |

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
| 12 | Previous selection · Alt+← | Previous selection · Alt+← | [not built] Previous selection | same after the milestone; today: kind command/disabled (not built), key Alt+←/none | m PC-1 |
| 13 | Next selection · Alt+→ | Next selection · Alt+→ | [not built] Next selection | same after the milestone; today: kind command/disabled (not built), key Alt+→/none | m PC-1 |
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
| 30 | [submenu] Landmark types | [submenu] Landmark types |  | same |  |
| 31 | › [submenu] Physical | › [submenu] Physical |  | same |  |
| 32 | › › Peak | › › Peak |  | same |  |
| 33 | › › Ridge | › › Ridge |  | same |  |
| 34 | › › Saddle   not buildable | › › Saddle |  | differs: label | s R1 |
| 35 | › › Cliff | › › Cliff |  | same |  |
| 36 | › › Gorge | › › Gorge |  | same |  |
| 37 | › › Cave | › › Cave |  | same |  |
| 38 | › › Waterfall | › › Waterfall |  | same |  |
| 39 | › › Spring | › › Spring |  | same |  |
| 40 | › › Lake | › › Lake |  | same |  |
| 41 | › › Delta   not buildable | › › Delta |  | differs: label | s R1 |
| 42 | › › River confluence | › › River confluence |  | same |  |
| 43 | › › Volcanic feature | › › Volcanic feature |  | same |  |
| 44 | › › Rock formation | › › Rock formation |  | same |  |
| 45 | › › Glacial feature   not buildable | › › Glacial feature |  | differs: label | s R1 |
| 46 | › › Ancient forest   not buildable | › › Ancient forest |  | differs: label | s R1 |
| 47 | › › ——— | › › ——— |  | same |  |
| 48 | › › Open Physical in the dock | › › Open Physical in the dock |  | same |  |
| 49 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 50 | › [submenu] Transport | › [submenu] Transport |  | same |  |
| 51 | › › Mountain pass | › › Mountain pass |  | same |  |
| 52 | › › River crossing   not buildable | › › River crossing |  | differs: label | s R1 |
| 53 | › › Ford | › › Ford |  | same |  |
| 54 | › › Bridge site | › › Bridge site |  | same |  |
| 55 | › › Road junction | › › Road junction |  | same |  |
| 56 | › › Caravan station | › › Caravan station |  | same |  |
| 57 | › › Portage | › › Portage |  | same |  |
| 58 | › › Harbour | › › Harbour |  | same |  |
| 59 | › › ——— | › › ——— |  | same |  |
| 60 | › › Open Transport in the dock | › › Open Transport in the dock |  | same |  |
| 61 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 62 | › [submenu] Economic | › [submenu] Economic |  | same |  |
| 63 | › › Mine | › › Mine |  | same |  |
| 64 | › › Quarry | › › Quarry |  | same |  |
| 65 | › › Salt works   not buildable | › › Salt works |  | differs: label | s R1 |
| 66 | › › Resource extraction site | › › Resource extraction site |  | same |  |
| 67 | › › Market site | › › Market site |  | same |  |
| 68 | › › Trade depot | › › Trade depot |  | same |  |
| 69 | › › ——— | › › ——— |  | same |  |
| 70 | › › Open Economic in the dock | › › Open Economic in the dock |  | same |  |
| 71 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 72 | › [submenu] Military | › [submenu] Military |  | same |  |
| 73 | › › Fort | › › Fort |  | same |  |
| 74 | › › Watchtower | › › Watchtower |  | same |  |
| 75 | › › Fortified pass | › › Fortified pass |  | same |  |
| 76 | › › Fortified crossing | › › Fortified crossing |  | same |  |
| 77 | › › Battlefield | › › Battlefield |  | same |  |
| 78 | › › Border marker | › › Border marker |  | same |  |
| 79 | › › ——— | › › ——— |  | same |  |
| 80 | › › Open Military in the dock | › › Open Military in the dock |  | same |  |
| 81 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 82 | › [submenu] Cultural | › [submenu] Cultural |  | same |  |
| 83 | › › Shrine   not buildable | › › Shrine |  | differs: label | s R1 |
| 84 | › › Temple   not buildable | › › Temple |  | differs: label | s R1 |
| 85 | › › Sacred grove   not buildable | › › Sacred grove |  | differs: label | s R1 |
| 86 | › › Sacred mountain   not buildable | › › Sacred mountain |  | differs: label | s R1 |
| 87 | › › Pilgrimage site   not buildable | › › Pilgrimage site |  | differs: label | s R1 |
| 88 | › › Tomb   not buildable | › › Tomb |  | differs: label | s R1 |
| 89 | › › Monument   not buildable | › › Monument |  | differs: label | s R1 |
| 90 | › › Ceremonial site   not buildable | › › Ceremonial site |  | differs: label | s R1 |
| 91 | › › ——— | › › ——— |  | same |  |
| 92 | › › Open Cultural in the dock | › › Open Cultural in the dock |  | same |  |
| 93 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 94 | › [submenu] Historical | › [submenu] Historical |  | same |  |
| 95 | › › Ruin   not buildable | › › Ruin |  | differs: label | s R1 |
| 96 | › › Abandoned settlement | › › Abandoned settlement |  | same |  |
| 97 | › › Ancient road | › › Ancient road |  | same |  |
| 98 | › › Historic battlefield   not buildable | › › Historic battlefield |  | differs: label | s R1 |
| 99 | › › Destroyed fortress   not buildable | › › Destroyed fortress |  | differs: label | s R1 |
| 100 | › › Historic crossing   not buildable | › › Historic crossing |  | differs: label | s R1 |
| 101 | › › ——— | › › ——— |  | same |  |
| 102 | › › Open Historical in the dock | › › Open Historical in the dock |  | same |  |
| 103 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 104 | › Placed by hand   3 | › Placed by hand |  | same |  |
| 105 | › ——— | › ——— |  | same |  |
| 106 | › Landmark icons… | › Landmark icons… |  | same |  |
| 107 | › Landmark label style… | › Landmark label style… |  | same |  |
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
| 48 | › [submenu] Azimuth | › [submenu] Azimuth |  | same |  |
| 49 | › › [radio] 0 deg | › › [radio] 0 deg |  | same |  |
| 50 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 51 | › › [radio] 90 deg | › › [radio] 90 deg |  | same |  |
| 52 | › › [radio] 135 deg | › › [radio] 135 deg |  | same |  |
| 53 | › › [radio] 180 deg | › › [radio] 180 deg |  | same |  |
| 54 | › › [radio] 225 deg | › › [radio] 225 deg |  | same |  |
| 55 | › › [radio] 270 deg | › › [radio] 270 deg |  | same |  |
| 56 | › › [radio] 315 deg | › › [radio] 315 deg |  | same |  |
| 57 | › [submenu] Elevation | › [submenu] Elevation |  | same |  |
| 58 | › › [radio] 15 deg | › › [radio] 15 deg |  | same |  |
| 59 | › › [radio] 25 deg | › › [radio] 25 deg |  | same |  |
| 60 | › › [radio] 35 deg | › › [radio] 35 deg |  | same |  |
| 61 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 62 | › › [radio] 55 deg | › › [radio] 55 deg |  | same |  |
| 63 | › › [radio] 65 deg | › › [radio] 65 deg |  | same |  |
| 64 | › › [radio] 75 deg | › › [radio] 75 deg |  | same |  |
| 65 | › [submenu] Ambient | › [submenu] Ambient |  | same |  |
| 66 | › › [radio] 0.00 | › › [radio] 0.00 |  | same |  |
| 67 | › › [radio] 0.15 | › › [radio] 0.15 |  | same |  |
| 68 | › › [radio] 0.25 | › › [radio] 0.25 |  | same |  |
| 69 | › › [radio] 0.35 | › › [radio] 0.35 |  | same |  |
| 70 | › › [radio] 0.50 | › › [radio] 0.50 |  | same |  |
| 71 | › › [radio] 0.70 | › › [radio] 0.70 |  | same |  |
| 72 | › [submenu] Multidirectional | › [submenu] Multidirectional |  | same |  |
| 73 | › › [radio] 1 light (single sun) | › › [radio] 1 light (single sun) |  | same |  |
| 74 | › › [radio] 2 lights | › › [radio] 2 lights |  | same |  |
| 75 | › › [radio] 4 lights | › › [radio] 4 lights |  | same |  |
| 76 | › › [radio] 6 lights | › › [radio] 6 lights |  | same |  |
| 77 | › › [radio] 8 lights | › › [radio] 8 lights |  | same |  |
| 78 | › › [radio] 12 lights | › › [radio] 12 lights |  | same |  |
| 79 | › ——— | › ——— |  | same |  |
| 80 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 81 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 82 | › [radio] 1x | › [radio] 1x |  | same |  |
| 83 | › [radio] 2x | › [radio] 2x |  | same |  |
| 84 | › [radio] 4x | › [radio] 4x |  | same |  |
| 85 | › ——— | › ——— |  | same |  |
| 86 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 87 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 88 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 89 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 90 | › [radio] Manual | › [radio] Manual |  | same |  |
| 91 | › ——— | › ——— |  | same |  |
| 92 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 93 | › Leave deep detail | › Leave deep detail |  | same |  |
| 94 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 95 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 96 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 97 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 98 | › ——— | › ——— |  | same |  |
| 99 | › [submenu] LOD levels | › [submenu] LOD levels |  | same |  |
| 100 | › › [radio] LOD 0–0   1 tile | › › [radio] LOD 0–0   1 tile |  | same |  |
| 101 | › › [radio] LOD 0–1   5 tiles | › › [radio] LOD 0–1   5 tiles |  | same |  |
| 102 | › › [radio] LOD 0–2   21 tiles | › › [radio] LOD 0–2   21 tiles |  | same |  |
| 103 | › › [radio] LOD 0–3   85 tiles | › › [radio] LOD 0–3   85 tiles |  | same |  |
| 104 | › › [radio] LOD 0–4   341 tiles | › › [radio] LOD 0–4   341 tiles |  | same |  |
| 105 | › › [radio] LOD 0–5   1 365 tiles | › › [radio] LOD 0–5   1 365 tiles |  | same |  |
| 106 | › › [radio] LOD 0–6   5 461 tiles | › › [radio] LOD 0–6   5 461 tiles |  | same |  |
| 107 | › › [radio] LOD 0–7   21 845 tiles | › › [radio] LOD 0–7   21 845 tiles |  | same |  |
| 108 | › › [radio] LOD 0–8   87 381 tiles | › › [radio] LOD 0–8   87 381 tiles |  | same |  |
| 109 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 110 | › [readout] 85 chunks · 1.2 GB · to LOD 3 | › [readout] … chunks · … · empty |  | differs: label | s R2 |
| 111 | › [submenu] Size cap · GB | › [submenu] Size cap · GB |  | same |  |
| 112 | › › [radio] No cap | › › [radio] No cap |  | same |  |
| 113 | › › [radio] 0.5 GB | › › [radio] 0.5 GB |  | same |  |
| 114 | › › [radio] 1 GB | › › [radio] 1 GB |  | same |  |
| 115 | › › [radio] 2 GB | › › [radio] 2 GB |  | same |  |
| 116 | › › [radio] 4 GB | › › [radio] 4 GB |  | same |  |
| 117 | › › [radio] 8 GB | › › [radio] 8 GB |  | same |  |
| 118 | › › [radio] 16 GB | › › [radio] 16 GB |  | same |  |
| 119 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 120 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 121 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 122 | › ——— | › ——— |  | same |  |
| 123 | › Export atlas… | › Export atlas… |  | same |  |
| 124 | › Import atlas… | › Import atlas… |  | same |  |
| 125 | › ——— | › ——— |  | same |  |
| 126 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 127 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 128 | › [check] Grid | › [check] Grid |  | same |  |
| 129 | › [check] Colors | › [check] Colors |  | same |  |
| 130 | › [check] Labels | › [check] Labels |  | same |  |
| 131 | › ——— | › ——— |  | same |  |
| 132 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 133 | **MEMORY** | **MEMORY** |  | same |  |
| 134 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 135 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 136 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 137 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 138 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 139 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 140 | › ——— | › ——— |  | same |  |
| 141 | › Clear undo history now | › Clear undo history now |  | same |  |
| 142 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 143 | Clear caches… | Clear caches… |  | same |  |
| 144 | **APPLICATION** | **APPLICATION** |  | same |  |
| 145 | Storage locations… | Storage locations… |  | same |  |
| 146 | Assistant… | Assistant… |  | same |  |
| 147 | Quick bar… | Quick bar… |  | same |  |
| 148 | [check] Open panels in their own window | [check] Open panels in their own window |  | same |  |
| 149 | [submenu] Theme | [submenu] Theme |  | same |  |
| 150 | › [radio] Dark | › [radio] Dark |  | same |  |
| 151 | › [radio] Light | › [radio] Light |  | same |  |
| 152 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 153 | [submenu] Units | [submenu] Units |  | same |  |
| 154 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 155 | › [radio] Miles | › [radio] Miles |  | same |  |
| 156 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 157 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 158 | › [radio] Left | › [radio] Left |  | same |  |
| 159 | › [radio] Right | › [radio] Right |  | same |  |
| 160 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |

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
| 22 | › Reset panel | › [not built] Reset panel |  | differs: kind command/disabled (not built) | m PC-10 |
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
| 33 | › [submenu] Force tier | › [submenu] Force tier |  | same |  |
| 34 | › › [radio] Automatic (as measured) | › › [radio] Automatic (as measured) |  | same |  |
| 35 | › › [radio] Off | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 36 | › › [radio] Basic | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 37 | › › [radio] Standard | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 38 | › › [radio] Full | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 39 | ——— | ——— |  | same |  |
| 40 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 41 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 42 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 43 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 44 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 45 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 46 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 47 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 48 | ——— | ——— |  | same |  |
| 49 | Reset layout | Reset layout |  | same |  |
| 50 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 51 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 52 | › Generate | › Generate |  | same |  |
| 53 | › Sculpt | › Sculpt |  | same |  |
| 54 | › Cartography | › Cartography |  | same |  |
| 55 | › Journey | › Journey |  | same |  |
| 56 | › ——— | › ——— |  | same |  |
| 57 | › Save layout as… | › Save layout as… |  | same |  |
| 58 | › Forget layout… | › Forget layout… |  | same |  |

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
| 12 | Previous selection · Alt+← | Previous selection · Alt+← | [not built] Previous selection | same after the milestone; today: kind command/disabled (not built), key Alt+←/none | m PC-1 |
| 13 | Next selection · Alt+→ | Next selection · Alt+→ | [not built] Next selection | same after the milestone; today: kind command/disabled (not built), key Alt+→/none | m PC-1 |
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
| 30 | [submenu] Landmark types | [submenu] Landmark types |  | same |  |
| 31 | › [submenu] Physical | › [submenu] Physical |  | same |  |
| 32 | › › Peak | › › Peak |  | same |  |
| 33 | › › Ridge | › › Ridge |  | same |  |
| 34 | › › Saddle   not buildable | › › Saddle |  | differs: label | s R1 |
| 35 | › › Cliff | › › Cliff |  | same |  |
| 36 | › › Gorge | › › Gorge |  | same |  |
| 37 | › › Cave | › › Cave |  | same |  |
| 38 | › › Waterfall | › › Waterfall |  | same |  |
| 39 | › › Spring | › › Spring |  | same |  |
| 40 | › › Lake | › › Lake |  | same |  |
| 41 | › › Delta   not buildable | › › Delta |  | differs: label | s R1 |
| 42 | › › River confluence | › › River confluence |  | same |  |
| 43 | › › Volcanic feature | › › Volcanic feature |  | same |  |
| 44 | › › Rock formation | › › Rock formation |  | same |  |
| 45 | › › Glacial feature   not buildable | › › Glacial feature |  | differs: label | s R1 |
| 46 | › › Ancient forest   not buildable | › › Ancient forest |  | differs: label | s R1 |
| 47 | › › ——— | › › ——— |  | same |  |
| 48 | › › Open Physical in the dock | › › Open Physical in the dock |  | same |  |
| 49 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 50 | › [submenu] Transport | › [submenu] Transport |  | same |  |
| 51 | › › Mountain pass | › › Mountain pass |  | same |  |
| 52 | › › River crossing   not buildable | › › River crossing |  | differs: label | s R1 |
| 53 | › › Ford | › › Ford |  | same |  |
| 54 | › › Bridge site | › › Bridge site |  | same |  |
| 55 | › › Road junction | › › Road junction |  | same |  |
| 56 | › › Caravan station | › › Caravan station |  | same |  |
| 57 | › › Portage | › › Portage |  | same |  |
| 58 | › › Harbour | › › Harbour |  | same |  |
| 59 | › › ——— | › › ——— |  | same |  |
| 60 | › › Open Transport in the dock | › › Open Transport in the dock |  | same |  |
| 61 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 62 | › [submenu] Economic | › [submenu] Economic |  | same |  |
| 63 | › › Mine | › › Mine |  | same |  |
| 64 | › › Quarry | › › Quarry |  | same |  |
| 65 | › › Salt works   not buildable | › › Salt works |  | differs: label | s R1 |
| 66 | › › Resource extraction site | › › Resource extraction site |  | same |  |
| 67 | › › Market site | › › Market site |  | same |  |
| 68 | › › Trade depot | › › Trade depot |  | same |  |
| 69 | › › ——— | › › ——— |  | same |  |
| 70 | › › Open Economic in the dock | › › Open Economic in the dock |  | same |  |
| 71 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 72 | › [submenu] Military | › [submenu] Military |  | same |  |
| 73 | › › Fort | › › Fort |  | same |  |
| 74 | › › Watchtower | › › Watchtower |  | same |  |
| 75 | › › Fortified pass | › › Fortified pass |  | same |  |
| 76 | › › Fortified crossing | › › Fortified crossing |  | same |  |
| 77 | › › Battlefield | › › Battlefield |  | same |  |
| 78 | › › Border marker | › › Border marker |  | same |  |
| 79 | › › ——— | › › ——— |  | same |  |
| 80 | › › Open Military in the dock | › › Open Military in the dock |  | same |  |
| 81 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 82 | › [submenu] Cultural | › [submenu] Cultural |  | same |  |
| 83 | › › Shrine   not buildable | › › Shrine |  | differs: label | s R1 |
| 84 | › › Temple   not buildable | › › Temple |  | differs: label | s R1 |
| 85 | › › Sacred grove   not buildable | › › Sacred grove |  | differs: label | s R1 |
| 86 | › › Sacred mountain   not buildable | › › Sacred mountain |  | differs: label | s R1 |
| 87 | › › Pilgrimage site   not buildable | › › Pilgrimage site |  | differs: label | s R1 |
| 88 | › › Tomb   not buildable | › › Tomb |  | differs: label | s R1 |
| 89 | › › Monument   not buildable | › › Monument |  | differs: label | s R1 |
| 90 | › › Ceremonial site   not buildable | › › Ceremonial site |  | differs: label | s R1 |
| 91 | › › ——— | › › ——— |  | same |  |
| 92 | › › Open Cultural in the dock | › › Open Cultural in the dock |  | same |  |
| 93 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 94 | › [submenu] Historical | › [submenu] Historical |  | same |  |
| 95 | › › Ruin   not buildable | › › Ruin |  | differs: label | s R1 |
| 96 | › › Abandoned settlement | › › Abandoned settlement |  | same |  |
| 97 | › › Ancient road | › › Ancient road |  | same |  |
| 98 | › › Historic battlefield   not buildable | › › Historic battlefield |  | differs: label | s R1 |
| 99 | › › Destroyed fortress   not buildable | › › Destroyed fortress |  | differs: label | s R1 |
| 100 | › › Historic crossing   not buildable | › › Historic crossing |  | differs: label | s R1 |
| 101 | › › ——— | › › ——— |  | same |  |
| 102 | › › Open Historical in the dock | › › Open Historical in the dock |  | same |  |
| 103 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 104 | › Placed by hand   3 | › Placed by hand |  | same |  |
| 105 | › ——— | › ——— |  | same |  |
| 106 | › Landmark icons… | › Landmark icons… |  | same |  |
| 107 | › Landmark label style… | › Landmark label style… |  | same |  |
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
| 48 | › [submenu] Azimuth | › [submenu] Azimuth |  | same |  |
| 49 | › › [radio] 0 deg | › › [radio] 0 deg |  | same |  |
| 50 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 51 | › › [radio] 90 deg | › › [radio] 90 deg |  | same |  |
| 52 | › › [radio] 135 deg | › › [radio] 135 deg |  | same |  |
| 53 | › › [radio] 180 deg | › › [radio] 180 deg |  | same |  |
| 54 | › › [radio] 225 deg | › › [radio] 225 deg |  | same |  |
| 55 | › › [radio] 270 deg | › › [radio] 270 deg |  | same |  |
| 56 | › › [radio] 315 deg | › › [radio] 315 deg |  | same |  |
| 57 | › [submenu] Elevation | › [submenu] Elevation |  | same |  |
| 58 | › › [radio] 15 deg | › › [radio] 15 deg |  | same |  |
| 59 | › › [radio] 25 deg | › › [radio] 25 deg |  | same |  |
| 60 | › › [radio] 35 deg | › › [radio] 35 deg |  | same |  |
| 61 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 62 | › › [radio] 55 deg | › › [radio] 55 deg |  | same |  |
| 63 | › › [radio] 65 deg | › › [radio] 65 deg |  | same |  |
| 64 | › › [radio] 75 deg | › › [radio] 75 deg |  | same |  |
| 65 | › [submenu] Ambient | › [submenu] Ambient |  | same |  |
| 66 | › › [radio] 0.00 | › › [radio] 0.00 |  | same |  |
| 67 | › › [radio] 0.15 | › › [radio] 0.15 |  | same |  |
| 68 | › › [radio] 0.25 | › › [radio] 0.25 |  | same |  |
| 69 | › › [radio] 0.35 | › › [radio] 0.35 |  | same |  |
| 70 | › › [radio] 0.50 | › › [radio] 0.50 |  | same |  |
| 71 | › › [radio] 0.70 | › › [radio] 0.70 |  | same |  |
| 72 | › [submenu] Multidirectional | › [submenu] Multidirectional |  | same |  |
| 73 | › › [radio] 1 light (single sun) | › › [radio] 1 light (single sun) |  | same |  |
| 74 | › › [radio] 2 lights | › › [radio] 2 lights |  | same |  |
| 75 | › › [radio] 4 lights | › › [radio] 4 lights |  | same |  |
| 76 | › › [radio] 6 lights | › › [radio] 6 lights |  | same |  |
| 77 | › › [radio] 8 lights | › › [radio] 8 lights |  | same |  |
| 78 | › › [radio] 12 lights | › › [radio] 12 lights |  | same |  |
| 79 | › ——— | › ——— |  | same |  |
| 80 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 81 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 82 | › [radio] 1x | › [radio] 1x |  | same |  |
| 83 | › [radio] 2x | › [radio] 2x |  | same |  |
| 84 | › [radio] 4x | › [radio] 4x |  | same |  |
| 85 | › ——— | › ——— |  | same |  |
| 86 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 87 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 88 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 89 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 90 | › [radio] Manual | › [radio] Manual |  | same |  |
| 91 | › ——— | › ——— |  | same |  |
| 92 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 93 | › Leave deep detail | › Leave deep detail |  | same |  |
| 94 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 95 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 96 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 97 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 98 | › ——— | › ——— |  | same |  |
| 99 | › [submenu] LOD levels | › [submenu] LOD levels |  | same |  |
| 100 | › › [radio] LOD 0–0   1 tile | › › [radio] LOD 0–0   1 tile |  | same |  |
| 101 | › › [radio] LOD 0–1   5 tiles | › › [radio] LOD 0–1   5 tiles |  | same |  |
| 102 | › › [radio] LOD 0–2   21 tiles | › › [radio] LOD 0–2   21 tiles |  | same |  |
| 103 | › › [radio] LOD 0–3   85 tiles | › › [radio] LOD 0–3   85 tiles |  | same |  |
| 104 | › › [radio] LOD 0–4   341 tiles | › › [radio] LOD 0–4   341 tiles |  | same |  |
| 105 | › › [radio] LOD 0–5   1 365 tiles | › › [radio] LOD 0–5   1 365 tiles |  | same |  |
| 106 | › › [radio] LOD 0–6   5 461 tiles | › › [radio] LOD 0–6   5 461 tiles |  | same |  |
| 107 | › › [radio] LOD 0–7   21 845 tiles | › › [radio] LOD 0–7   21 845 tiles |  | same |  |
| 108 | › › [radio] LOD 0–8   87 381 tiles | › › [radio] LOD 0–8   87 381 tiles |  | same |  |
| 109 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 110 | › [readout] 85 chunks · 1.2 GB · to LOD 3 | › [readout] … chunks · … · empty |  | differs: label | s R2 |
| 111 | › [submenu] Size cap · GB | › [submenu] Size cap · GB |  | same |  |
| 112 | › › [radio] No cap | › › [radio] No cap |  | same |  |
| 113 | › › [radio] 0.5 GB | › › [radio] 0.5 GB |  | same |  |
| 114 | › › [radio] 1 GB | › › [radio] 1 GB |  | same |  |
| 115 | › › [radio] 2 GB | › › [radio] 2 GB |  | same |  |
| 116 | › › [radio] 4 GB | › › [radio] 4 GB |  | same |  |
| 117 | › › [radio] 8 GB | › › [radio] 8 GB |  | same |  |
| 118 | › › [radio] 16 GB | › › [radio] 16 GB |  | same |  |
| 119 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 120 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 121 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 122 | › ——— | › ——— |  | same |  |
| 123 | › Export atlas… | › Export atlas… |  | same |  |
| 124 | › Import atlas… | › Import atlas… |  | same |  |
| 125 | › ——— | › ——— |  | same |  |
| 126 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 127 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 128 | › [check] Grid | › [check] Grid |  | same |  |
| 129 | › [check] Colors | › [check] Colors |  | same |  |
| 130 | › [check] Labels | › [check] Labels |  | same |  |
| 131 | › ——— | › ——— |  | same |  |
| 132 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 133 | **MEMORY** | **MEMORY** |  | same |  |
| 134 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 135 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 136 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 137 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 138 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 139 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 140 | › ——— | › ——— |  | same |  |
| 141 | › Clear undo history now | › Clear undo history now |  | same |  |
| 142 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 143 | Clear caches… | Clear caches… |  | same |  |
| 144 | **APPLICATION** | **APPLICATION** |  | same |  |
| 145 | Storage locations… | Storage locations… |  | same |  |
| 146 | Assistant… | Assistant… |  | same |  |
| 147 | Quick bar… | Quick bar… |  | same |  |
| 148 | [DESKTOP ONLY] Open panels in their own window |  |  | mockup-only | n N3 |
| 149 | [submenu] Theme | [submenu] Theme |  | same |  |
| 150 | › [radio] Dark | › [radio] Dark |  | same |  |
| 151 | › [radio] Light | › [radio] Light |  | same |  |
| 152 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 153 | [submenu] Units | [submenu] Units |  | same |  |
| 154 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 155 | › [radio] Miles | › [radio] Miles |  | same |  |
| 156 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 157 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 158 | › [radio] Left | › [radio] Left |  | same |  |
| 159 | › [radio] Right | › [radio] Right |  | same |  |
| 160 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |
| 161 | **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 162 | Touch & gestures… |  |  | mockup-only | n N1 |

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
| 22 | › Reset panel | › [not built] Reset panel |  | differs: kind command/disabled (not built) | m PC-10 |
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
| 33 | › [submenu] Force tier | › [submenu] Force tier |  | same |  |
| 34 | › › [radio] Automatic (as measured) | › › [radio] Automatic (as measured) |  | same |  |
| 35 | › › [radio] Off | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 36 | › › [radio] Basic | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 37 | › › [radio] Standard | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 38 | › › [radio] Full | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 39 | ——— | ——— |  | same |  |
| 40 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 41 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 42 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 43 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 44 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 45 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 46 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 47 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 48 | ——— | ——— |  | same |  |
| 49 | Reset layout | Reset layout |  | same |  |
| 50 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 51 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 52 | › Generate | › Generate |  | same |  |
| 53 | › Sculpt | › Sculpt |  | same |  |
| 54 | › Cartography | › Cartography |  | same |  |
| 55 | › Journey | › Journey |  | same |  |
| 56 | › ——— | › ——— |  | same |  |
| 57 | › Save layout as… | › Save layout as… |  | same |  |
| 58 | › Forget layout… | › Forget layout… |  | same |  |

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
| 6 | [submenu] Landmark types | [submenu] Landmark types |  | same |  |
| 7 | › [submenu] Physical | › [submenu] Physical |  | same |  |
| 8 | › › Peak | › › Peak |  | same |  |
| 9 | › › Ridge | › › Ridge |  | same |  |
| 10 | › › Saddle   not buildable | › › Saddle |  | differs: label | s R1 |
| 11 | › › Cliff | › › Cliff |  | same |  |
| 12 | › › Gorge | › › Gorge |  | same |  |
| 13 | › › Cave | › › Cave |  | same |  |
| 14 | › › Waterfall | › › Waterfall |  | same |  |
| 15 | › › Spring | › › Spring |  | same |  |
| 16 | › › Lake | › › Lake |  | same |  |
| 17 | › › Delta   not buildable | › › Delta |  | differs: label | s R1 |
| 18 | › › River confluence | › › River confluence |  | same |  |
| 19 | › › Volcanic feature | › › Volcanic feature |  | same |  |
| 20 | › › Rock formation | › › Rock formation |  | same |  |
| 21 | › › Glacial feature   not buildable | › › Glacial feature |  | differs: label | s R1 |
| 22 | › › Ancient forest   not buildable | › › Ancient forest |  | differs: label | s R1 |
| 23 | › › ——— | › › ——— |  | same |  |
| 24 | › › Open Physical in the dock | › › Open Physical in the dock |  | same |  |
| 25 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 26 | › [submenu] Transport | › [submenu] Transport |  | same |  |
| 27 | › › Mountain pass | › › Mountain pass |  | same |  |
| 28 | › › River crossing   not buildable | › › River crossing |  | differs: label | s R1 |
| 29 | › › Ford | › › Ford |  | same |  |
| 30 | › › Bridge site | › › Bridge site |  | same |  |
| 31 | › › Road junction | › › Road junction |  | same |  |
| 32 | › › Caravan station | › › Caravan station |  | same |  |
| 33 | › › Portage | › › Portage |  | same |  |
| 34 | › › Harbour | › › Harbour |  | same |  |
| 35 | › › ——— | › › ——— |  | same |  |
| 36 | › › Open Transport in the dock | › › Open Transport in the dock |  | same |  |
| 37 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 38 | › [submenu] Economic | › [submenu] Economic |  | same |  |
| 39 | › › Mine | › › Mine |  | same |  |
| 40 | › › Quarry | › › Quarry |  | same |  |
| 41 | › › Salt works   not buildable | › › Salt works |  | differs: label | s R1 |
| 42 | › › Resource extraction site | › › Resource extraction site |  | same |  |
| 43 | › › Market site | › › Market site |  | same |  |
| 44 | › › Trade depot | › › Trade depot |  | same |  |
| 45 | › › ——— | › › ——— |  | same |  |
| 46 | › › Open Economic in the dock | › › Open Economic in the dock |  | same |  |
| 47 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 48 | › [submenu] Military | › [submenu] Military |  | same |  |
| 49 | › › Fort | › › Fort |  | same |  |
| 50 | › › Watchtower | › › Watchtower |  | same |  |
| 51 | › › Fortified pass | › › Fortified pass |  | same |  |
| 52 | › › Fortified crossing | › › Fortified crossing |  | same |  |
| 53 | › › Battlefield | › › Battlefield |  | same |  |
| 54 | › › Border marker | › › Border marker |  | same |  |
| 55 | › › ——— | › › ——— |  | same |  |
| 56 | › › Open Military in the dock | › › Open Military in the dock |  | same |  |
| 57 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 58 | › [submenu] Cultural | › [submenu] Cultural |  | same |  |
| 59 | › › Shrine   not buildable | › › Shrine |  | differs: label | s R1 |
| 60 | › › Temple   not buildable | › › Temple |  | differs: label | s R1 |
| 61 | › › Sacred grove   not buildable | › › Sacred grove |  | differs: label | s R1 |
| 62 | › › Sacred mountain   not buildable | › › Sacred mountain |  | differs: label | s R1 |
| 63 | › › Pilgrimage site   not buildable | › › Pilgrimage site |  | differs: label | s R1 |
| 64 | › › Tomb   not buildable | › › Tomb |  | differs: label | s R1 |
| 65 | › › Monument   not buildable | › › Monument |  | differs: label | s R1 |
| 66 | › › Ceremonial site   not buildable | › › Ceremonial site |  | differs: label | s R1 |
| 67 | › › ——— | › › ——— |  | same |  |
| 68 | › › Open Cultural in the dock | › › Open Cultural in the dock |  | same |  |
| 69 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 70 | › [submenu] Historical | › [submenu] Historical |  | same |  |
| 71 | › › Ruin   not buildable | › › Ruin |  | differs: label | s R1 |
| 72 | › › Abandoned settlement | › › Abandoned settlement |  | same |  |
| 73 | › › Ancient road | › › Ancient road |  | same |  |
| 74 | › › Historic battlefield   not buildable | › › Historic battlefield |  | differs: label | s R1 |
| 75 | › › Destroyed fortress   not buildable | › › Destroyed fortress |  | differs: label | s R1 |
| 76 | › › Historic crossing   not buildable | › › Historic crossing |  | differs: label | s R1 |
| 77 | › › ——— | › › ——— |  | same |  |
| 78 | › › Open Historical in the dock | › › Open Historical in the dock |  | same |  |
| 79 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 80 | › Placed by hand   3 | › Placed by hand |  | same |  |
| 81 | › ——— | › ——— |  | same |  |
| 82 | › Landmark icons… | › Landmark icons… |  | same |  |
| 83 | › Landmark label style… | › Landmark label style… |  | same |  |
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
| 102 | › Previous selection · Alt+← | › Previous selection · Alt+← | › [not built] Previous selection | same after the milestone; today: kind command/disabled (not built), key Alt+←/none | m PC-1 |
| 103 | › Next selection · Alt+→ | › Next selection · Alt+→ | › [not built] Next selection | same after the milestone; today: kind command/disabled (not built), key Alt+→/none | m PC-1 |
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
| 188 | › › [submenu] Azimuth | › › [submenu] Azimuth |  | same |  |
| 189 | › › › [radio] 0 deg | › › › [radio] 0 deg |  | same |  |
| 190 | › › › [radio] 45 deg | › › › [radio] 45 deg |  | same |  |
| 191 | › › › [radio] 90 deg | › › › [radio] 90 deg |  | same |  |
| 192 | › › › [radio] 135 deg | › › › [radio] 135 deg |  | same |  |
| 193 | › › › [radio] 180 deg | › › › [radio] 180 deg |  | same |  |
| 194 | › › › [radio] 225 deg | › › › [radio] 225 deg |  | same |  |
| 195 | › › › [radio] 270 deg | › › › [radio] 270 deg |  | same |  |
| 196 | › › › [radio] 315 deg | › › › [radio] 315 deg |  | same |  |
| 197 | › › [submenu] Elevation | › › [submenu] Elevation |  | same |  |
| 198 | › › › [radio] 15 deg | › › › [radio] 15 deg |  | same |  |
| 199 | › › › [radio] 25 deg | › › › [radio] 25 deg |  | same |  |
| 200 | › › › [radio] 35 deg | › › › [radio] 35 deg |  | same |  |
| 201 | › › › [radio] 45 deg | › › › [radio] 45 deg |  | same |  |
| 202 | › › › [radio] 55 deg | › › › [radio] 55 deg |  | same |  |
| 203 | › › › [radio] 65 deg | › › › [radio] 65 deg |  | same |  |
| 204 | › › › [radio] 75 deg | › › › [radio] 75 deg |  | same |  |
| 205 | › › [submenu] Ambient | › › [submenu] Ambient |  | same |  |
| 206 | › › › [radio] 0.00 | › › › [radio] 0.00 |  | same |  |
| 207 | › › › [radio] 0.15 | › › › [radio] 0.15 |  | same |  |
| 208 | › › › [radio] 0.25 | › › › [radio] 0.25 |  | same |  |
| 209 | › › › [radio] 0.35 | › › › [radio] 0.35 |  | same |  |
| 210 | › › › [radio] 0.50 | › › › [radio] 0.50 |  | same |  |
| 211 | › › › [radio] 0.70 | › › › [radio] 0.70 |  | same |  |
| 212 | › › [submenu] Multidirectional | › › [submenu] Multidirectional |  | same |  |
| 213 | › › › [radio] 1 light (single sun) | › › › [radio] 1 light (single sun) |  | same |  |
| 214 | › › › [radio] 2 lights | › › › [radio] 2 lights |  | same |  |
| 215 | › › › [radio] 4 lights | › › › [radio] 4 lights |  | same |  |
| 216 | › › › [radio] 6 lights | › › › [radio] 6 lights |  | same |  |
| 217 | › › › [radio] 8 lights | › › › [radio] 8 lights |  | same |  |
| 218 | › › › [radio] 12 lights | › › › [radio] 12 lights |  | same |  |
| 219 | › › ——— | › › ——— |  | same |  |
| 220 | › › Use the engine’s own rig | › › Use the engine's own rig |  | same |  |
| 221 | › [submenu] Relief exaggeration | › [submenu] Relief exaggeration |  | same |  |
| 222 | › › [radio] 1x | › › [radio] 1x |  | same |  |
| 223 | › › [radio] 2x | › › [radio] 2x |  | same |  |
| 224 | › › [radio] 4x | › › [radio] 4x |  | same |  |
| 225 | › › ——— | › › ——— |  | same |  |
| 226 | › › Use the engine’s own 3.4x | › › Use the engine's own 3.4x |  | same |  |
| 227 | › **TILES & LOD** | › **TILES & LOD** |  | same |  |
| 228 | › [submenu] Tiled LOD | › [submenu] Tiled LOD |  | same |  |
| 229 | › › [radio] Auto on zoom | › › [radio] Auto on zoom |  | same |  |
| 230 | › › [radio] Manual | › › [radio] Manual |  | same |  |
| 231 | › › ——— | › › ——— |  | same |  |
| 232 | › › Enter deep detail now | › › Enter deep detail now |  | same |  |
| 233 | › › Leave deep detail | › › Leave deep detail |  | same |  |
| 234 | › [submenu] Tile size · LOD levels | › [submenu] Tile size · LOD levels |  | same |  |
| 235 | › › [radio] 256 px | › › [radio] 256 px |  | same |  |
| 236 | › › [radio] 512 px | › › [radio] 512 px |  | same |  |
| 237 | › › [radio] 1024 px | › › [radio] 1024 px |  | same |  |
| 238 | › › ——— | › › ——— |  | same |  |
| 239 | › › [submenu] LOD levels | › › [submenu] LOD levels |  | same |  |
| 240 | › › › [radio] LOD 0–0   1 tile | › › › [radio] LOD 0–0   1 tile |  | same |  |
| 241 | › › › [radio] LOD 0–1   5 tiles | › › › [radio] LOD 0–1   5 tiles |  | same |  |
| 242 | › › › [radio] LOD 0–2   21 tiles | › › › [radio] LOD 0–2   21 tiles |  | same |  |
| 243 | › › › [radio] LOD 0–3   85 tiles | › › › [radio] LOD 0–3   85 tiles |  | same |  |
| 244 | › › › [radio] LOD 0–4   341 tiles | › › › [radio] LOD 0–4   341 tiles |  | same |  |
| 245 | › › › [radio] LOD 0–5   1 365 tiles | › › › [radio] LOD 0–5   1 365 tiles |  | same |  |
| 246 | › › › [radio] LOD 0–6   5 461 tiles | › › › [radio] LOD 0–6   5 461 tiles |  | same |  |
| 247 | › › › [radio] LOD 0–7   21 845 tiles | › › › [radio] LOD 0–7   21 845 tiles |  | same |  |
| 248 | › › › [radio] LOD 0–8   87 381 tiles | › › › [radio] LOD 0–8   87 381 tiles |  | same |  |
| 249 | › [submenu] Atlas cache | › [submenu] Atlas cache |  | same |  |
| 250 | › › [readout] 85 chunks · 1.2 GB · to LOD 3 | › › [readout] … chunks · … · empty |  | differs: label | s R2 |
| 251 | › › [submenu] Size cap · GB | › › [submenu] Size cap · GB |  | same |  |
| 252 | › › › [radio] No cap | › › › [radio] No cap |  | same |  |
| 253 | › › › [radio] 0.5 GB | › › › [radio] 0.5 GB |  | same |  |
| 254 | › › › [radio] 1 GB | › › › [radio] 1 GB |  | same |  |
| 255 | › › › [radio] 2 GB | › › › [radio] 2 GB |  | same |  |
| 256 | › › › [radio] 4 GB | › › › [radio] 4 GB |  | same |  |
| 257 | › › › [radio] 8 GB | › › › [radio] 8 GB |  | same |  |
| 258 | › › › [radio] 16 GB | › › › [radio] 16 GB |  | same |  |
| 259 | › › Refine detail for the current view | › › Refine detail for the current view |  | same |  |
| 260 | › › Refine viewshed for the current view | › › Refine viewshed for the current view |  | same |  |
| 261 | › › [readout] Viewshed | › › [readout] Viewshed: never refined |  | same |  |
| 262 | › › ——— | › › ——— |  | same |  |
| 263 | › › Export atlas… | › › Export atlas… |  | same |  |
| 264 | › › Import atlas… | › › Import atlas… |  | same |  |
| 265 | › › ——— | › › ——— |  | same |  |
| 266 | › › Clear atlas cache now… | › › Clear atlas cache now… |  | same |  |
| 267 | › [submenu] Chunk debug overlay | › [submenu] Chunk debug overlay |  | same |  |
| 268 | › › [check] Grid | › › [check] Grid |  | same |  |
| 269 | › › [check] Colors | › › [check] Colors |  | same |  |
| 270 | › › [check] Labels | › › [check] Labels |  | same |  |
| 271 | › › ——— | › › ——— |  | same |  |
| 272 | › › [check] Show tile borders on the map | › › [check] Show tile borders on the map |  | same |  |
| 273 | › **MEMORY** | › **MEMORY** |  | same |  |
| 274 | › [submenu] Undo history | › [submenu] Undo history |  | same |  |
| 275 | › › [radio] 64 MB | › › [radio] 64 MB |  | same |  |
| 276 | › › [radio] 128 MB | › › [radio] 128 MB |  | same |  |
| 277 | › › [radio] 256 MB | › › [radio] 256 MB |  | same |  |
| 278 | › › [radio] 512 MB | › › [radio] 512 MB |  | same |  |
| 279 | › › [radio] 1024 MB | › › [radio] 1024 MB |  | same |  |
| 280 | › › ——— | › › ——— |  | same |  |
| 281 | › › Clear undo history now | › › Clear undo history now |  | same |  |
| 282 | › [readout] Working set | › [readout] Working set   … GB of … GB |  | same |  |
| 283 | › Clear caches… | › Clear caches… |  | same |  |
| 284 | › **APPLICATION** | › **APPLICATION** |  | same |  |
| 285 | › Storage locations… | › Storage locations… |  | same |  |
| 286 | › Assistant… | › Assistant… |  | same |  |
| 287 | › Quick bar… | › Quick bar… |  | same |  |
| 288 | › [DESKTOP ONLY] Open panels in their own window |  |  | mockup-only | n N3 |
| 289 | › [submenu] Theme | › [submenu] Theme |  | same |  |
| 290 | › › [radio] Dark | › › [radio] Dark |  | same |  |
| 291 | › › [radio] Light | › › [radio] Light |  | same |  |
| 292 | › › [radio] Follow system | › › [radio] Follow system |  | same |  |
| 293 | › [submenu] Handedness | › [submenu] Handedness |  | same |  |
| 294 | › › [radio] Left | › › [radio] Left |  | same |  |
| 295 | › › [radio] Right | › › [radio] Right |  | same |  |
| 296 | › Keyboard shortcuts… | › Keyboard shortcuts… |  | same |  |
| 297 | › **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 298 | › Touch & gestures… |  |  | mockup-only | n N1 |
| 299 | [submenu] Window | [submenu] Window |  | same |  |
| 300 | › [check] Left dock | › [check] Left dock |  | same |  |
| 301 | › [check] Context panel | › [check] Context panel | › [check] Right dock | same after the milestone; today: label | m PC-4 |
| 302 | › [submenu] Context panel | › [submenu] Context panel |  | same |  |
| 303 | › › Inspect · Ctrl+1 | › › Inspect · Ctrl+1 | › › [not built] Inspect | same after the milestone; today: kind command/disabled (not built), key Ctrl+1/none | m PC-4 |
| 304 | › › Note · Ctrl+2 | › › Note · Ctrl+2 | › › [not built] Note | same after the milestone; today: kind command/disabled (not built), key Ctrl+2/none | m PC-5 |
| 305 | › › Ask (assistant) · Ctrl+3 | › › Ask (assistant) · Ctrl+3 | › › [not built] Ask (assistant) | same after the milestone; today: kind command/disabled (not built), key Ctrl+3/none | m PC-6 |
| 306 | › › ——— | › › ——— |  | same |  |
| 307 | › › Pin to the current subject | › › Pin to the current subject | › › [not built] Pin to the current subject | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 308 | › › Follow the selection | › › Follow the selection | › › [not built] Follow the selection | same after the milestone; today: kind command/disabled (not built) | m PC-4 |
| 309 | › › **MODULES** | › › **MODULES** |  | same |  |
| 310 | › › [check] History | › › [not built] History |  | differs: kind check/disabled (not built) | m PC-10 |
| 311 | › › [check] Economy | › › [not built] Economy |  | differs: kind check/disabled (not built) | m PC-10 |
| 312 | › › [check] Military | › › [not built] Military |  | differs: kind check/disabled (not built) | m PC-10 |
| 313 | › › [check] Faith, culture & relations | › › [not built] Faith, culture & relations |  | differs: kind check/disabled (not built) | m PC-10 |
| 314 | › › [check] Territory | › › [not built] Territory |  | differs: kind check/disabled (not built) | m PC-10 |
| 315 | › › [check] City layout | › › [not built] City layout |  | differs: kind check/disabled (not built) | m PC-10 |
| 316 | › › [check] Here | › › [not built] Here |  | differs: kind check/disabled (not built) | m PC-10 |
| 317 | › › [check] Routes & journeys | › › [not built] Routes & journeys |  | differs: kind check/disabled (not built) | m PC-10 |
| 318 | › › [check] Links | › › [not built] Links |  | differs: kind check/disabled (not built) | m PC-10 |
| 319 | › › [check] Recent & bookmarks | › › [not built] Recent & bookmarks |  | differs: kind check/disabled (not built) | m PC-10 |
| 320 | › › ——— | › › ——— |  | same |  |
| 321 | › › Reset panel | › › [not built] Reset panel |  | differs: kind command/disabled (not built) | m PC-10 |
| 322 | › [check] Timeline | › [check] Timeline |  | same |  |
| 323 | › [check] Status bar | › [check] Status bar |  | same |  |
| 324 | › [check] Domain rail | › [check] Domain rail |  | same |  |
| 325 | › ——— | › ——— |  | same |  |
| 326 | › [check] Session mode · Ctrl+Shift+M | › [check] Session mode · Ctrl+Shift+M |  | same |  |
| 327 | › [check] Diagnostics overlay · Shift+D | › [check] Diagnostics overlay · Shift+D |  | same |  |
| 328 | › [check] 3D view | › [check] 3D view |  | same |  |
| 329 | › [submenu] 3D capability | › [submenu] 3D capability |  | same |  |
| 330 | › › [readout] 3D tier | › › [readout] 3D tier: not tested |  | same |  |
| 331 | › › Re-test 3D | › › Re-test 3D |  | same |  |
| 332 | › › [submenu] Force tier | › › [submenu] Force tier |  | same |  |
| 333 | › › › [radio] Automatic (as measured) | › › › [radio] Automatic (as measured) |  | same |  |
| 334 | › › › [radio] Off | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 335 | › › › [radio] Basic | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 336 | › › › [radio] Standard | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 337 | › › › [radio] Full | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 338 | › ——— | › ——— |  | same |  |
| 339 | › [submenu] Workspace | › [submenu] Workspace |  | same |  |
| 340 | › › [radio] World · SM | › › [radio] World · SM |  | same |  |
| 341 | › › [radio] Civilization | › › [radio] Civilization |  | same |  |
| 342 | › › [radio] Cartography | › › [radio] Cartography |  | same |  |
| 343 | › [submenu] Open windows | › [submenu] Open windows |  | same |  |
| 344 | › › [readout] No windows open |  |  | mockup-only | s S3 |
| 345 | › › [check] Context panel | › › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 346 |  | › › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 347 | › ——— | › ——— |  | same |  |
| 348 | › Reset layout | › Reset layout |  | same |  |
| 349 | › [submenu] Layouts | › [submenu] Layouts |  | same |  |
| 350 | › › Default (reset layout) | › › Default (reset layout) |  | same |  |
| 351 | › › Generate | › › Generate |  | same |  |
| 352 | › › Sculpt | › › Sculpt |  | same |  |
| 353 | › › Cartography | › › Cartography |  | same |  |
| 354 | › › Journey | › › Journey |  | same |  |
| 355 | › › ——— | › › ——— |  | same |  |
| 356 | › › Save layout as… | › › Save layout as… |  | same |  |
| 357 | › › Forget layout… | › › Forget layout… |  | same |  |
| 358 | [submenu] Help | [submenu] Help |  | same |  |
| 359 | › [not built] Documentation | › [not built] Documentation |  | differs: reason | s S1 |
| 360 | › Gesture reference… |  |  | mockup-only | n N2 |
| 361 | › Credits & academic principles | › Credits & academic principles |  | same |  |
| 362 | › Generation info… | › Generation info… |  | same |  |
| 363 | › Save diagnostic report… | › Save diagnostic report… |  | same |  |

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
| 12 | Previous selection | Previous selection | [not built] Previous selection | same after the milestone; today: kind command/disabled (not built) | m PC-1 |
| 13 | Next selection | Next selection | [not built] Next selection | same after the milestone; today: kind command/disabled (not built) | m PC-1 |
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
| 30 | [submenu] Landmark types | [submenu] Landmark types |  | same |  |
| 31 | › [submenu] Physical | › [submenu] Physical |  | same |  |
| 32 | › › Peak | › › Peak |  | same |  |
| 33 | › › Ridge | › › Ridge |  | same |  |
| 34 | › › Saddle   not buildable | › › Saddle |  | differs: label | s R1 |
| 35 | › › Cliff | › › Cliff |  | same |  |
| 36 | › › Gorge | › › Gorge |  | same |  |
| 37 | › › Cave | › › Cave |  | same |  |
| 38 | › › Waterfall | › › Waterfall |  | same |  |
| 39 | › › Spring | › › Spring |  | same |  |
| 40 | › › Lake | › › Lake |  | same |  |
| 41 | › › Delta   not buildable | › › Delta |  | differs: label | s R1 |
| 42 | › › River confluence | › › River confluence |  | same |  |
| 43 | › › Volcanic feature | › › Volcanic feature |  | same |  |
| 44 | › › Rock formation | › › Rock formation |  | same |  |
| 45 | › › Glacial feature   not buildable | › › Glacial feature |  | differs: label | s R1 |
| 46 | › › Ancient forest   not buildable | › › Ancient forest |  | differs: label | s R1 |
| 47 | › › ——— | › › ——— |  | same |  |
| 48 | › › Open Physical in the dock | › › Open Physical in the dock |  | same |  |
| 49 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 50 | › [submenu] Transport | › [submenu] Transport |  | same |  |
| 51 | › › Mountain pass | › › Mountain pass |  | same |  |
| 52 | › › River crossing   not buildable | › › River crossing |  | differs: label | s R1 |
| 53 | › › Ford | › › Ford |  | same |  |
| 54 | › › Bridge site | › › Bridge site |  | same |  |
| 55 | › › Road junction | › › Road junction |  | same |  |
| 56 | › › Caravan station | › › Caravan station |  | same |  |
| 57 | › › Portage | › › Portage |  | same |  |
| 58 | › › Harbour | › › Harbour |  | same |  |
| 59 | › › ——— | › › ——— |  | same |  |
| 60 | › › Open Transport in the dock | › › Open Transport in the dock |  | same |  |
| 61 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 62 | › [submenu] Economic | › [submenu] Economic |  | same |  |
| 63 | › › Mine | › › Mine |  | same |  |
| 64 | › › Quarry | › › Quarry |  | same |  |
| 65 | › › Salt works   not buildable | › › Salt works |  | differs: label | s R1 |
| 66 | › › Resource extraction site | › › Resource extraction site |  | same |  |
| 67 | › › Market site | › › Market site |  | same |  |
| 68 | › › Trade depot | › › Trade depot |  | same |  |
| 69 | › › ——— | › › ——— |  | same |  |
| 70 | › › Open Economic in the dock | › › Open Economic in the dock |  | same |  |
| 71 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 72 | › [submenu] Military | › [submenu] Military |  | same |  |
| 73 | › › Fort | › › Fort |  | same |  |
| 74 | › › Watchtower | › › Watchtower |  | same |  |
| 75 | › › Fortified pass | › › Fortified pass |  | same |  |
| 76 | › › Fortified crossing | › › Fortified crossing |  | same |  |
| 77 | › › Battlefield | › › Battlefield |  | same |  |
| 78 | › › Border marker | › › Border marker |  | same |  |
| 79 | › › ——— | › › ——— |  | same |  |
| 80 | › › Open Military in the dock | › › Open Military in the dock |  | same |  |
| 81 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 82 | › [submenu] Cultural | › [submenu] Cultural |  | same |  |
| 83 | › › Shrine   not buildable | › › Shrine |  | differs: label | s R1 |
| 84 | › › Temple   not buildable | › › Temple |  | differs: label | s R1 |
| 85 | › › Sacred grove   not buildable | › › Sacred grove |  | differs: label | s R1 |
| 86 | › › Sacred mountain   not buildable | › › Sacred mountain |  | differs: label | s R1 |
| 87 | › › Pilgrimage site   not buildable | › › Pilgrimage site |  | differs: label | s R1 |
| 88 | › › Tomb   not buildable | › › Tomb |  | differs: label | s R1 |
| 89 | › › Monument   not buildable | › › Monument |  | differs: label | s R1 |
| 90 | › › Ceremonial site   not buildable | › › Ceremonial site |  | differs: label | s R1 |
| 91 | › › ——— | › › ——— |  | same |  |
| 92 | › › Open Cultural in the dock | › › Open Cultural in the dock |  | same |  |
| 93 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 94 | › [submenu] Historical | › [submenu] Historical |  | same |  |
| 95 | › › Ruin   not buildable | › › Ruin |  | differs: label | s R1 |
| 96 | › › Abandoned settlement | › › Abandoned settlement |  | same |  |
| 97 | › › Ancient road | › › Ancient road |  | same |  |
| 98 | › › Historic battlefield   not buildable | › › Historic battlefield |  | differs: label | s R1 |
| 99 | › › Destroyed fortress   not buildable | › › Destroyed fortress |  | differs: label | s R1 |
| 100 | › › Historic crossing   not buildable | › › Historic crossing |  | differs: label | s R1 |
| 101 | › › ——— | › › ——— |  | same |  |
| 102 | › › Open Historical in the dock | › › Open Historical in the dock |  | same |  |
| 103 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 104 | › Placed by hand   3 | › Placed by hand |  | same |  |
| 105 | › ——— | › ——— |  | same |  |
| 106 | › Landmark icons… | › Landmark icons… |  | same |  |
| 107 | › Landmark label style… | › Landmark label style… |  | same |  |
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
| 48 | › [submenu] Azimuth | › [submenu] Azimuth |  | same |  |
| 49 | › › [radio] 0 deg | › › [radio] 0 deg |  | same |  |
| 50 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 51 | › › [radio] 90 deg | › › [radio] 90 deg |  | same |  |
| 52 | › › [radio] 135 deg | › › [radio] 135 deg |  | same |  |
| 53 | › › [radio] 180 deg | › › [radio] 180 deg |  | same |  |
| 54 | › › [radio] 225 deg | › › [radio] 225 deg |  | same |  |
| 55 | › › [radio] 270 deg | › › [radio] 270 deg |  | same |  |
| 56 | › › [radio] 315 deg | › › [radio] 315 deg |  | same |  |
| 57 | › [submenu] Elevation | › [submenu] Elevation |  | same |  |
| 58 | › › [radio] 15 deg | › › [radio] 15 deg |  | same |  |
| 59 | › › [radio] 25 deg | › › [radio] 25 deg |  | same |  |
| 60 | › › [radio] 35 deg | › › [radio] 35 deg |  | same |  |
| 61 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 62 | › › [radio] 55 deg | › › [radio] 55 deg |  | same |  |
| 63 | › › [radio] 65 deg | › › [radio] 65 deg |  | same |  |
| 64 | › › [radio] 75 deg | › › [radio] 75 deg |  | same |  |
| 65 | › [submenu] Ambient | › [submenu] Ambient |  | same |  |
| 66 | › › [radio] 0.00 | › › [radio] 0.00 |  | same |  |
| 67 | › › [radio] 0.15 | › › [radio] 0.15 |  | same |  |
| 68 | › › [radio] 0.25 | › › [radio] 0.25 |  | same |  |
| 69 | › › [radio] 0.35 | › › [radio] 0.35 |  | same |  |
| 70 | › › [radio] 0.50 | › › [radio] 0.50 |  | same |  |
| 71 | › › [radio] 0.70 | › › [radio] 0.70 |  | same |  |
| 72 | › [submenu] Multidirectional | › [submenu] Multidirectional |  | same |  |
| 73 | › › [radio] 1 light (single sun) | › › [radio] 1 light (single sun) |  | same |  |
| 74 | › › [radio] 2 lights | › › [radio] 2 lights |  | same |  |
| 75 | › › [radio] 4 lights | › › [radio] 4 lights |  | same |  |
| 76 | › › [radio] 6 lights | › › [radio] 6 lights |  | same |  |
| 77 | › › [radio] 8 lights | › › [radio] 8 lights |  | same |  |
| 78 | › › [radio] 12 lights | › › [radio] 12 lights |  | same |  |
| 79 | › ——— | › ——— |  | same |  |
| 80 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 81 | [submenu] Relief exaggeration | [submenu] Relief exaggeration |  | same |  |
| 82 | › [radio] 1x | › [radio] 1x |  | same |  |
| 83 | › [radio] 2x | › [radio] 2x |  | same |  |
| 84 | › [radio] 4x | › [radio] 4x |  | same |  |
| 85 | › ——— | › ——— |  | same |  |
| 86 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 87 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 88 | [submenu] Tiled LOD | [submenu] Tiled LOD |  | same |  |
| 89 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 90 | › [radio] Manual | › [radio] Manual |  | same |  |
| 91 | › ——— | › ——— |  | same |  |
| 92 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 93 | › Leave deep detail | › Leave deep detail |  | same |  |
| 94 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 95 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 96 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 97 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 98 | › ——— | › ——— |  | same |  |
| 99 | › [submenu] LOD levels | › [submenu] LOD levels |  | same |  |
| 100 | › › [radio] LOD 0–0   1 tile | › › [radio] LOD 0–0   1 tile |  | same |  |
| 101 | › › [radio] LOD 0–1   5 tiles | › › [radio] LOD 0–1   5 tiles |  | same |  |
| 102 | › › [radio] LOD 0–2   21 tiles | › › [radio] LOD 0–2   21 tiles |  | same |  |
| 103 | › › [radio] LOD 0–3   85 tiles | › › [radio] LOD 0–3   85 tiles |  | same |  |
| 104 | › › [radio] LOD 0–4   341 tiles | › › [radio] LOD 0–4   341 tiles |  | same |  |
| 105 | › › [radio] LOD 0–5   1 365 tiles | › › [radio] LOD 0–5   1 365 tiles |  | same |  |
| 106 | › › [radio] LOD 0–6   5 461 tiles | › › [radio] LOD 0–6   5 461 tiles |  | same |  |
| 107 | › › [radio] LOD 0–7   21 845 tiles | › › [radio] LOD 0–7   21 845 tiles |  | same |  |
| 108 | › › [radio] LOD 0–8   87 381 tiles | › › [radio] LOD 0–8   87 381 tiles |  | same |  |
| 109 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 110 | › [readout] 85 chunks · 1.2 GB · to LOD 3 | › [readout] … chunks · … · empty |  | differs: label | s R2 |
| 111 | › [submenu] Size cap · GB | › [submenu] Size cap · GB |  | same |  |
| 112 | › › [radio] No cap | › › [radio] No cap |  | same |  |
| 113 | › › [radio] 0.5 GB | › › [radio] 0.5 GB |  | same |  |
| 114 | › › [radio] 1 GB | › › [radio] 1 GB |  | same |  |
| 115 | › › [radio] 2 GB | › › [radio] 2 GB |  | same |  |
| 116 | › › [radio] 4 GB | › › [radio] 4 GB |  | same |  |
| 117 | › › [radio] 8 GB | › › [radio] 8 GB |  | same |  |
| 118 | › › [radio] 16 GB | › › [radio] 16 GB |  | same |  |
| 119 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 120 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 121 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 122 | › ——— | › ——— |  | same |  |
| 123 | › Export atlas… | › Export atlas… |  | same |  |
| 124 | › Import atlas… | › Import atlas… |  | same |  |
| 125 | › ——— | › ——— |  | same |  |
| 126 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 127 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 128 | › [check] Grid | › [check] Grid |  | same |  |
| 129 | › [check] Colors | › [check] Colors |  | same |  |
| 130 | › [check] Labels | › [check] Labels |  | same |  |
| 131 | › ——— | › ——— |  | same |  |
| 132 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 133 | **MEMORY** | **MEMORY** |  | same |  |
| 134 | [submenu] Undo history | [submenu] Undo history |  | same |  |
| 135 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 136 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 137 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 138 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 139 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 140 | › ——— | › ——— |  | same |  |
| 141 | › Clear undo history now | › Clear undo history now |  | same |  |
| 142 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 143 | Clear caches… | Clear caches… |  | same |  |
| 144 | **APPLICATION** | **APPLICATION** |  | same |  |
| 145 | Storage locations… | Storage locations… |  | same |  |
| 146 | Assistant… | Assistant… |  | same |  |
| 147 | Quick bar… | Quick bar… |  | same |  |
| 148 | [submenu] Theme | [submenu] Theme |  | same |  |
| 149 | › [radio] Dark | › [radio] Dark |  | same |  |
| 150 | › [radio] Light | › [radio] Light |  | same |  |
| 151 | › [radio] Follow system | › [radio] Follow system |  | same |  |
| 152 | [submenu] Units | [submenu] Units |  | same |  |
| 153 | › [radio] Kilometres | › [radio] Kilometres |  | same |  |
| 154 | › [radio] Miles | › [radio] Miles |  | same |  |
| 155 | › [radio] Nautical miles | › [radio] Nautical miles |  | same |  |
| 156 | [submenu] Handedness | [submenu] Handedness |  | same |  |
| 157 | › [radio] Left | › [radio] Left |  | same |  |
| 158 | › [radio] Right | › [radio] Right |  | same |  |
| 159 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |
| 160 | **TOUCH · NEW** |  |  | mockup-only | n N1 |
| 161 | Touch & gestures… |  |  | mockup-only | n N1 |

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
| 20 | › [submenu] Force tier | › [submenu] Force tier |  | same |  |
| 21 | › › [radio] Automatic (as measured) | › › [radio] Automatic (as measured) |  | same |  |
| 22 | › › [radio] Off | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 23 | › › [radio] Basic | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 24 | › › [radio] Standard | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 25 | › › [radio] Full | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 26 | ——— | ——— |  | same |  |
| 27 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 28 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 29 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 30 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 31 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 32 | › [readout] No windows open |  |  | mockup-only | s S3 |
| 33 | › [check] Context panel | › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 34 |  | › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 35 | ——— | ——— |  | same |  |
| 36 | Reset layout | Reset layout |  | same |  |
| 37 | [submenu] Layouts | [submenu] Layouts |  | same |  |
| 38 | › Default (reset layout) | › Default (reset layout) |  | same |  |
| 39 | › Generate | › Generate |  | same |  |
| 40 | › Sculpt | › Sculpt |  | same |  |
| 41 | › Cartography | › Cartography |  | same |  |
| 42 | › Journey | › Journey |  | same |  |
| 43 | › ——— | › ——— |  | same |  |
| 44 | › Save layout as… | › Save layout as… |  | same |  |
| 45 | › Forget layout… | › Forget layout… |  | same |  |

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
