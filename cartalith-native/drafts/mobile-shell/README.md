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
both checks: 93 calls (88 until the 2026-11-01 menu parity fixes to drafts 05 and 08), none missing; no collisions. Neither check is a parse. The first thing to run on the PC, after pasting each draft:

```
godot --headless --path cartalith-native/godot-project --check-only --script res://shell/<file>.gd
```

Then parse-check `shell/app.gd` as well. Re-run the parse check after every edit, not once per file. A failed script load makes
`--headless` hang rather than exit (`MISTAKES.md`).

## Order of work

| # | Draft | Milestone | Target | Replaces / adds | Blocked by |
|---|---|---|---|---|---|
| 0 | `check_symbols.sh` | — | — | re-checks the drafts against the current tree | nothing; run it first |
| 1 | `01_dcc_theme_classify.gd` | MS-1 S1 | `shell/dcc_theme.gd` | adds `classify()`, `width_class_of()`, `layout()`; takes over `_PHONE_ASPECT_MAX` and `_TABLET_MIN_DP` from the shell | nothing |
| 2 | `02_dcc_shell_density_unit.gd` | MS-1 S1, S2, S3 | `shell/dcc_shell.gd` | replaces `_compute_layout_mode()`, `_is_tablet_sized()` and `_on_window_resized()`; adds `_density()`, `--density`, `--ui-scale`, `_apply_unit()` and the resize debounce | S0's measurements before S2 (spec §4.1); MQ-2 and MQ-3 before S2 |
| 3 | `03_dcc_shell_phone_tabs.gd` | MS-2 | `shell/dcc_shell.gd` | replaces `PHONE_TABS`, `_pick_phone_tab()`, `_phone_tab_for_domain()` and `_phone_tab_drives_sheet()`; adds the domain tab's tap, hold and swipe; renames the old tabs in two drawn strings (EDIT H1 in `shell/phone_menu.gd::_fill_help()`, H2 the coach mark, added 2026-11-01) | drafts 4 and 5 |
| 4 | `04_phone_domain_fan.gd` | MS-2 | new `shell/phone_domain_fan.gd` | the domain fan: WORLD · GENERATE, CIVIL, CARTO | nothing |
| 5 | `05_phone_menu_root.gd` | MS-4, MS-6 | `shell/phone_menu.gd` | replaces `_fill_root()`, `_fill_story_section()` (the Story screen's four band rows, without the old root's "Story" and "World" heads) and `_fill_popup()` (a leading captioned separator is kept as a band); adds the `world` and `story-root` screens and `open_story()`; retires `ROOT_ROWS`, `ROOT_REST`, `_named_in_root()` | nothing |
| 6 | `06_quick_drawer.gd` | MS-11 | new `shell/quick_drawer.gd` | the phone's side drawer of pinned commands | Q10 and Q11 (proposal §7) |
| 7 | `07_quick_bar_phone_drawer_patch.gd` | MS-11 | `shell/quick_bar_phone.gd`, `shell/dcc_shell.gd` | replaces `QuickBarPhone.refresh()`; adds `set_phone_quick_drawer()` and the back hook | draft 6 |
| 8 | `08_menus_tablet_bar.gd` | MS-7 (menu bar) | `shell/menus.gd` | replaces `build()`: eight menus from 1 180 dp, World being the PC's (`_pc_world()`) on the desktop, the phone and the wide tablet | draft 1; `drafts/pc-shell/` 01 and 02 (they add `_pc_world()`); MS-1 S4 for a live switch across 1 180 dp |

Drafts 3 to 8 do not depend on MS-1's unit; they size themselves through the shell's existing `_pscale()`, `_pfont()` and
`phone_scale()`. They can land before S2 if the owner prefers the menus first.

## Not drafted, and why

- **MS-1 S4, a class change without a restart.** It starts with an inventory (spec §3.5, S4.0) of every field, hook and once-only
  `setup()` the two compositions own. A draft written before that inventory would be a guess.
  `_request_class_swap()` (draft 2) is its entry point and logs until then.
- **MS-3, the domain sheet from the dock builders.** It needs the dock category builders read one by one. Until it lands, MAP keeps
  selecting CARTOGRAPHY, as shipped (draft 3 says where).
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

## Where each retired MENU root row goes (draft 5)

`MISTAKES.md`, "Delete a surface": every item gets a stated home.

| Retired root row (`ROOT_ROWS` and bands) | New home |
|---|---|
| Quick (band) | MENU, first, unchanged (`_fill_quick()`) |
| Project → `project` | MENU ▸ File (the same screen) |
| Civilization → `civ` | MENU ▸ World ▸ On the phone; its lasting home is the CIVIL domain sheet (MS-3) |
| Story band: Timeline, Journeys, Conflicts & campaigns, Notes | the STORY tab (`open_story()`), and MENU ▸ Story (`story-root`) |
| Data manager → `data` | MENU ▸ World ▸ On the phone (and World's own first row opens the window) |
| Asset library → `assets` | MENU ▸ Assets (the same screen, with its fill badge) |
| Travel library → `travel` | MENU ▸ World ▸ On the phone |
| Simulation → `sim` | MENU ▸ World ▸ On the phone |
| Preferences → `prefs` | MENU ▸ Preferences (the same screen) |
| Help & about → `help` | MENU ▸ Help (the same screen) |
| Status (band) | MENU, last, unchanged (`_fill_status()`, moved verbatim) |
| "Not on the MORE list": Edit, Window | MENU ▸ Edit and MENU ▸ Window, in menu-bar order (`_menu_row()`) |

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
  - MENU's root lists Quick, then File · Edit · Assets · World · Story · Preferences · Window · Help in the menu bar's order, then Status.
  - Every row in the table above is reachable.
  - `open_story("JOURNEYS")` lands on the band, and system back returns to Story.
- **Drafts 6 and 7:** proposal MS-11.
  - Every pin runs from the drawer.
  - The grip is at least 44 × 88 dp and draws nothing while the bar is empty, which keeps `_quickbar_phoneid_probe.gd` green.
  - Tap and drag open the drawer; the scrim, a drag back and system back close it.
  - The drawer sits under the sheet and the bar at every detent.
- **Draft 8:**
  - At 1 180 dp and wider, the tablet shows the eight menus, and World is the PC's: Generate world · Ctrl+R first, no second Reset generation parameters or Reset one stage (they are in Edit).
  - The desktop's World starts with Generate world (this `build()` replaces the shipped one, so it must carry the PC drafts' World).
  - Below that it shows `☰ · File · World · Story`.
  - Every ☰ row is reachable in the eight-menu form.
  - The PC's framebuffer is unchanged (B9).

Every stage also runs `cargo test --workspace` (no Rust changes, so it is unaffected) and parse-checks every touched `.gd` file plus
`shell/app.gd` (`MISTAKES.md`, "Declare the work green").

## `MISTAKES.md` rows to carry into the PC session

Schedule or act on a backlog row (re-open each symbol; `check_symbols.sh` does part of it) · Call a shell helper (grep `func <name>`
first) · Change behaviour (hunt the old prose: "decided once at boot", `MAP · GENERATE · PLAN · MORE`, the MORE root's band names) ·
Delete a surface · Move a command off the menu bar (the `CommandIndex` walks the live bar) · Run `godot --headless --import` (drafts 4
and 6 avoid `class_name` for this reason) · Re-run the parse check after EVERY edit · Run a pixel probe (windowed) · Assert a control
is present, sized or reachable (drawn rect against the visible rect) · Ask a lane to walk a touch floor (target, both axes) · Add a
probe in the same commit that changes the behaviour · Write a probe's usage header · Declare the work green.
