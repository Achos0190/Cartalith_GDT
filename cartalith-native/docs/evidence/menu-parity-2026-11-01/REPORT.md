# Menu parity: Godot against the mobile-shell mockup (2026-11-01)

Owner, verbatim: *"check the godot version of all menu's against the mockups in the artifact. Per platform and
option if they are actually the same."*

Evidence only. It records what differs and why. It is not a status record: how far any milestone has got is
`cartalith-native/docs/STATUS.md`'s answer.

## What was compared

- **Mockup:** `mockup_menus.json`. `mockup_menus.js` evaluates the mockup's own `menuTree()`, `tabletOverflow()` and
  `tabletWorldMenu()` from `design/mobile-shell-2026-10-31/Shell.dc.html` (last changed in `fcc4b6f`). Re-run on
  2026-11-01, it reproduced the committed file exactly.
- **Godot:** `godot_menus.json`, written by `godot_menus.py`. It covers the shipped shell at `6295e11`, with these
  drafts applied in memory: `drafts/pc-shell/` 01-06 and their EDITs; `drafts/mobile-shell/` 05 and 08; and mobile 03's
  EDIT H1. This session has no Godot binary, so `godot_menus.py` interprets the menu builders. Every assumption it makes
  is in its `CONDITIONS` and `RUNTIME` tables (a fresh install, 8 logical cores). The interpreter left **0** conditions
  unresolved on every platform, in both states.
- **Per platform:**
  - **desktop:** `build()`'s eight menus.
  - **tablet ≥ 1 180 dp:** draft 08's whole bar.
  - **tablet < 1 180 dp:** the shipped bar, `☰ · File · World · Story`.
  - **phone:** MENU, which is draft 05's root over `phone_menu.gd`'s screens and the phone's own popups.
- **Two Godot states:**
  - **today:** PC-1, PC-4, PC-5 and PC-6 not built, so each row takes its drafted fallback.
  - **after:** those four milestones built.

  Rows are compared in the after state. The *Godot today* column is filled only where today's form differs.
- **Per row:** label, kind, accelerator (not on the phone, which draws none), disabled state and its reason,
  Session-mode gate, and submenu contents. A run-time list is compared by its pattern. On the phone, a radio submenu
  drawn as chips is compared by its options, not its shape.

Commands behind every count below:

```
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/godot_menus.py   # rows per platform and state; unresolved 0
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/compare.py       # this file; exits 1 if a difference is unclassified
```

## Totals

*mockup rows* and *Godot rows* count each side's rows once, at every depth (on the phone, the root's eleven included; the mockup's three separators inside rows Godot draws as chips are not compared). *rows* counts the tables' rows: a run-time Godot row paired with several of the mockup's sample rows counts once per sample. Each row that is not *same* counts once more, under its class. Class (b) has no rows: its fixes are already applied on the Godot side.

| Platform | mockup rows | Godot rows | rows | same | same after its milestone | differs | mockup-only | Godot-only | a | c | m | s | n |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Desktop | 434 | 429 | 435 | 399 | 12 | 22 | 1 | 1 | 0 | 0 | 12 | 24 | 0 |
| Tablet, 1 180 dp and wider | 437 | 428 | 438 | 398 | 12 | 22 | 5 | 1 | 0 | 0 | 12 | 24 | 4 |
| Tablet, below 1 180 dp | 452 | 443 | 453 | 413 | 12 | 22 | 5 | 1 | 0 | 0 | 12 | 24 | 4 |
| Phone, MENU | 442 | 449 | 486 | 380 | 5 | 24 | 33 | 44 | 0 | 73 | 5 | 26 | 2 |

Rows per rule, per platform:

| Rule | Desktop | Tablet, 1 180 dp and wider | Tablet, below 1 180 dp | Phone, MENU |
|---|---:|---:|---:|---:|
| R1 | 18 | 18 | 18 | 18 |
| R2 | 1 | 1 | 1 | 1 |
| P1 | 0 | 0 | 0 | 64 |
| P2 | 0 | 0 | 0 | 9 |
| S1 | 3 | 3 | 3 | 3 |
| S2 | 0 | 0 | 0 | 4 |
| N1 | 0 | 2 | 2 | 2 |
| N2 | 0 | 1 | 1 | 0 |
| N3 | 0 | 1 | 1 | 0 |
| PC-1 | 2 | 2 | 2 | 2 |
| PC-4 | 6 | 6 | 6 | 1 |
| PC-5 | 2 | 2 | 2 | 1 |
| PC-6 | 2 | 2 | 2 | 1 |
| S3 | 2 | 2 | 2 | 0 |

## Run-time labels (same at run time)

