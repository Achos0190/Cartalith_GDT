# Mobile shell: GDScript drafts for the PC agent

**What this is.** These are GDScript drafts of the mobile shell's menus and their foundation, written 2026-10-31 against commit `5bd7af8`.
The owner asked for them: *"when you are done creating the proper documents you will start drafting the proper menus in .gd format. This
so that when we can start the local agent on pc again the implementation would be a relatively simple task."*
Each draft is code meant to be pasted into, or added beside, a named shipped file. Each draft's header names its target symbols, what it
replaces, its milestone and its prerequisites.

**What it is not.** It is not a build. Nothing here is loaded, because this folder is outside `godot-project/`, so Godot never sees it.
How far any milestone has got is `cartalith-native/docs/STATUS.md`'s answer, never this folder's. The design is
`MOBILE_SHELL_PROPOSAL.md` (with the mockup in `design/mobile-shell-2026-10-31/`). MS-1's build specification is
`MOBILE_SHELL_MS1_SPEC.md`.

**Not parse-checked.** The cloud session that wrote these had no Godot binary, and downloading one was not permitted. Every shipped
symbol the drafts call was checked to exist, and every new name was checked to be free in its target file. `check_symbols.sh` repeats
both checks: 97 calls (88 on 2026-10-31; 93 after the menu parity fixes to drafts 05 and 08; 97 since Ruling DX's drafts 05 and 09, and still 97 after Ruling DY's draft 08, which calls `_world` where it called `_tablet_world`), none missing; 59 new names, no collisions; and every text an edit lands on is still there once. Neither check is a parse. The first thing to run on the PC, after pasting each draft:

```
godot --headless --path cartalith-native/godot-project --check-only --script res://shell/<file>.gd
```

Then parse-check `shell/app.gd` as well. Re-run the parse check after every edit, not once per file. A failed script load makes
`--headless` hang rather than exit (`MISTAKES.md`).

## Transplanted (a pointer; `STATUS.md` holds the state)

