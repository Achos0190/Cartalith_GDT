# CLAUDE.md additions for v2.73 (paste into Cartalith_RC's root `CLAUDE.md`)

Two pastes. Neither changes anything else in the file.

## 1 · The file table

**Replace** the opening of the `Cartalith v2.72 DCC test.html` row,
`**The DCC shell line's current head, not a mainline version.**`, with `Previous DCC-line file.` (the rest of
that row stays as it is — the convention every older row follows), and **insert this row above it**:

```
| `Cartalith v2.73 DCC test.html` | **The DCC shell line's current head, not a mainline version.** Owner: *"Can we upgrade the layout in the html with the new design?"*, *"Everything, staged"*, and on v2.72's phone capture *"The screenshots on phone don't seem to have the pull-out tab from the side."* **STAGE 2 OF 3: THE PHONE AND TABLET SHELLS**, from Cartalith_GDT's `MOBILE_SHELL_PROPOSAL.md` §2.1–§2.5, §3, §4 and Rulings DT/DU/DX/DY/EB. **ONE decider, `_shellClassOf`**: under 600px wide, or under 500px tall in landscape, is a PHONE (MQ-1, Ruling DY); otherwise a coarse primary pointer is a TABLET (the desktop shell at touch size) and a fine one the DESKTOP (stage 1's shell) — re-run on resize, rotation and pointer change, with 16px hysteresis and a keyboard guard, reading `documentElement.clientWidth` (a phone's `innerWidth` read 504 for 360). **CSS keys on `html[data-shell]` through `:where()`, never on the width** (re-keying v2.72 changed 0 computed styles over 2 305 elements at 1366 and 1600). **`_SHELL_SLOTS`** generalises v2.72's table to a home per mode — desktop, tablet, the narrow bar, phone — 46 rows into 36 slots, still one moved element each. The PHONE: a bottom bar (MAP · domain tab · STORY · MENU; a left rail in landscape), an app bar (world pill · Undo · Redo · Search), ONE sheet (peek 76 · half · full) whose pages are the left dock, the Context panel, Explore's Timeline and Journeys, the Base map popover and the eight menu panels row for row; the domain fan (380ms hold, swipe, the title's ▾); the armed tool's strip; **the quick drawer on a 44×88 grip** (the pull-out tab, away from the dominant hand); the long-press card (the right-click menu's own items). **DS-13** on every touch shell: ⌖ · −|+ · ✋ (✋ only with a drawing tool), mirrored by Preferences › Handedness. **Touch gestures** in one capture layer: a tap tool's press is withheld and replayed, a drift pans (DS13-Q2 default, pending), double-tap ×1.7 and thumb zoom (down = in) with Inspect/Info or ✋, off with a drawing or placing tool (pending), Krita's guard. The TABLET: 44px floor, 48px drill-in menus, below 1180px ☰ · File · World · Records · Story (the desktop below 860px too). **The docking rule** on desktop and tablet: a dock floats (closed, pin, closes on a map press) unless the map keeps 520px. `tests/run.sh` 1280/0, `run_um.sh` 882/0, `hash_gen1.js` vs v2.72 **ALL IDENTICAL**. Verify with `tests/perf/probe_mobileshell.js` (159 assertions; 88 fail on v2.72). |
```

## 2 · The section

**Insert above** `### The desktop shell, stage 1: menus, tool settings only, the Context panel, the year everywhere (v2.72, DCC-line file only)`
(DCC-line sections run newest first):

```markdown
### The phone and tablet shells, stage 2: one decider, one sheet, the pull-out tab, DS-13 and the touch layer (v2.73, DCC-line file only)

Owner: *"Can we upgrade the layout in the html with the new design?"*, then *"Everything, staged"*; on v2.72's phone
capture, *"The screenshots on phone don't seem to have the pull-out tab from the side."* Stage 2 is the phone and the
tablet; stage 3 is the Context panel's modules and the planner window. Design: Cartalith_GDT `MOBILE_SHELL_PROPOSAL.md`
§2.1–§2.5, §3, §4 and Rulings DT, DU, DX, DY, EB. `tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs
v2.72 **ALL IDENTICAL**; verification is `tests/perf/probe_mobileshell.js` (159 assertions of real taps,
CDP touches and geometry at seven sizes across the three classes; run on v2.72 it fails 88).