- **R1. Landmark types: a kind the engine cannot place.** The static read sees the build-time label (the kind's name); `menus.gd::_refresh_landmark_menu()` appends `   not buildable` when the menu opens, which is what the mockup draws since A1's fix.
- **R2. Atlas cache: the store readout.** The static read sees the placeholder `… chunks · … · empty`; `_refresh_atlas_cache_menu()` writes the value (`%d chunk%s · <size> · to LOD n`), which is what the mockup draws since A3's fix.

## (a) The mockup is wrong: the fixes (all eleven made in `Shell.dc.html` on 2026-11-01; the counts above are after them)

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

## (b) Drafts this check corrected

- **B1. `drafts/mobile-shell/08_menus_tablet_bar.gd`, `build()`.** Its whole-bar World was `_tablet_world if DccTheme.is_tablet() else _data`. That dropped the PC drafts' Generate world from the desktop, and it repeated Edit's two Reset rows in the wide tablet's World. It is now `_pc_world` (Ruling DU Q7, *the eight menus whole*). The header lists where each narrow-World row lives instead; README row 8 adds verify steps. `pc-shell/02` EDIT W1 and its README row now say to skip W1 when 08 is applied.
- **B2. `drafts/mobile-shell/05_phone_menu_root.gd`.** Three fixes. (1) `story-root` drew the old MORE section, which opened with a `Story` head and ended with an empty `World` head; `_fill_story_section()` is replaced. (2) The `MENU_SCREENS` comment claimed File's own order; it is corrected. (3) `_fill_popup()` dropped a leading captioned separator, which lost FOR THE TABLE, SOURCES and ACTIVE PACK on the phone; it is replaced, with only that test changed. README row 5 is updated.
- **B3. `drafts/mobile-shell/03_dcc_shell_phone_tabs.gd`.** Draft 03 renames the tabs (MS-2) and names the documents to correct with them, but not the two strings the app draws. EDIT H1 is added: MENU ▸ Help's closing line becomes *…reachable through MAP · DOMAIN · STORY · MENU.*, the mockup's own About wording. H2 flags the bottom-bar coach mark and the two probes that pin it; its wording is MS-2's to write. `check_symbols.sh` gains an anchor check for both strings, and README row 3 is updated.
- **B4. `drafts/mobile-shell/check_symbols.sh`.** The need list gains `_fill_popup _band _popup_row _clean STORY_ROWS`: 93 calls, up from 88. The anchor section is new.

## (c) Owner decisions

- **K1. PQ10: where the card's and the cartwheel's rows live.** The drafts put Edit ▸ Context card (Shift+F10) and Cartwheel at the pointer (Q) after Find on map… on the desktop and both tablets. The mockup has neither row. *Default: as drafted.* If it is taken, add both rows to the mockup's Edit.
- **K2. PQ11: one label for the regenerate command.** The command is Generate world on the PC and the wide tablet, and Run pipeline on the narrow tablet; it is the same command, id 710. It bears on A11. *Default: keep two labels.*
- **K3. PQ12: Alt+← / Alt+→ while a text field has focus.** It bears on Edit ▸ Previous / Next selection; the rows match. *Default: yes, as a browser does.*
- **P1. Phone: MENU's File, Assets, Preferences and Help open the phone screens, which reorder the rows and add some.** Ruling DT Q2: *the eight desktop menus row for row, in the same order and under the same names, plus Quick and Status*. The mockup's MENU follows it, and its band reads *same names, same order*. Draft 05 keeps `_fill_project()`, `_fill_assets()`, `_fill_prefs()` and `_fill_help()`. These draw:
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
- **P2. Phone: Window ▸ Context panel.** The mockup draws the Context panel check and its submenu (Inspect, Note, Ask, Pin, Follow) on the phone. `pc-shell/05` keeps `Right dock` and no submenu there. It reads `PC_SHELL_PROPOSAL.md` §3.11 (*the phone gets the model, not the panel*). The same section also gives the phone *the menu rows*. *Default: as drafted.* If it is taken, the mockup hides both Context panel rows on the phone and draws `Right dock`.

## Same in substance (drawing only)

- **S1. Disabled-row reasons: Anti-aliasing · anisotropy, 3D viewport defaults, Help ▸ Documentation.** The mockup's reasons are short forms of Godot's tooltips, and they make the same claim.
- **S2. Phone: plain separators inside a menu screen.** `_rest_of()` skips an unlabelled separator, and `_add()` draws a hairline between every row. The mockup's phone draws a gap. Captions are kept on both sides.

## Not drafted: the mockup's design, which no draft builds yet

- **N1. Preferences ▸ TOUCH · NEW ▸ Touch & gestures… (tablet and phone).** This is the proposal's Touch & gestures window. It holds Ruling DU Q5's switch, the hold durations and, per `MOBILE_SHELL_PROPOSAL.md`, the UI scale. `MOBILE_SHELL_MS1_SPEC.md` §3.6 specifies UI scale as its own `Preferences ▸ UI scale` submenu on touch builds, and leaves the panel to later work. On the phone, Godot has `TOUCH ▸ Gesture reference`, the read-only half.
- **N2. Tablet: Help ▸ Gesture reference….** The gestures screen exists only in `phone_menu.gd` (`_fill_gestures()`).
- **N3. Tablet: Preferences ▸ Open panels in their own window, disabled (DESKTOP ONLY).** PW-5 (*disabled-with-reason on tablet*) is unbuilt. Godot omits the row there today.

## Same after its milestone

- **PC-1. Edit ▸ Previous / Next selection.** A `_todo` row today; live, with Alt+← / Alt+→, from PC-1.
- **PC-4. Story ▸ Political history of the selection; Window ▸ Context panel (the region row) and its Inspect, Pin and Follow rows.** The shipped label or a `_todo` today; the mockup's row from PC-4.
- **PC-5. Story ▸ Note for the selection; Window ▸ Context panel ▸ Note.** As above, from PC-5.
- **PC-6. Story ▸ Ask the assistant about the selection (Ctrl+J); Window ▸ Context panel ▸ Ask.** As above, from PC-6.

## The tables

Columns: the mockup's row; Godot's row with PC-1, PC-4, PC-5 and PC-6 built; Godot today, shown only where it differs; the status; the class and rule. `›` marks one level of submenu. Marks: `[kind]`; `· Ctrl+…` the accelerator; `· dark` disabled when built, as Godot builds it; `· SM` disabled in Session mode; `· run-time list` a row repeated at run time (its text is the expression that makes it).

### Desktop

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
| 40 | [signpost] imports live under World ▸ Import | [signpost] imports live under World ▸ Import |  | same |  |
| 41 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

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

#### World

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Generate world · Ctrl+R · SM | Generate world · Ctrl+R · SM |  | same |  |
| 2 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 3 | **RECORDS** | **RECORDS** |  | same |  |
| 4 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 5 | Places… | Places… |  | same |  |
| 6 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 7 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 8 | Places without notes… | Places without notes… |  | same |  |
| 9 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 10 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 11 | World data tables… | World data tables… |  | same |  |
| 12 | [submenu] Import | [submenu] Import |  | same |  |
| 13 | › Maps   tiles | › Maps   tiles |  | same |  |
| 14 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 15 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 16 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 17 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 18 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 19 | › **SOURCES** | › **SOURCES** |  | same |  |
| 20 | › External Sources | › External Sources |  | same |  |
| 21 | › Connected Sources | › Connected Sources |  | same |  |
| 22 | › Source Registry | › Source Registry |  | same |  |
| 23 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 24 | › Definitions | › Definitions |  | same |  |
| 25 | › Check Data | › Check Data |  | same |  |
| 26 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 27 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

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
| 15 | [signpost] Places without notes is in World ▸ Records | [signpost] Places without notes is in World ▸ Records |  | same |  |
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
| 10 | [check] Timeline | [check] Timeline |  | same |  |
| 11 | [check] Status bar | [check] Status bar |  | same |  |
| 12 | [check] Domain rail | [check] Domain rail |  | same |  |
| 13 | ——— | ——— |  | same |  |
| 14 | [check] Session mode · Ctrl+Shift+M | [check] Session mode · Ctrl+Shift+M |  | same |  |
| 15 | [check] Diagnostics overlay · Shift+D | [check] Diagnostics overlay · Shift+D |  | same |  |
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
| 3 | Credits & academic principles | Credits & academic principles |  | same |  |
| 4 | Generation info… | Generation info… |  | same |  |
| 5 | Save diagnostic report… | Save diagnostic report… |  | same |  |
| 6 | About | About |  | same |  |

### Tablet, 1 180 dp and wider

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
| 40 | [signpost] imports live under World ▸ Import | [signpost] imports live under World ▸ Import |  | same |  |
| 41 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

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

#### World

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Generate world · Ctrl+R · SM | Generate world · Ctrl+R · SM |  | same |  |
| 2 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 3 | **RECORDS** | **RECORDS** |  | same |  |
| 4 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 5 | Places… | Places… |  | same |  |
| 6 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 7 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 8 | Places without notes… | Places without notes… |  | same |  |
| 9 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 10 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 11 | World data tables… | World data tables… |  | same |  |
| 12 | [submenu] Import | [submenu] Import |  | same |  |
| 13 | › Maps   tiles | › Maps   tiles |  | same |  |
| 14 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 15 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 16 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 17 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 18 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 19 | › **SOURCES** | › **SOURCES** |  | same |  |
| 20 | › External Sources | › External Sources |  | same |  |
| 21 | › Connected Sources | › Connected Sources |  | same |  |
| 22 | › Source Registry | › Source Registry |  | same |  |
| 23 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 24 | › Definitions | › Definitions |  | same |  |
| 25 | › Check Data | › Check Data |  | same |  |
| 26 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 27 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

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
| 15 | [signpost] Places without notes is in World ▸ Records | [signpost] Places without notes is in World ▸ Records |  | same |  |
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
| 10 | [check] Timeline | [check] Timeline |  | same |  |
| 11 | [check] Status bar | [check] Status bar |  | same |  |
| 12 | [check] Domain rail | [check] Domain rail |  | same |  |
| 13 | ——— | ——— |  | same |  |
| 14 | [check] Session mode · Ctrl+Shift+M | [check] Session mode · Ctrl+Shift+M |  | same |  |
| 15 | [check] Diagnostics overlay · Shift+D | [check] Diagnostics overlay · Shift+D |  | same |  |
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
| 309 | › [check] Timeline | › [check] Timeline |  | same |  |
| 310 | › [check] Status bar | › [check] Status bar |  | same |  |
| 311 | › [check] Domain rail | › [check] Domain rail |  | same |  |
| 312 | › ——— | › ——— |  | same |  |
| 313 | › [check] Session mode · Ctrl+Shift+M | › [check] Session mode · Ctrl+Shift+M |  | same |  |
| 314 | › [check] Diagnostics overlay · Shift+D | › [check] Diagnostics overlay · Shift+D |  | same |  |
| 315 | › [check] 3D view | › [check] 3D view |  | same |  |
| 316 | › [submenu] 3D capability | › [submenu] 3D capability |  | same |  |
| 317 | › › [readout] 3D tier | › › [readout] 3D tier: not tested |  | same |  |
| 318 | › › Re-test 3D | › › Re-test 3D |  | same |  |
| 319 | › › [submenu] Force tier | › › [submenu] Force tier |  | same |  |
| 320 | › › › [radio] Automatic (as measured) | › › › [radio] Automatic (as measured) |  | same |  |
| 321 | › › › [radio] Off | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 322 | › › › [radio] Basic | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 323 | › › › [radio] Standard | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 324 | › › › [radio] Full | › › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 325 | › ——— | › ——— |  | same |  |
| 326 | › [submenu] Workspace | › [submenu] Workspace |  | same |  |
| 327 | › › [radio] World · SM | › › [radio] World · SM |  | same |  |
| 328 | › › [radio] Civilization | › › [radio] Civilization |  | same |  |
| 329 | › › [radio] Cartography | › › [radio] Cartography |  | same |  |
| 330 | › [submenu] Open windows | › [submenu] Open windows |  | same |  |
| 331 | › › [readout] No windows open |  |  | mockup-only | s S3 |
| 332 | › › [check] Context panel | › › [check] Context panel |  | same after the milestone; today: absent today | m PC-4 |
| 333 |  | › › [check] {w.title.strip_edges()} · run-time list |  | Godot-only | s S3 |
| 334 | › ——— | › ——— |  | same |  |
| 335 | › Reset layout | › Reset layout |  | same |  |
| 336 | › [submenu] Layouts | › [submenu] Layouts |  | same |  |
| 337 | › › Default (reset layout) | › › Default (reset layout) |  | same |  |
| 338 | › › Generate | › › Generate |  | same |  |
| 339 | › › Sculpt | › › Sculpt |  | same |  |
| 340 | › › Cartography | › › Cartography |  | same |  |
| 341 | › › Journey | › › Journey |  | same |  |
| 342 | › › ——— | › › ——— |  | same |  |
| 343 | › › Save layout as… | › › Save layout as… |  | same |  |
| 344 | › › Forget layout… | › › Forget layout… |  | same |  |
| 345 | [submenu] Help | [submenu] Help |  | same |  |
| 346 | › [not built] Documentation | › [not built] Documentation |  | differs: reason | s S1 |
| 347 | › Gesture reference… |  |  | mockup-only | n N2 |
| 348 | › Credits & academic principles | › Credits & academic principles |  | same |  |
| 349 | › Generation info… | › Generation info… |  | same |  |
| 350 | › Save diagnostic report… | › Save diagnostic report… |  | same |  |

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
| 40 | [signpost] imports live under World ▸ Import | [signpost] imports live under World ▸ Import |  | same |  |
| 41 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

#### World

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Run pipeline · Ctrl+R · SM | Run pipeline · Ctrl+R · SM |  | same |  |
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
| 20 | ——— | ——— |  | same |  |
| 21 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 22 | **RECORDS** | **RECORDS** |  | same |  |
| 23 | Factions… · Ctrl+Shift+F | Factions… · Ctrl+Shift+F |  | same |  |
| 24 | Places… | Places… |  | same |  |
| 25 | Markdown vault… · Ctrl+Shift+V | Markdown vault… · Ctrl+Shift+V |  | same |  |
| 26 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 27 | Places without notes… | Places without notes… |  | same |  |
| 28 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 29 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 30 | World data tables… | World data tables… |  | same |  |
| 31 | [submenu] Import | [submenu] Import |  | same |  |
| 32 | › Maps   tiles | › Maps   tiles |  | same |  |
| 33 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 34 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 35 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 36 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 37 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 38 | › **SOURCES** | › **SOURCES** |  | same |  |
| 39 | › External Sources | › External Sources |  | same |  |
| 40 | › Connected Sources | › Connected Sources |  | same |  |
| 41 | › Source Registry | › Source Registry |  | same |  |
| 42 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 43 | › Definitions | › Definitions |  | same |  |
| 44 | › Check Data | › Check Data |  | same |  |
| 45 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 46 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |

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
| 15 | [signpost] Places without notes is in World ▸ Records | [signpost] Places without notes is in World ▸ Records |  | same |  |
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
| 6 | [submenu] World | [submenu] World |  | same |  |
| 7 | [submenu] Story | [submenu] Story |  | same |  |
| 8 | [submenu] Preferences | [submenu] Preferences |  | same |  |
| 9 | [submenu] Window | [submenu] Window |  | same |  |
| 10 | [submenu] Help | [submenu] Help |  | same |  |
| 11 | [status] STATUS (world, state, last autosave) | [status] STATUS (world, state, last autosave) |  | same |  |

#### File

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 |  | Save project |  | Godot-only | (c) P1 |
| 2 |  | Save as… |  | Godot-only | (c) P1 |
| 3 | New world… · SM | New world… · SM |  | same |  |
| 4 | Open project… | Open project… |  | same |  |
| 5 | [submenu] Recent worlds |  |  | mockup-only | (c) P1 |
| 6 | › VHAREN REACH — 129384 · 5 d ago |  |  | mockup-only (inside) | (c) P1 |
| 7 | › KESSA — 774201 · 3 w ago |  |  | mockup-only (inside) | (c) P1 |
| 8 | › THORNWOOD — 908133 · 2 mo ago |  |  | mockup-only (inside) | (c) P1 |
| 9 |  | **RECENT WORLDS** |  | Godot-only | (c) P1 |
| 10 |  | {meta.get('name', '')} · run-time list |  | Godot-only | (c) P1 |
| 11 |  | **AUTOSAVE** |  | Godot-only | (c) P1 |
| 12 |  | [check] Autosave |  | Godot-only | (c) P1 |
| 13 |  | [chips] Interval |  | Godot-only | (c) P1 |
| 14 |  | › [radio] Off |  | Godot-only (inside) | (c) P1 |
| 15 |  | › [radio] Every 1 min |  | Godot-only (inside) | (c) P1 |
| 16 |  | › [radio] Every 5 min |  | Godot-only (inside) | (c) P1 |
| 17 |  | › [radio] Every 15 min |  | Godot-only (inside) | (c) P1 |
| 18 | New world from selection… · SM | New world from selection… · SM |  | same |  |
| 19 | ——— |  |  | mockup-only | s S2 |
| 20 | Save project |  |  | mockup-only | (c) P1 |
| 21 | Save as… |  |  | mockup-only | (c) P1 |
| 22 | [check] Autosave |  |  | mockup-only | (c) P1 |
| 23 | [submenu] Autosave interval |  |  | mockup-only | (c) P1 |
| 24 | › [radio] Off |  |  | mockup-only (inside) | (c) P1 |
| 25 | › [radio] Every 1 min |  |  | mockup-only (inside) | (c) P1 |
| 26 | › [radio] Every 5 min |  |  | mockup-only (inside) | (c) P1 |
| 27 | › [radio] Every 15 min |  |  | mockup-only (inside) | (c) P1 |
| 28 | Revert to last save | Revert to last save |  | same |  |
| 29 | Close project | Close project |  | same |  |
| 30 | ——— |  |  | mockup-only | s S2 |
| 31 | [submenu] Export | [submenu] Export |  | same |  |
| 32 | › **FOR THE TABLE** | › **FOR THE TABLE** |  | same |  |
| 33 | › Player map… | › Player map… |  | same |  |
| 34 | › DM / full map… | › DM / full map… |  | same |  |
| 35 | › **SHARE** | › **SHARE** |  | same |  |
| 36 | › Export map image… | › Export map image… |  | same |  |
| 37 | › [not built] Print sheet (PDF)… | › [not built] Print sheet (PDF)… |  | same |  |
| 38 | › [not built] VTT battle map (grid + walls)… | › [not built] VTT battle map (grid + walls)… |  | same |  |
| 39 | › **DATA** | › **DATA** |  | same |  |
| 40 | › Maps   tiles | › Maps   tiles |  | same |  |
| 41 | › GIS / GeoJSON   .geojson | › GIS / GeoJSON   .geojson |  | same |  |
| 42 | › World Data   map + atlas | › World Data   map + atlas |  | same |  |
| 43 | › Assets   .zip | › Assets   .zip |  | same |  |
| 44 | **STORAGE LOCATIONS** | **STORAGE LOCATIONS** |  | same |  |
| 45 | [readout] projects | [readout] projects   … |  | same |  |
| 46 | [readout] tile atlas cache | [readout] tile atlas cache   … |  | same |  |
| 47 | [readout] asset packs | [readout] asset packs   … |  | same |  |
| 48 | [readout] exports | [readout] exports   … |  | same |  |
| 49 | Change locations… | Change locations… |  | same |  |
| 50 | Show project on disk | Show project on disk |  | same |  |
| 51 | [signpost] imports live under World ▸ Import | [signpost] imports live under World ▸ Import |  | same |  |
| 52 | [signpost] asset packs under Assets | [signpost] asset packs under Assets |  | same |  |

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
| 1 |  | Splat channels |  | Godot-only | (c) P1 |
| 2 |  | Biome textures |  | Godot-only | (c) P1 |
| 3 |  | Terrain textures |  | Godot-only | (c) P1 |
| 4 |  | Feature icons |  | Godot-only | (c) P1 |
| 5 |  | Settlement pins |  | Godot-only | (c) P1 |
| 6 |  | Settlement traits |  | Godot-only | (c) P1 |
| 7 |  | Points of interest |  | Godot-only | (c) P1 |
| 8 |  | Sea marks |  | Godot-only | (c) P1 |
| 9 |  | Custom icons |  | Godot-only | (c) P1 |
| 10 |  | **PACK** |  | Godot-only | (c) P1 |
| 11 |  | [readout] Name |  | Godot-only | (c) P1 |
| 12 |  | [readout] Author |  | Godot-only | (c) P1 |
| 13 |  | [readout] Licence |  | Godot-only | (c) P1 |
| 14 |  | [readout] Stored items |  | Godot-only | (c) P1 |
| 15 | ⧉ Asset library | ⧉ Asset library |  | same |  |
| 16 | ⧉ Sprite sheet slicer (▦) | ⧉ Sprite sheet slicer (▦) |  | same |  |
| 17 | Import asset pack .zip… | Import asset pack .zip… |  | same |  |
| 18 | Open pack in library… | Open pack in library… |  | same |  |
| 19 | Pack report… | Pack report… |  | same |  |
| 20 | Import folder of art… | Import folder of art… |  | same |  |
| 21 | Compare pack with library… | Compare pack with library… |  | same |  |
| 22 | ——— |  |  | mockup-only | s S2 |
| 23 | [submenu] Asset pack | [submenu] Asset pack |  | same |  |
| 24 | › **ACTIVE PACK** | › **ACTIVE PACK** |  | same |  |
| 25 | › [readout] NAME | › [readout] NAME   — loading — |  | same |  |
| 26 | › [readout] AUTHOR | › [readout] AUTHOR   — loading — |  | same |  |
| 27 | › [readout] LICENSE | › [readout] LICENSE   — loading — |  | same |  |
| 28 | › [readout] SCHEMA | › [readout] SCHEMA   — loading — |  | same |  |
| 29 | › [readout] FILLED | › [readout] FILLED   — loading — |  | same |  |
| 30 | › Pack metadata… | › Pack metadata… |  | same |  |
| 31 | › Validate pack | › Validate pack |  | same |  |
| 32 | › Export pack .zip… | › Export pack .zip… |  | same |  |
| 33 | [submenu] Icon families | [submenu] Icon families |  | same |  |
| 34 | › Feature icons   22/31 | › Feature icons |  | same |  |
| 35 | › Settlement pins   8/10 | › Settlement pins |  | same |  |
| 36 | › Settlement traits   5/12 | › Settlement traits |  | same |  |
| 37 | › Points of interest   14/21 | › Points of interest |  | same |  |
| 38 | › Sea marks   4/9 | › Sea marks |  | same |  |
| 39 | › Custom icons   0/6 | › Custom icons |  | same |  |
| 40 | [submenu] Texture sets | [submenu] Texture sets |  | same |  |
| 41 | › Splat channels   4/4 | › Splat channels |  | same |  |
| 42 | › Biome textures   9/12 | › Biome textures |  | same |  |
| 43 | › Terrain textures   6/8 | › Terrain textures |  | same |  |
| 44 | [submenu] Landmark types | [submenu] Landmark types |  | same |  |
| 45 | › [submenu] Physical | › [submenu] Physical |  | same |  |
| 46 | › › Peak | › › Peak |  | same |  |
| 47 | › › Ridge | › › Ridge |  | same |  |
| 48 | › › Saddle   not buildable | › › Saddle |  | differs: label | s R1 |
| 49 | › › Cliff | › › Cliff |  | same |  |
| 50 | › › Gorge | › › Gorge |  | same |  |
| 51 | › › Cave | › › Cave |  | same |  |
| 52 | › › Waterfall | › › Waterfall |  | same |  |
| 53 | › › Spring | › › Spring |  | same |  |
| 54 | › › Lake | › › Lake |  | same |  |
| 55 | › › Delta   not buildable | › › Delta |  | differs: label | s R1 |
| 56 | › › River confluence | › › River confluence |  | same |  |
| 57 | › › Volcanic feature | › › Volcanic feature |  | same |  |
| 58 | › › Rock formation | › › Rock formation |  | same |  |
| 59 | › › Glacial feature   not buildable | › › Glacial feature |  | differs: label | s R1 |
| 60 | › › Ancient forest   not buildable | › › Ancient forest |  | differs: label | s R1 |
| 61 | › › ——— | › › ——— |  | same |  |
| 62 | › › Open Physical in the dock | › › Open Physical in the dock |  | same |  |
| 63 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 64 | › [submenu] Transport | › [submenu] Transport |  | same |  |
| 65 | › › Mountain pass | › › Mountain pass |  | same |  |
| 66 | › › River crossing   not buildable | › › River crossing |  | differs: label | s R1 |
| 67 | › › Ford | › › Ford |  | same |  |
| 68 | › › Bridge site | › › Bridge site |  | same |  |
| 69 | › › Road junction | › › Road junction |  | same |  |
| 70 | › › Caravan station | › › Caravan station |  | same |  |
| 71 | › › Portage | › › Portage |  | same |  |
| 72 | › › Harbour | › › Harbour |  | same |  |
| 73 | › › ——— | › › ——— |  | same |  |
| 74 | › › Open Transport in the dock | › › Open Transport in the dock |  | same |  |
| 75 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 76 | › [submenu] Economic | › [submenu] Economic |  | same |  |
| 77 | › › Mine | › › Mine |  | same |  |
| 78 | › › Quarry | › › Quarry |  | same |  |
| 79 | › › Salt works   not buildable | › › Salt works |  | differs: label | s R1 |
| 80 | › › Resource extraction site | › › Resource extraction site |  | same |  |
| 81 | › › Market site | › › Market site |  | same |  |
| 82 | › › Trade depot | › › Trade depot |  | same |  |
| 83 | › › ——— | › › ——— |  | same |  |
| 84 | › › Open Economic in the dock | › › Open Economic in the dock |  | same |  |
| 85 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 86 | › [submenu] Military | › [submenu] Military |  | same |  |
| 87 | › › Fort | › › Fort |  | same |  |
| 88 | › › Watchtower | › › Watchtower |  | same |  |
| 89 | › › Fortified pass | › › Fortified pass |  | same |  |
| 90 | › › Fortified crossing | › › Fortified crossing |  | same |  |
| 91 | › › Battlefield | › › Battlefield |  | same |  |
| 92 | › › Border marker | › › Border marker |  | same |  |
| 93 | › › ——— | › › ——— |  | same |  |
| 94 | › › Open Military in the dock | › › Open Military in the dock |  | same |  |
| 95 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 96 | › [submenu] Cultural | › [submenu] Cultural |  | same |  |
| 97 | › › Shrine   not buildable | › › Shrine |  | differs: label | s R1 |
| 98 | › › Temple   not buildable | › › Temple |  | differs: label | s R1 |
| 99 | › › Sacred grove   not buildable | › › Sacred grove |  | differs: label | s R1 |
| 100 | › › Sacred mountain   not buildable | › › Sacred mountain |  | differs: label | s R1 |
| 101 | › › Pilgrimage site   not buildable | › › Pilgrimage site |  | differs: label | s R1 |
| 102 | › › Tomb   not buildable | › › Tomb |  | differs: label | s R1 |
| 103 | › › Monument   not buildable | › › Monument |  | differs: label | s R1 |
| 104 | › › Ceremonial site   not buildable | › › Ceremonial site |  | differs: label | s R1 |
| 105 | › › ——— | › › ——— |  | same |  |
| 106 | › › Open Cultural in the dock | › › Open Cultural in the dock |  | same |  |
| 107 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 108 | › [submenu] Historical | › [submenu] Historical |  | same |  |
| 109 | › › Ruin   not buildable | › › Ruin |  | differs: label | s R1 |
| 110 | › › Abandoned settlement | › › Abandoned settlement |  | same |  |
| 111 | › › Ancient road | › › Ancient road |  | same |  |
| 112 | › › Historic battlefield   not buildable | › › Historic battlefield |  | differs: label | s R1 |
| 113 | › › Destroyed fortress   not buildable | › › Destroyed fortress |  | differs: label | s R1 |
| 114 | › › Historic crossing   not buildable | › › Historic crossing |  | differs: label | s R1 |
| 115 | › › ——— | › › ——— |  | same |  |
| 116 | › › Open Historical in the dock | › › Open Historical in the dock |  | same |  |
| 117 | › › [signpost] Rows here read state; they do not arm a type. | › › [signpost] Rows here read state; they do not arm a type. |  | same |  |
| 118 | › Placed by hand   3 | › Placed by hand |  | same |  |
| 119 | › ——— | › ——— |  | same |  |
| 120 | › Landmark icons… | › Landmark icons… |  | same |  |
| 121 | › Landmark label style… | › Landmark label style… |  | same |  |
| 122 | ——— |  |  | mockup-only | s S2 |
| 123 | Apply library to map | Apply library to map |  | same |  |
| 124 | Clear library… | Clear library…   destructive |  | same |  |

#### World

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | Generate world · SM | Generate world · SM |  | same |  |
| 2 | ⧉ Data manager | ⧉ Data manager |  | same |  |
| 3 | **RECORDS** | **RECORDS** |  | same |  |
| 4 | Factions… | Factions… |  | same |  |
| 5 | Places… | Places… |  | same |  |
| 6 | Markdown vault… | Markdown vault… |  | same |  |
| 7 | Browse & edit a note… | Browse & edit a note… |  | same |  |
| 8 | Places without notes… | Places without notes… |  | same |  |
| 9 | [signpost] Timeline is in Story ▸ Timeline | [signpost] Timeline is in Story ▸ Timeline |  | same |  |
| 10 | **TECHNICAL** | **TECHNICAL** |  | same |  |
| 11 | World data tables… | World data tables… |  | same |  |
| 12 | [submenu] Import | [submenu] Import |  | same |  |
| 13 | › Maps   tiles | › Maps   tiles |  | same |  |
| 14 | › Heightmaps   PNG | › Heightmaps   PNG |  | same |  |
| 15 | › GIS / GeoJSON | › GIS / GeoJSON |  | same |  |
| 16 | › World Data   .ctl | › World Data   .ctl |  | same |  |
| 17 | › Assets   → Assets | › Assets   → Assets |  | same |  |
| 18 | [submenu] Sources & validation | [submenu] Sources & validation |  | same |  |
| 19 | › **SOURCES** | › **SOURCES** |  | same |  |
| 20 | › External Sources | › External Sources |  | same |  |
| 21 | › Connected Sources | › Connected Sources |  | same |  |
| 22 | › Source Registry | › Source Registry |  | same |  |
| 23 | › **VALIDATION** | › **VALIDATION** |  | same |  |
| 24 | › Definitions | › Definitions |  | same |  |
| 25 | › Check Data | › Check Data |  | same |  |
| 26 | › Repair / Normalize | › Repair / Normalize |  | same |  |
| 27 | [signpost] Export is in File ▸ Export | [signpost] Export is in File ▸ Export |  | same |  |
| 28 |  | **ON THE PHONE** |  | Godot-only | (c) P1 |
| 29 |  | Civilization |  | Godot-only | (c) P1 |
| 30 |  | Data manager |  | Godot-only | (c) P1 |
| 31 |  | Travel library |  | Godot-only | (c) P1 |
| 32 |  | Simulation |  | Godot-only | (c) P1 |

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
| 15 | [signpost] Places without notes is in World ▸ Records | [signpost] Places without notes is in World ▸ Records |  | same |  |
| 16 | Ask the assistant about the selection | Ask the assistant about the selection | Assistant panel… | same after the milestone; today: label | m PC-6 |

#### Preferences

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 |  | **APPLICATION** |  | Godot-only | (c) P1 |
| 2 |  | [chips] Theme |  | Godot-only | (c) P1 |
| 3 |  | › [radio] Dark |  | Godot-only (inside) | (c) P1 |
| 4 |  | › [radio] Light |  | Godot-only (inside) | (c) P1 |
| 5 |  | › [radio] Follow system |  | Godot-only (inside) | (c) P1 |
| 6 |  | [chips] Units |  | Godot-only | (c) P1 |
| 7 |  | › [radio] Kilometres |  | Godot-only (inside) | (c) P1 |
| 8 |  | › [radio] Miles |  | Godot-only (inside) | (c) P1 |
| 9 |  | › [radio] Nautical miles |  | Godot-only (inside) | (c) P1 |
| 10 | **PERFORMANCE** | **PERFORMANCE** |  | same |  |
| 11 | [check] GPU acceleration | [check] GPU acceleration |  | same |  |
| 12 | [submenu] Devices | [submenu] Devices |  | same |  |
| 13 | › [check] Automatic (highest-performance GPU) | › [check] Automatic (highest-performance GPU) |  | same |  |
| 14 | › ——— | › ——— |  | same |  |
| 15 | › [check] Adreno 630 · integrated · Vulkan | › [check] … · … · … · run-time list |  | same (run-time list) |  |
| 16 | › ——— | › ——— |  | same |  |
| 17 | › [readout] Memory | › [readout] Memory: not measured yet |  | same |  |
| 18 | › ——— | › ——— |  | same |  |
| 19 | › Rescan devices… | › Rescan devices… |  | same |  |
| 20 | [submenu] Multi-GPU mode | [chips] Multi-GPU mode |  | same |  |
| 21 | › [radio] Single device | › [radio] Single device |  | same |  |
| 22 | › [radio] Split tiles | › [radio] Split tiles |  | same |  |
| 23 | › [not built] Alternate frames | › [radio] Alternate frames · dark |  | same |  |
| 24 | [submenu] CPU worker threads | [chips] CPU worker threads |  | same |  |
| 25 | › [radio] Automatic | › [radio] Automatic |  | same |  |
| 26 | › [radio] 1 — 1 thread | › [radio] 1 — 1 thread |  | same |  |
| 27 | › [radio] 2 — quarter | › [radio] 2 — quarter |  | same |  |
| 28 | › [radio] 4 — half | › [radio] 4 — half |  | same |  |
| 29 | › [radio] 8 — all cores | › [radio] 8 — all cores |  | same |  |
| 30 | › [readout] Running | › [readout] Running … workers now |  | same |  |
| 31 | [submenu] VRAM budget | [chips] VRAM budget |  | same |  |
| 32 | › [radio] No cap | › [radio] No cap |  | same |  |
| 33 | › [radio] 1 GB | › [radio] 1 GB |  | same |  |
| 34 | › [radio] 2 GB | › [radio] 2 GB |  | same |  |
| 35 | › [radio] 4 GB | › [radio] 4 GB |  | same |  |
| 36 | › [radio] 8 GB | › [radio] 8 GB |  | same |  |
| 37 | › [radio] 12 GB | › [radio] 12 GB |  | same |  |
| 38 | › [radio] 16 GB | › [radio] 16 GB |  | same |  |
| 39 | › [radio] 24 GB | › [radio] 24 GB |  | same |  |
| 40 | › [readout] Current grid 2048×2048 needs about | › [readout] Current grid …x… needs about … MB |  | same |  |
| 41 | [submenu] Fallback when VRAM full | [chips] Fallback when VRAM full |  | same |  |
| 42 | › [radio] CPU tile pass | › [radio] CPU tile pass |  | same |  |
| 43 | › [radio] Fail with error | › [radio] Fail with error |  | same |  |
| 44 | › [not built] Reduce working res | › [radio] Reduce working res · dark |  | same |  |
| 45 | Try the GPU again · dark | Try the GPU again · dark |  | same |  |
| 46 | **GRAPHICS** | **GRAPHICS** |  | same |  |
| 47 | [submenu] Render quality | [chips] Render quality |  | same |  |
| 48 | › [radio] performance | › [radio] performance |  | same |  |
| 49 | › [radio] balanced | › [radio] balanced |  | same |  |
| 50 | › [radio] quality | › [radio] quality |  | same |  |
| 51 | › [radio] ultra | › [radio] ultra |  | same |  |
| 52 | [not built] Anti-aliasing · anisotropy | [not built] Anti-aliasing · anisotropy |  | differs: reason | s S1 |
| 53 | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management | [signpost] Colour management — CARTO ▸ Colours ▸ Colour management |  | same |  |
| 54 | [not built] 3D viewport defaults | [not built] 3D viewport defaults |  | differs: reason | s S1 |
| 55 | [submenu] Lighting rig defaults | [submenu] Lighting rig defaults |  | same |  |
| 56 | › [readout] Following the engine | › [readout] Following the engine |  | same |  |
| 57 | › [submenu] Azimuth | › [submenu] Azimuth |  | same |  |
| 58 | › › [radio] 0 deg | › › [radio] 0 deg |  | same |  |
| 59 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 60 | › › [radio] 90 deg | › › [radio] 90 deg |  | same |  |
| 61 | › › [radio] 135 deg | › › [radio] 135 deg |  | same |  |
| 62 | › › [radio] 180 deg | › › [radio] 180 deg |  | same |  |
| 63 | › › [radio] 225 deg | › › [radio] 225 deg |  | same |  |
| 64 | › › [radio] 270 deg | › › [radio] 270 deg |  | same |  |
| 65 | › › [radio] 315 deg | › › [radio] 315 deg |  | same |  |
| 66 | › [submenu] Elevation | › [submenu] Elevation |  | same |  |
| 67 | › › [radio] 15 deg | › › [radio] 15 deg |  | same |  |
| 68 | › › [radio] 25 deg | › › [radio] 25 deg |  | same |  |
| 69 | › › [radio] 35 deg | › › [radio] 35 deg |  | same |  |
| 70 | › › [radio] 45 deg | › › [radio] 45 deg |  | same |  |
| 71 | › › [radio] 55 deg | › › [radio] 55 deg |  | same |  |
| 72 | › › [radio] 65 deg | › › [radio] 65 deg |  | same |  |
| 73 | › › [radio] 75 deg | › › [radio] 75 deg |  | same |  |
| 74 | › [submenu] Ambient | › [submenu] Ambient |  | same |  |
| 75 | › › [radio] 0.00 | › › [radio] 0.00 |  | same |  |
| 76 | › › [radio] 0.15 | › › [radio] 0.15 |  | same |  |
| 77 | › › [radio] 0.25 | › › [radio] 0.25 |  | same |  |
| 78 | › › [radio] 0.35 | › › [radio] 0.35 |  | same |  |
| 79 | › › [radio] 0.50 | › › [radio] 0.50 |  | same |  |
| 80 | › › [radio] 0.70 | › › [radio] 0.70 |  | same |  |
| 81 | › [submenu] Multidirectional | › [submenu] Multidirectional |  | same |  |
| 82 | › › [radio] 1 light (single sun) | › › [radio] 1 light (single sun) |  | same |  |
| 83 | › › [radio] 2 lights | › › [radio] 2 lights |  | same |  |
| 84 | › › [radio] 4 lights | › › [radio] 4 lights |  | same |  |
| 85 | › › [radio] 6 lights | › › [radio] 6 lights |  | same |  |
| 86 | › › [radio] 8 lights | › › [radio] 8 lights |  | same |  |
| 87 | › › [radio] 12 lights | › › [radio] 12 lights |  | same |  |
| 88 | › ——— | › ——— |  | same |  |
| 89 | › Use the engine’s own rig | › Use the engine's own rig |  | same |  |
| 90 | [submenu] Relief exaggeration | [chips] Relief exaggeration |  | same |  |
| 91 | › [radio] 1x | › [radio] 1x |  | same |  |
| 92 | › [radio] 2x | › [radio] 2x |  | same |  |
| 93 | › [radio] 4x | › [radio] 4x |  | same |  |
| 94 | › Use the engine’s own 3.4x | › Use the engine's own 3.4x |  | same |  |
| 95 | **TILES & LOD** | **TILES & LOD** |  | same |  |
| 96 | [submenu] Tiled LOD | [chips] Tiled LOD |  | same |  |
| 97 | › [radio] Auto on zoom | › [radio] Auto on zoom |  | same |  |
| 98 | › [radio] Manual | › [radio] Manual |  | same |  |
| 99 | › Enter deep detail now | › Enter deep detail now |  | same |  |
| 100 | › Leave deep detail | › Leave deep detail |  | same |  |
| 101 | [submenu] Tile size · LOD levels | [submenu] Tile size · LOD levels |  | same |  |
| 102 | › [radio] 256 px | › [radio] 256 px |  | same |  |
| 103 | › [radio] 512 px | › [radio] 512 px |  | same |  |
| 104 | › [radio] 1024 px | › [radio] 1024 px |  | same |  |
| 105 | › ——— | › ——— |  | same |  |
| 106 | › [submenu] LOD levels | › [submenu] LOD levels |  | same |  |
| 107 | › › [radio] LOD 0–0   1 tile | › › [radio] LOD 0–0   1 tile |  | same |  |
| 108 | › › [radio] LOD 0–1   5 tiles | › › [radio] LOD 0–1   5 tiles |  | same |  |
| 109 | › › [radio] LOD 0–2   21 tiles | › › [radio] LOD 0–2   21 tiles |  | same |  |
| 110 | › › [radio] LOD 0–3   85 tiles | › › [radio] LOD 0–3   85 tiles |  | same |  |
| 111 | › › [radio] LOD 0–4   341 tiles | › › [radio] LOD 0–4   341 tiles |  | same |  |
| 112 | › › [radio] LOD 0–5   1 365 tiles | › › [radio] LOD 0–5   1 365 tiles |  | same |  |
| 113 | › › [radio] LOD 0–6   5 461 tiles | › › [radio] LOD 0–6   5 461 tiles |  | same |  |
| 114 | › › [radio] LOD 0–7   21 845 tiles | › › [radio] LOD 0–7   21 845 tiles |  | same |  |
| 115 | › › [radio] LOD 0–8   87 381 tiles | › › [radio] LOD 0–8   87 381 tiles |  | same |  |
| 116 | [submenu] Atlas cache | [submenu] Atlas cache |  | same |  |
| 117 | › [readout] 85 chunks · 1.2 GB · to LOD 3 | › [readout] … chunks · … · empty |  | differs: label | s R2 |
| 118 | › [submenu] Size cap · GB | › [submenu] Size cap · GB |  | same |  |
| 119 | › › [radio] No cap | › › [radio] No cap |  | same |  |
| 120 | › › [radio] 0.5 GB | › › [radio] 0.5 GB |  | same |  |
| 121 | › › [radio] 1 GB | › › [radio] 1 GB |  | same |  |
| 122 | › › [radio] 2 GB | › › [radio] 2 GB |  | same |  |
| 123 | › › [radio] 4 GB | › › [radio] 4 GB |  | same |  |
| 124 | › › [radio] 8 GB | › › [radio] 8 GB |  | same |  |
| 125 | › › [radio] 16 GB | › › [radio] 16 GB |  | same |  |
| 126 | › Refine detail for the current view | › Refine detail for the current view |  | same |  |
| 127 | › Refine viewshed for the current view | › Refine viewshed for the current view |  | same |  |
| 128 | › [readout] Viewshed | › [readout] Viewshed: never refined |  | same |  |
| 129 | › ——— | › ——— |  | same |  |
| 130 | › Export atlas… | › Export atlas… |  | same |  |
| 131 | › Import atlas… | › Import atlas… |  | same |  |
| 132 | › ——— | › ——— |  | same |  |
| 133 | › Clear atlas cache now… | › Clear atlas cache now… |  | same |  |
| 134 | [submenu] Chunk debug overlay | [submenu] Chunk debug overlay |  | same |  |
| 135 | › [check] Grid | › [check] Grid |  | same |  |
| 136 | › [check] Colors | › [check] Colors |  | same |  |
| 137 | › [check] Labels | › [check] Labels |  | same |  |
| 138 | › ——— | › ——— |  | same |  |
| 139 | › [check] Show tile borders on the map | › [check] Show tile borders on the map |  | same |  |
| 140 | **MEMORY** | **MEMORY** |  | same |  |
| 141 | [submenu] Undo history | [chips] Undo history |  | same |  |
| 142 | › [radio] 64 MB | › [radio] 64 MB |  | same |  |
| 143 | › [radio] 128 MB | › [radio] 128 MB |  | same |  |
| 144 | › [radio] 256 MB | › [radio] 256 MB |  | same |  |
| 145 | › [radio] 512 MB | › [radio] 512 MB |  | same |  |
| 146 | › [radio] 1024 MB | › [radio] 1024 MB |  | same |  |
| 147 | › Clear undo history now | › Clear undo history now |  | same |  |
| 148 | [readout] Working set | [readout] Working set   … GB of … GB |  | same |  |
| 149 | Clear caches… | Clear caches… |  | same |  |
| 150 | **APPLICATION** | **APPLICATION** |  | same |  |
| 151 | Storage locations… | Storage locations… |  | same |  |
| 152 | Assistant… | Assistant… |  | same |  |
| 153 | Quick bar… | Quick bar… |  | same |  |
| 154 | [submenu] Theme |  |  | mockup-only | (c) P1 |
| 155 | › [radio] Dark |  |  | mockup-only (inside) | (c) P1 |
| 156 | › [radio] Light |  |  | mockup-only (inside) | (c) P1 |
| 157 | › [radio] Follow system |  |  | mockup-only (inside) | (c) P1 |
| 158 | [submenu] Units |  |  | mockup-only | (c) P1 |
| 159 | › [radio] Kilometres |  |  | mockup-only (inside) | (c) P1 |
| 160 | › [radio] Miles |  |  | mockup-only (inside) | (c) P1 |
| 161 | › [radio] Nautical miles |  |  | mockup-only (inside) | (c) P1 |
| 162 | [submenu] Handedness | [chips] Handedness |  | same |  |
| 163 | › [radio] Left | › [radio] Left |  | same |  |
| 164 | › [radio] Right | › [radio] Right |  | same |  |
| 165 | Keyboard shortcuts… | Keyboard shortcuts… |  | same |  |
| 166 | **TOUCH · NEW** | **TOUCH** |  | differs: label | n N1 |
| 167 | Touch & gestures… | Gesture reference |  | differs: label | n N1 |

#### Window

| # | Mockup | Godot | Godot today | Status | Class |
|---:|---|---|---|---|---|
| 1 | [check] Left dock | [check] Left dock |  | same |  |
| 2 | [check] Context panel |  |  | mockup-only | (c) P2 |
| 3 | [submenu] Context panel |  |  | mockup-only | (c) P2 |
| 4 | › Inspect |  |  | mockup-only (inside) | (c) P2 |
| 5 | › Note |  |  | mockup-only (inside) | (c) P2 |
| 6 | › Ask (assistant) |  |  | mockup-only (inside) | (c) P2 |
| 7 | › ——— |  |  | mockup-only (inside) | (c) P2 |
| 8 | › Pin to the current subject |  |  | mockup-only (inside) | (c) P2 |
| 9 | › Follow the selection |  |  | mockup-only (inside) | (c) P2 |
| 10 |  | [check] Right dock |  | Godot-only | (c) P2 |
| 11 | [check] Timeline | [check] Timeline |  | same |  |
| 12 | [check] Status bar | [check] Status bar |  | same |  |
| 13 | [check] Domain rail | [check] Domain rail |  | same |  |
| 14 | ——— | ——— |  | same |  |
| 15 | [check] Session mode | [check] Session mode |  | same |  |
| 16 | [check] Diagnostics overlay | [check] Diagnostics overlay |  | same |  |
| 17 | [check] 3D view | [check] 3D view |  | same |  |
| 18 | [submenu] 3D capability | [submenu] 3D capability |  | same |  |
| 19 | › [readout] 3D tier | › [readout] 3D tier: not tested |  | same |  |
| 20 | › Re-test 3D | › Re-test 3D |  | same |  |
| 21 | › [submenu] Force tier | › [submenu] Force tier |  | same |  |
| 22 | › › [radio] Automatic (as measured) | › › [radio] Automatic (as measured) |  | same |  |
| 23 | › › [radio] Off | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 24 | › › [radio] Basic | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 25 | › › [radio] Standard | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 26 | › › [radio] Full | › › [radio] {View3DGate.TIER_NAMES[t]} · run-time list |  | same (run-time list) |  |
| 27 | ——— | ——— |  | same |  |
| 28 | [submenu] Workspace | [submenu] Workspace |  | same |  |
| 29 | › [radio] World · SM | › [radio] World · SM |  | same |  |
| 30 | › [radio] Civilization | › [radio] Civilization |  | same |  |
| 31 | › [radio] Cartography | › [radio] Cartography |  | same |  |
| 32 | [submenu] Open windows | [submenu] Open windows |  | same |  |
| 33 | › [readout] No windows open | › [check] {w.title.strip_edges()} · run-time list |  | same (run-time list) |  |
| 34 | › [check] Context panel | › [check] {w.title.strip_edges()} · run-time list |  | same (run-time list) |  |
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
| 1 |  | [readout] Runtime |  | Godot-only | (c) P1 |
| 2 |  | [readout] Engine |  | Godot-only | (c) P1 |
| 3 |  | Gesture reference |  | Godot-only | (c) P1 |
| 4 | [not built] Documentation | [not built] Documentation |  | differs: reason | s S1 |
| 5 | Keyboard reference… | Keyboard reference… |  | same |  |
| 6 | Gesture reference… |  |  | mockup-only | (c) P1 |
| 7 | Credits & academic principles | Credits & academic principles |  | same |  |
| 8 | Generation info… | Generation info… |  | same |  |
| 9 | Save diagnostic report… | Save diagnostic report… |  | same |  |
| 10 | About | About |  | same |  |
| 11 |  | [info] The phone reorganises rather than truncates: every desktop function is reachable through MAP · DOMAIN · STORY · MENU. |  | Godot-only | (c) P1 |