Drafts 1 and 2 landed with MS-1 (S1 to S3). **Drafts 3 and 4 landed with MS-2** (2026-11-08), as `shell/dcc_shell.gd`
(`PHONE_TABS`, `DOMAIN_TAB`, `_build_phone_tab_cell()`, `_pick_phone_tab()`, the domain tab's gesture handlers) and the new
`shell/phone_domain_fan.gd`, with **draft 5's STORY half only** (`PhoneMenu.open_story()`, the `story-root` screen and
`_fill_story_rows()`): draft 3 calls `open_story()`. The rest of draft 5 (MENU's root, `_fill_popup()`, the fourteen
unreachable screens and their sixteen probes) is MS-4 and is **not** landed; MENU is still the old MORE root, retitled
"Menu". Departures from the drafts as pasted, each commented where it lives: `_phone_tab_for_domain()` keeps MAP and the
overlay tabs lit (the draft's `"domain"` for everything would un-light MAP on its own press); `_pick_phone_tab()` also
refuses MAP under Session mode (the shipped loop did) and a second tap on a lit STORY closes it; `DOMAIN_TAB` gained a
`sub` line for the sheet header; `_refresh_phone_tabs()` tints the glyph (it moved out of `_select_domain()`'s registry
loop with the cells); the PLAN header's live "journey A to B" now belongs to STORY while the planner is on screen. The
coach mark (H2) reads "MAP · domain · STORY switch tasks here — hold the domain tab to change domain, MENU reaches
everything else.", with `_coachcoldboot_probe.gd` and `_pandraghint_probe.gd` re-pinned. `check_symbols.sh` no longer
anchors draft 03's coach-mark text (replaced); its "TAKEN" lines for MS-1's and MS-2's names are expected now that drafts 1 to 4 are in.

## Order of work

| # | Draft | Milestone | Target | Replaces / adds | Blocked by |
|---|---|---|---|---|---|
| 0 | `check_symbols.sh` | — | — | re-checks the drafts against the current tree | nothing; run it first |
| 1 | `01_dcc_theme_classify.gd` | MS-1 S1 | `shell/dcc_theme.gd` | adds `classify()`, `width_class_of()`, `layout()`; takes over `_PHONE_ASPECT_MAX` and `_TABLET_MIN_DP` from the shell | nothing |
| 2 | `02_dcc_shell_density_unit.gd` | MS-1 S1, S2, S3 | `shell/dcc_shell.gd` | replaces `_compute_layout_mode()`, `_is_tablet_sized()` and `_on_window_resized()`; adds `_density()`, `--density`, `--ui-scale`, `_apply_unit()` and the resize debounce | S0's measurements before S2 (spec §4.1); MQ-2 and MQ-3 before S2 |
| 3 | `03_dcc_shell_phone_tabs.gd` | MS-2 (**landed**) | `shell/dcc_shell.gd` | replaces `PHONE_TABS`, `_pick_phone_tab()`, `_phone_tab_for_domain()` and `_phone_tab_drives_sheet()`; adds the domain tab's tap, hold and swipe; flags the coach mark that still names the old tabs (H2, added 2026-11-01; H1 withdrawn by Ruling DX Q13) | drafts 4 and 5 |
| 4 | `04_phone_domain_fan.gd` | MS-2 (**landed**) | new `shell/phone_domain_fan.gd` | the domain fan: WORLD · GENERATE, CIVIL, CARTO | nothing |
| 5 | `05_phone_menu_root.gd` | MS-4, MS-6 | `shell/phone_menu.gd` | replaces `_fill_root()` (the eight menus row for row, Ruling DX Q13), `_fill_story_section()` (the STORY tab's four band rows, without the old root's "Story" and "World" heads) and `_fill_popup()` (a leading captioned separator is kept as a band); adds `_menu_root_row()`, the `story-root` screen, `open_story()` and `open_screen()`; retires `ROOT_ROWS`, `ROOT_REST`, `_named_in_root()` and leaves fourteen screens unreachable (the inventory below) | nothing |
| 6 | `06_quick_drawer.gd` | MS-11 | new `shell/quick_drawer.gd` | the phone's side drawer of pinned commands | Q10 and Q11 (proposal §7) |
| 7 | `07_quick_bar_phone_drawer_patch.gd` | MS-11 | `shell/quick_bar_phone.gd`, `shell/dcc_shell.gd` | replaces `QuickBarPhone.refresh()`; adds `set_phone_quick_drawer()` and the back hook | draft 6 |
| 8 | `08_menus_tablet_bar.gd` | MS-7 (menu bar) | `shell/menus.gd` | replaces `build()`: the PC's eight menus from 1 180 dp, with Records where World was (Ruling DY); below it ☰ · File · World · Records · Story, World being the WORLD domain's own rows (`_world()`) | draft 1; `drafts/pc-shell/` 01 and 02 (they add `_generate_in_world`, read by File's Generate world row, and retire `_tablet_world()`); MS-1 S4 for a live switch across 1 180 dp |
| 9 | `09_menus_phone_rows.gd` | MS-4 | `shell/menus.gd`, one function for `shell/dcc_shell.gd` | adds Help ▸ Gesture reference… on the phone (`ID_HELP_GESTURES` 919, `_build_gesture_reference_row()`, an `_on_help()` arm) and `DccShell.phone_open_menu_screen()`; makes `_open_landmark_dock()` open the left sheet on the phone | draft 5 (`open_screen()`) |

Drafts 3 to 9 do not depend on MS-1's unit; they size themselves through the shell's existing `_pscale()`, `_pfont()` and
`phone_scale()`. They can land before S2 if the owner prefers the menus first.

## Not drafted, and why

- **MS-1 S4, a class change without a restart.** It starts with an inventory (spec §3.5, S4.0) of every field, hook and once-only
  `setup()` the two compositions own. A draft written before that inventory would be a guess.
  `_request_class_swap()` (draft 2) is its entry point and logs until then.
- **MS-3, the domain sheet from the dock builders.** It had no draft, because it needs the dock builders read one by one. Its first step
  landed 2026-11-08 without one, as new code in `shell/dcc_shell.gd` (a departure from this README's plan of transplanting: there was
  nothing to transplant, and the owner's *rebuilding is nonsense* rules out a second copy of the dock): `_mount_dock_in_sheet()` reparents
  the left dock's mode switch and category scroller into the phone tool sheet for CIVIL and CARTO (`_refresh_phone_gen_panel()` decides),
  and `_set_sheet_open("left", true)` returns them. WORLD keeps `build_phone_generate()`'s column, and MAP keeps the tool row and still
  selects CARTOGRAPHY (draft 3 says where); both are MS-3's remainder, listed in `STATUS.md`.
- **MS-5** (tool strip, tool fan, context card verbs), **MS-8** (New World auto-populate), **MS-9** (search runs every command) and
  **MS-10** (device passes) are not menus.

## The adaptable quick bar and the cartwheel (checked 2026-10-31)

- **The cartwheel is present on the tablet and stays.** `radial_ring.gd` (CM-3, CM-4) opens it with the context card on a 500 ms hold
  (`MAP_CONTEXT_SCOPE.md` §7.1), at its touch scale (`_apply_touch_scale()`). Up to four pins sit under it (CQ-4,
  `quick_bar_ring.gd`). No draft touches it. The proposal had described the tablet hold as opening the card alone; §2.2 and §2.4 now
  correct that.
- **The quick bar keeps every home.**
  - The tablet tool bar's cluster (CQ-2, `DccQuickBar`) fits as many pins as the bar's width allows, so a wider tablet already shows more.
  - MENU's Quick section is `PhoneMenu._fill_quick()`, kept first by draft 5.
  - On the phone, drafts 6 and 7 move the pins from CQ-6's row in the sheet to a side drawer on a grip, which is the owner's suggestion
    and Q10's default.

## Where each phone-only row goes (drafts 5 and 9, Ruling DX Q13)

`MISTAKES.md`, "Delete a surface": every item gets a stated home or a stated drop. Ruling DX Q13 (2026-11-01): the
phone's MENU follows the mockup and repeats the desktop menus row for row, instead of opening the phone screens. Each
row of those screens was read at its builder in `shell/phone_menu.gd` (`_fill_project`, `_fill_assets`,
`_fill_prefs`, `_fill_help`, `_fill_civ`, `_fill_data`, `_fill_sim`, `_fill_travel`; draft 5's first version added
`_fill_world`'s "On the phone" band). Each home was checked at the symbol named.

| Phone screen ▸ row | Home, or a stated drop |
|---|---|
| Root: Quick, Status | MENU, first and last, unchanged (`_fill_quick()`, `_fill_status()`) |
| Root: the eight menus | MENU's MENUS band, each row opening its menu's popup row for row (`_menu_root_row()`, `_fill_popup()`) |
| Project: Save project, Save as…, New world…, Open project… drawn first | MENU ▸ File, the same rows in File's order (Generate world second since Ruling DY) |
| Project: Recent worlds inline under a band | MENU ▸ File ▸ Recent worlds (one drill deeper; same `_refresh_recent_worlds()` rows) |
| Project: Autosave and its "Interval" chips under a band | MENU ▸ File ▸ Autosave and ▸ Autosave interval |
| Project, Asset library, Help & about: the screen titles | **dropped**: a popup's screen is titled with its menu's name (File, Assets, Help) |
| Asset library: the nine families with fill badges, and their slot grid and slot inspector (`assets-grid`, `asset-slot`) | MENU ▸ Assets ▸ Icon families (six) and ▸ Texture sets (three); each opens the Asset library on that family (`app.gd::open_asset_library()`), whose rail, slot grid and inspector are the same slots |
| Asset library: PACK Name, Author, Licence, Stored items | MENU ▸ Assets ▸ Asset pack ▸ NAME, AUTHOR, LICENSE and FILLED (FILLED carries the item total, `_refresh_asset_pack_stats()`) |
| Preferences: Theme and Units drawn first | MENU ▸ Preferences ▸ Theme and Units, in the APPLICATION band where the popup has them |
| Preferences: TOUCH ▸ Gesture reference; Help & about: Gesture reference | MENU ▸ Help ▸ Gesture reference…, a new phone row (draft 9), which is the mockup's own row |
| Help & about: Runtime (Godot version · OS) | MENU ▸ Help ▸ About (`app.gd::open_about()` prints the same `Godot %s · %s`) |
| Help & about: Engine (the GPU) | MENU's Status band, GPU (the same `top_gpu` slot, `STATUS_ROWS`) |
| Help & about: the closing line ("The phone reorganises rather than truncates …") | **dropped** from Help with the screen. The mockup draws the sentence in its About window; Godot's About has no such line, and none is drafted |
| Civilization: Settlement, Territory, Territory lasso, Way, Route, Conflict (arm a tool) | Window ▸ Workspace ▸ Civilization, then Window ▸ Left dock: the CIVIL dock's TOOLS block arms the same six (`_go_civilization()` landed there). MS-3's domain sheet and MS-5's tool strip are their planned phone homes |
| Civilization: Civilization dock | the same two Window rows |
| Civilization: Point of interest (a not-built row) | **dropped**: no POI tool exists; POI art stays at Assets ▸ Icon families ▸ Points of interest |
| Civilization: Landmark generation (`landmarks`, `lm-fam`: caps, spacing, one run) | Assets ▸ Landmark types ▸ any family's rows, which open CIVIL ▸ Landmarks at that family; draft 9 makes them open the left sheet on the phone |
| Civilization: Open journey planner | Story ▸ Journey planner |
| Data manager: the records menu's rows | MENU ▸ Records (World until Ruling DY), the same popup |
| Data manager: Maps · tile pyramid (`data-tiles`) | File ▸ Export ▸ Maps (tiles), the Data manager window's `export_maps` route |
| Travel library (`travel`, `travel-item`) | Story ▸ Travel library…, the window the screen already ended on |
| Simulation: Transport strip on map, Year, Speed, Playback | Window ▸ Timeline: the phone's timeline bar, whose row opens the sim strip (play, scrub, speed; `app.gd::_fill_phone_timeline_row()`) |
| Simulation: the six Simulation layers | **dropped on the phone**: they record a choice and no layer renders yet (`DccShell.TL_LAYER_NOTE`); the desktop and the tablet keep them on the timeline strip. If the owner wants them on the phone: Story ▸ TIME ▸ "Timeline layers ▸" (six checks), phone only |
| World ▸ "On the phone" band (draft 5's first version) | **dropped** with the four doors above |
| Story band rows (Timeline, Journeys, Conflicts & campaigns, Notes) | the STORY tab (`open_story()`); MENU ▸ Story is the Story popup row for row |
| "Not on the MORE list": Edit, Window | MENU ▸ Edit and MENU ▸ Window, in menu-bar order, as every menu is now |

**Unreachable after draft 5**, and to be deleted in the same change: the screens `project`, `civ`, `data`, `data-tiles`,
`sim`, `prefs`, `assets`, `assets-grid`, `asset-slot`, `travel`, `travel-item`, `landmarks`, `lm-fam` and `help`, with their
`SCREEN_TITLES` entries and helpers. Their only doors were the retired root and each other: `grep -n '_push_screen("'
shell/phone_menu.gd` lists every one. `gestures` (draft 9) and `story` (the STORY tab) keep theirs. Sixteen probes push
or read those screens or `ROOT_ROWS`, found with `grep -ln '_push_screen\|ROOT_ROWS' cartalith-native/godot-project/*.gd`:
`_chipink_probe.gd`, `_emptyphone_probe.gd`, `_etd4_probe.gd`, `_phonemore2_probe.gd`, `_phonemore_act_probe.gd`,
`_phonemore_reach_probe.gd`, `_phonescroll_probe.gd`, `_phonetiles_probe.gd`, `_prefchip_probe.gd`,
`_quickbar_phone_probe.gd`, `_quickbar_phoneid_probe.gd`, `_rangeswipe_probe.gd`, `_storymenus_probe.gd`,
`_vfyb13_probe.gd`, `_vfybold_probe.gd`, `_vfychip_probe.gd`. Each is re-pointed at the menu row that replaced its screen,
or retired with it. That is a reading of their file names and a grep, not a run (no Godot here).

**Mockup rows this needs**: none new. Gesture reference… is already the mockup's (Help, `mobile`). The one row the owner
might add is the optional "Timeline layers" above.

## Ruling DY (owner, 2026-11-01)

The owner answered the PC shell's questions and their follow-ups the same day (`LARGE_ITEM_RULINGS.md`, Ruling DY).
Three answers reach these drafts:

- **The World menu is Records** (PQ6, "Rename to 'Records'"). Draft 5's two tables follow it by title
  (`MENU_SUBS`, `MENU_GLYPHS`); every other MENU row is the desktop popup's, so it follows by itself.
- **Generate world sits in File, after New world…**, so MENU ▸ File gains it as its second row, from the desktop popup.
- **The tablet's short bar is `☰ · File · World · Records · Story`** ("File · World · Records · Story"). Draft 8 builds
  it, with World the WORLD domain's own rows (`_world()`) and Records the desktop's (`_data()`); the short bar's File
  leaves Generate world out because World holds it (`drafts/pc-shell/02`'s `_generate_in_world`, a labelled
  judgement). It reopens Ruling CK Q2, which merged the records rows into the tablet's World menu
  (`_tablet_world()`, retired by `drafts/pc-shell/02`). **The five-title bar is unmeasured at 600 dp**: draft 8's bar
  above asks the implementing session to measure it before calling it done.

Q10 to Q12 (the quick drawer, its grip, the tablet's quick bar and cartwheel) and MQ-1 to MQ-5 (MS-1) were answered by
the same ruling; drafts 6 and 7 already draw Q10 and Q11's defaults, and no draft here implements MQ-5's choice
(MS-1 S4 is not drafted: "Not drafted, and why" above).

## Verifying each draft

These are the specs' bars, restated per draft. Every probe is windowed when it measures pixels. Every probe is a committed `.tscn` with
a usage header that names exactly the arguments it reads. Every new probe must fail on the code before its change.

- **Draft 1:** spec B1. The classification table is literal, with boundary rows from both sides, and the mutation of 900, 0.6 and the
  16 dp band must turn it red.
- **Draft 2:** B2 and B10 at S1 (nothing moves: windowed captures byte-identical, every `--vp` probe's class unchanged), B3 and B7 at S2,
  and B4 at S3.
- **Drafts 3 and 4:**
  - All three domains are reachable in one gesture: hold, slide, lift.
  - A sideways swipe cycles the domain, and a plain tap still opens the domain sheet.
  - Under mouse emulation, one finger produces one gesture, not two.
  - The fan's targets are at least 44 × 44 dp, and a Session-mode refusal draws dim and cannot be picked.
  - The six `for t in PHONE_TABS` loops are re-read.
- **Draft 5:**
  - MENU's root lists Quick, then the MENUS band with File · Edit · Assets · Records · Story · Preferences · Window · Help in the menu bar's order, then Status.
  - MENU ▸ File's second row is Generate world (Ruling DY), and MENU ▸ Records is the records menu row for row.
  - Each menu opens its popup row for row, under the menu's own name: the same rows, order, bands and submenus as the desktop popup, File ▸ Export's FOR THE TABLE band included. `cartalith-native/docs/evidence/menu-parity-2026-11-01/` checks this statically against the mockup.
  - Every home in the table above is reachable, and every row marked dropped is gone.
  - `open_story("JOURNEYS")` lands on the band, and system back returns to Story.
- **Draft 9:**
  - On the phone MENU ▸ Help lists Gesture reference… third; it opens the gesture reference, and system back returns to MENU.
  - On the phone Assets ▸ Landmark types ▸ any type opens the left sheet on CIVIL ▸ Landmarks at that family.
  - The desktop and the tablet build no Gesture reference… row, and their landmark rows behave as before.
- **Drafts 6 and 7:** proposal MS-11.
  - Every pin runs from the drawer.
  - The grip is at least 44 × 88 dp and draws nothing while the bar is empty, which keeps `_quickbar_phoneid_probe.gd` green.
  - Tap and drag open the drawer; the scrim, a drag back and system back close it.
  - The drawer sits under the sheet and the bar at every detent.
- **Draft 8:**
  - At 1 180 dp and wider, the tablet shows the PC's eight menus: Records where World was, Generate world · Ctrl+R as File's second row, and no Reset generation parameters or Reset one stage outside Edit.
  - The desktop's bar is File · Edit · Assets · Records · Story · Preferences · Window · Help, and its File carries Generate world (this `build()` replaces the shipped one, so it must carry the PC drafts' rows).
  - Below 1 180 dp it shows `☰ · File · World · Records · Story`: World is Generate world, the two resets, Generation pipeline / Sculpt and Seed & size…, and nothing else; File has no Generate world; Records is the desktop's.
  - **The short bar fits** (Ruling DY added a fifth title): measured windowed at 600, 768, 1 024 and 1 179 dp, every title is drawn inside the bar, none is clipped, and none overlaps the quick cluster. Not measured yet; the draft's estimate is about 80 dp more, from the mockup's sizing rule.
  - Every ☰ row is reachable in the eight-menu form.
  - Against the PC drafts applied, this draft changes nothing on the PC (B9): the title Records and File's Generate world are `drafts/pc-shell/02`'s change, not this draft's, so B9 compares with that tree.

Every stage also runs `cargo test --workspace` (no Rust changes, so it is unaffected) and parse-checks every touched `.gd` file plus
`shell/app.gd` (`MISTAKES.md`, "Declare the work green").

## `MISTAKES.md` rows to carry into the PC session

Schedule or act on a backlog row (re-open each symbol; `check_symbols.sh` does part of it) · Call a shell helper (grep `func <name>`
first) · Change behaviour (hunt the old prose: "decided once at boot", `MAP · GENERATE · PLAN · MORE`, the MORE root's band names) ·
Delete a surface · Move a command off the menu bar (the `CommandIndex` walks the live bar) · Run `godot --headless --import` (drafts 4
and 6 avoid `class_name` for this reason) · Re-run the parse check after EVERY edit · Run a pixel probe (windowed) · Assert a control
is present, sized or reachable (drawn rect against the visible rect) · Ask a lane to walk a touch floor (target, both axes) · Add a
probe in the same commit that changes the behaviour · Write a probe's usage header · Declare the work green.