- **The class is decided in ONE function, `_shellClassOf`, and nowhere else.** Phone: under 600px wide, or landscape
  under 500px tall (MQ-1). Tablet: any other window whose PRIMARY pointer is coarse. Desktop: any other with a fine
  one. `_shellApply()` writes `html[data-shell]`, re-placed and re-synced on every resize, `orientationchange` and
  `(pointer:coarse)` change. Never add a second decider — **no media query for the class**: CSS keys on
  `:where(html[data-shell="phone"])` / `:where(html:not([data-shell="phone"]))`, zero added specificity, so the
  cascade is the one a media query would give and CSS and JS cannot disagree. Width tweaks that are not a class
  (status chips at 1560/1180/900, docks at 1440, the bar's wordmark at 760) may stay media queries.
- **Hysteresis and the keyboard guard are part of the decider, not of a caller.** Once a class is in force its line
  moves 16px away (a window on 600 or a landscape phone crossing 500 does not flap); while a text field has focus and
  the width is unchanged, the class is kept (an on-screen keyboard is not a rotation). A fresh load has no class, and
  its lines are exact.
- **Read the LAYOUT viewport, `documentElement.clientWidth/Height`, never `innerWidth`.** A phone's `innerWidth` is the
  VISUAL viewport, which a mobile browser widens and zooms out whenever anything overflows: measured, a 360px phone
  read 504 at boot and was given the tablet shell.
- **`_shellEarly()` runs at the TOP of script block 1** (hoisted) so the first paint is the right shell. So the decider
  and its helpers may read no `const` declared below them, and their state is `var` WITHOUT initialisers
  (`var _shell, _shellMode, _shellLastW;`) — a `var x=null` later in the block would wipe what the early call wrote.
- **`_SHELL_SLOTS` is v2.72's `_DSK_SLOTS` with one home per MODE**: `d` desktop, `t` tablet (defaults to `d`), `n` the
  narrow bar (tablet below 1180, desktop below 860; defaults to `t`, then `d`), `p` phone; a blank entry is the markup
  home. Still MOVED, never cloned, with v2.72's comment-marker way home. A control with the same home on every shell
  lives there in the markup instead (Seed/⟳/Center/Generate in Run, View in the Base map popover — the v2.24 document
  bar is gone). A new control that lives in different places by shell gets a row, not a copy.
- **The phone is a bar, an app bar and ONE sheet whose pages are existing panels, moved**: the domain page is the
  left dock (`#dockLeft`), the context page the Context panel, STORY › Time and Journeys are Explore's sections, MAP is
  the Base map popover + 2D/3D + legend, MENU holds the eight `.mb-panel`s. Every new element is frame only and
  presses a real control or runs `_DSK_CMDS`. The sheet's state has one writer, `_phSheetSet(page,detent)`;
  `_phLayout()` publishes `--ph-cover-b/-r` (how much map the sheet and strip cover) for DS-13, the scale bar, the
  joystick and the grip. A menu row that ran a command closes the sheet unless that command opened a page itself
  (`_phCommandFrom` compares `_ph.serial`).
- **Drill-in is one mechanism for the phone's MENU, the tablet's menus and ☰**: `_dskSubOpen` adds `.mb-drilled` to the
  owner on a touch shell (`_dskDrill()`), and CSS hides every sibling of the open `.mb-sub`, so the row you came
  through is the way back. The desktop keeps v2.72's cascade. A row nested in a panel that sits inside another
  panel is handled once, by its OWN panel (`_dskPanelWire` ignores buttons whose nearest `.mb-panel` is not itself).
- **The touch layer (`_tchDown/_tchMove/_tchUp`) is the only thing that decides what a finger means on the map**, in
  the capture phase of `.canvas-wrap`, for `pointerType==='touch'` only — mouse and pen are untouched. A tap tool's
  press is withheld and REPLAYED (pointerdown, pointerup, click) so every tool keeps its one implementation; a pan
  goes through the existing `panDrag`/`_lodPan` state; ✋'s press reaches no tool; Krita's guard cancels through
  `#view`'s own pointercancel handler. **Any new map tool must say whether it draws** (`_ds13DrawKey`) **or places**
  (`_tchPlaceTool`), or the finger, the latch and thumb zoom will treat it as Inspect.
- **Every way to arm a tool ends in `_dskToolOptsSync()`**, which now also drives DS-13's latch (`_ds13Sync`) and the
  phone's strip (`_phStripSync`). v2.73 added the calls `setRegionMode` and the icon density brush were missing.
- **The docking rule (`_shellDockSync`) runs on every resize, not only on a class change**: a dock floats unless the
  map keeps 520px (Context first, then the left dock), using the docks' own widths (372/304, 400/328 from 1440) and
  the rail's measured width. A floating dock's chevron opens it; a press on the map closes it unless pinned
  (`.dh-pin`), and that press is spent.
- **Pending the owner, built at the stated defaults and labelled**: DS13-Q1 (✋ only with a drawing tool), DS13-Q2
  (one finger pans with a tap tool), and thumb zoom/double-tap off with a drawing or placing tool unless ✋ is on
  (`MAP_INTERACTION_RESEARCH.md` §6). Each is one function: `_ds13DrawKey`, `_tchDown`'s tap branch, `_tchThumbOn`.
- **The v1.19 joystick stays** (the owner's, twice); it stacks above DS-13 and mirrors with Handedness.
- **Probing touch:** Playwright's touch emulation reports `maxTouchPoints` 1, so this file's own `isMobile` (v1.19)
  is false unless the user agent says Android — the probe sets one. Drag with CDP `Input.dispatchTouchEvent` and let
  the finger REST before lifting: a drag lifted at speed is a fling, and Chrome spends the next tap stopping it (no
  click). Scroll a sheet row into view before tapping it; an unclipped rect below the fold is not pressable.
- **RC's older shell probes that pin the v2.31 phone** (`probe_shellui`'s layers-popover and drawer-over-rail checks,
  `probe_domainrail`'s rail-as-drawer-head checks) and `probe_desktopshell.js`'s phone half fail on v2.73 by design:
  the drawer, the ☰ and the rail as its head are what stage 2 replaced. Re-point them at the bar, the sheet and the
  domain tab rather than reading them as regressions.
```
