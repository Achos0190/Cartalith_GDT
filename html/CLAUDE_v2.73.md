# CLAUDE.md additions for v2.73 (paste into Cartalith_RC's root `CLAUDE.md`)

Two pastes. Neither changes anything else in the file.

## 1 · The file table

**Replace** the opening of the `Cartalith v2.72 DCC test.html` row,
`**The DCC shell line's current head, not a mainline version.**`, with `Previous DCC-line file.` (the rest of
that row stays as it is — the convention every older row follows), and **insert this row above it**:

```
| `Cartalith v2.73 DCC test.html` | **The DCC shell line's current head, not a mainline version.** Owner: *"Can we upgrade the layout in the html with the new design?"*, *"Everything, staged"*, and on v2.72's phone capture *"The screenshots on phone don't seem to have the pull-out tab from the side."* **STAGE 2 OF 3: THE PHONE AND TABLET SHELLS**, from Cartalith_GDT's `MOBILE_SHELL_PROPOSAL.md` §2.1, §2.2, §2.4, §2.5, §3, §4 (not §2.3, New World) and Rulings DT/DU/DX/DY/EB/EC (EC items 1–4 answered EB's open touch questions). **ONE decider, `_shellClassOf`**: under 600px wide, or under 500px tall in landscape, is a PHONE (MQ-1, Ruling DY); otherwise a coarse primary pointer is a TABLET (the desktop shell at touch size) and a fine one the DESKTOP (stage 1's shell) — re-run on resize, rotation and pointer change, with 16px hysteresis and a keyboard guard, reading `documentElement.clientWidth` (a phone's `innerWidth` read 504 for 360). **CSS keys on `html[data-shell]` through `:where()`, never on the width** (re-keying v2.72 changed 0 computed styles over 2 305 elements at 1366 and 1600). **`_SHELL_SLOTS`** generalises v2.72's table to a home per mode — desktop, tablet, the narrow bar, phone — 47 rows into 40 slots, still one moved element each (one row's phone home follows a state: the Assets toggle is the app bar's while the library is open). The PHONE: a bottom bar (MAP · domain tab · STORY · MENU; a left rail in landscape), an app bar (world pill · Undo · Redo · Search — Redo its own button, a stated departure: no undo history exists to hold it; while the asset library is open the Assets toggle itself joins it, reading ← Map, and MAP, the domain tab and STORY leave the library), ONE sheet (peek 76 · half · full) whose pages are the left dock, the Context panel, Explore's Timeline and Journeys, the Base map popover with the scale bar's switch, and the eight menu panels row for row (two departures stated for the owner: Window's Status bar and Domain rail are not drawn, and its Left dock and Context panel are commands; a row that cannot run says why on a second line); the domain fan (380ms hold, swipe, the title's ▾) over a scrim, so its dismissing tap reaches nothing; the armed tool's strip (Sculpt's feature picker and Commit/Discard, two rows; its brush and mode at the top of WORLD › Sculpt, and the picker too in landscape, in the side sheet beside the strip); **the quick drawer on a 44×88 grip** (the pull-out tab, away from the dominant hand); the long-press card (the right-click menu's own items). **DS-13** on every touch shell: ⌖ · −|+ · ✋ (✋ only with a drawing tool), mirrored by Preferences › Handedness. **Touch gestures** in one capture layer: a tap tool's press is withheld and replayed, a drift pans, double-tap ×1.7 and thumb zoom (down = in) with Inspect/Info or ✋, off with a drawing or placing tool, Krita's guard — DS13-Q1/Q2 and both research findings as **Ruling EC** items 1–4 answered them; ✋ keeps the press and its click from every tool. The TABLET: 44px floor (every checkbox's label too, however written), 48px drill-in menus, Sculpt's Discard and Commit at the bar's end, below 1180px ☰ · File · World · Records · Story (the desktop below 860px too). **The docking rule** on desktop and tablet: a dock floats (closed, pin, closes on a map press) unless the map keeps 520px. `tests/run.sh` 1280/0, `run_um.sh` 882/0, `hash_gen1.js` vs v2.72 **ALL IDENTICAL**. Verify with `tests/perf/probe_mobileshell.js` (@@N@@ assertions over the @@V@@ viewports it prints, every screen reached by taps; @@F72@@ fail on v2.72, @@F1@@ on this version's first cut, @@FR1@@ on the fix pass's final bytes). |
```

## 2 · The section

**Insert above** `### The desktop shell, stage 1: menus, tool settings only, the Context panel, the year everywhere (v2.72, DCC-line file only)`
(DCC-line sections run newest first):

```markdown
### The phone and tablet shells, stage 2: one decider, one sheet, the pull-out tab, DS-13 and the touch layer (v2.73, DCC-line file only)

Owner: *"Can we upgrade the layout in the html with the new design?"*, then *"Everything, staged"*; on v2.72's phone
capture, *"The screenshots on phone don't seem to have the pull-out tab from the side."* Stage 2 is the phone and the
tablet; stage 3 is the Context panel's modules and the planner window. Design: Cartalith_GDT `MOBILE_SHELL_PROPOSAL.md`
§2.1, §2.2, §2.4, §2.5, §3, §4 and Rulings DT, DU, DX, DY, EB and EC (items 1–4); §2.3 (New World) is not built.
`tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs v2.72 **ALL IDENTICAL**; verification is
`tests/perf/probe_mobileshell.js` (@@N@@ assertions over the @@V@@ viewports it prints on its last lines, across the
three classes; every screen is reached by taps, CDP touches, the mouse or the keyboard, and `page.evaluate` only sets
up, reads, measures drawn rects, or resets between steps; run on v2.72 it fails @@F72@@, on this version's first
cut @@F1@@, and on the 2026-11-02 fix pass's final bytes, before fix round 2, @@FR1@@).

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
  (`var _shell, _shellMode, _shellLastW, _shellOrient;`) — a `var x=null` later in the block would wipe what the early
  call wrote.
- **`_SHELL_SLOTS` is v2.72's `_DSK_SLOTS` with one home per MODE**: `d` desktop, `t` tablet (defaults to `d`), `n` the
  narrow bar (tablet below 1180, desktop below 860; defaults to `t`, then `d`), `p` phone; a blank entry is the markup
  home. Still MOVED, never cloned, with v2.72's comment-marker way home. A control with the same home on every shell
  lives there in the markup instead (Seed/⟳/Center/Generate in Run, View in the Base map popover — the v2.24 document
  bar is gone). A new control that lives in different places by shell gets a row, not a copy. A home that depends on
  a STATE, not only the mode, is a row key too (`pa`: the Assets toggle's phone home while the library is open; `pl`:
  Sculpt's picker's in phone landscape), and whatever changes that state re-places the one row with
  `_shellPlaceRow(id)`, which keeps the table's order inside the slot. A handler that asks where its own button sits must ask the event's path, not the DOM: a click can move
  the button it landed on (the drawer's pin-close read the DOM and left the drawer open over the library).
- **The phone is a bar, an app bar and ONE sheet whose pages are existing panels, moved**: the domain page is the
  left dock (`#dockLeft`), the context page the Context panel, STORY › Time and Journeys are Explore's sections, MAP is
  the Base map popover + 2D/3D + legend, MENU holds the eight `.mb-panel`s. Every new element is frame only and
  presses a real control or runs `_DSK_CMDS`. The sheet's state has one writer, `_phSheetSet(page,detent)`;
  `_phLayout()` publishes `--ph-cover-b/-r` (how much map the sheet and strip cover) for DS-13, the scale bar, the
  joystick and the grip. A menu row that ran a command closes the sheet unless that command opened a page itself
  (`_phCommandFrom` compares `_ph.serial`).
- **Drill-in is one mechanism for the phone's MENU, the tablet's menus, ☰ and the narrow desktop bar**: `_dskSubOpen`
  adds `.mb-drilled` to the owner on a touch shell or the desktop below 860px (`_dskDrill()`), and CSS hides every
  sibling of the open `.mb-sub`, so the row you came through is the way back. The desktop at 860px and wider keeps
  v2.72's cascade. A row nested in a panel that sits inside another
  panel is handled once, by its OWN panel (`_dskPanelWire` ignores buttons whose nearest `.mb-panel` is not itself).
- **The touch layer (`_tchDown/_tchMove/_tchUp`) is the only thing that decides what a finger means on the map**, in
  the capture phase of `.canvas-wrap`, for `pointerType==='touch'` only — mouse and pen are untouched. A tap tool's
  press is withheld and REPLAYED (pointerdown, pointerup, click) so every tool keeps its one implementation; a pan
  goes through the existing `panDrag`/`_lodPan` state; ✋'s press reaches no tool; Krita's guard cancels through
  `#view`'s own pointercancel handler. **Any new map tool must say whether it draws** (`_ds13DrawKey`) **or places**
  (`_tchPlaceTool`), or the finger, the latch and thumb zoom will treat it as Inspect.
- **Every way to arm a tool ends in `_dskToolOptsSync()`**, which now also drives DS-13's latch (`_ds13Sync`) and the
  phone's strip (`_phStripSync`). v2.73 added the calls `setRegionMode` and the icon density brush were missing, and
  (fix round 2) the icon gallery's tile pick and its category select — Stop placing left ✋ drawn, and re-arming from
  the gallery left it hidden. Arm a tool by TAPS when you test this: a probe that arms it through `page.evaluate`
  goes round the very path that forgot the call.
- **The docking rule (`_shellDockSync`) runs on every resize, not only on a class change**: a dock floats unless the
  map keeps 520px (Context first, then the left dock), using the docks' own widths (372/304, 400/328 from 1440 —
  stage 1's on the tablet too, a stated departure from §3's tablet widths) and the rail's measured width. A floating dock's chevron opens it; a press on the map closes it unless pinned
  (`.dh-pin`), and that press is spent.
- **Ruling EC items 1–4 (owner, 2026-11-01) are built as answered**: DS13-Q1 (✋ only with a drawing tool), DS13-Q2
  (one finger pans with a tap tool), Krita's guard (nothing more of a two-finger touch reaches a tool — what the
  first finger already painted stays), and thumb zoom/double-tap off with a drawing or placing tool unless ✋ is on.
  Each is one function: `_ds13DrawKey`, `_tchDown`'s tap branch, `_tchMultiStart`, `_tchThumbOn`. **✋ keeps the
  browser's click from the tools as well as the press** (`_tch.swallowClick` in the latch branch): stopping
  pointerdown does not stop the click, and `#view`'s click handler places icons.
- **The phone's frame has a z-order, and anything floating over the map must sit under it on a phone**: DS-13 21, the
  grip 22, a place's edit popup 31 (v0.90's 61 elsewhere), the strip 32, the sheet 33, the bar 34, the scrim 35 (39
  under the fan), the drawer 36 (the open drawer's grip 37), the fan 40, the search row 41. The first cut left the
  popup at 61, over the sheet, and MENU's rows took the popup's taps. **A layer that closes on an outside tap lies
  over a scrim, and the scrim's click closes it** — closing on the press took the scrim away and the click fell
  through: the fan's dismissing tap dropped a settlement (fix round 2).
- **The phone's landscape is `html[data-orient]`, written by the decider under its keyboard guard — never
  `@media (orientation)`.** The media query let a keyboard turn a typing portrait phone's bar into the landscape rail
  while the guard kept the class. `_phLand()` reads the same value.
- **The strip shows the armed tool's settings only (Ruling DZ), and stays two rows.** It holds the name, the hint,
  Commit/Discard and, for Sculpt, its feature picker as one row of chips — kept there whatever is armed, so its group
  (`#phStripTool`) is hidden for every other tool; a future per-tool group must be gated the same way. Sculpt's other
  settings go to the top of WORLD › Sculpt (`#phSculptOpts`, the phone's domain page — §2.1's "full options at the
  top of the domain sheet"): all of them in the strip made it 323px tall at 412x915 and left 100px of map at 844x390
  (fix round 2); in landscape the picker goes there too (`pl`, re-placed when the phone turns), because beside the
  side sheet a chip row next to Commit and Discard had 41px at 740x360. Anything that changes the strip's height
  re-lays the map out through the `ResizeObserver` on
  `#phStrip` — do not add `_phLayout()` calls after content changes. **The map's floating controls thin out at §3's
  120px of free map** (`PH_FREE_MIN`: the row, the scale bar, the grip), and the v1.19 joystick, taller than all of
  them, has its own fit rule (`ph-nav-hide`); a new floating map control needs one or the other.
- **A floating dock's head gives every button its own grid column.** Two 44px buttons stacked in one cell of a
  ~45px head overlap completely: the Context panel's pin covered its chevron, so a tap meant to close pinned it.
- **The quick drawer closes from the tab that opened it**: while open, the grip rides the drawer's edge above the
  scrim, and the drawer is `touch-action: pan-y` (with the default, the browser cancelled its sideways swipe).
- **In Assets mode a phone's way out is the Assets toggle ITSELF, moved to the app bar** (`pa`, above): its drawer home
  opens from a grip on the map, which the library hides. Never a second "← Map" button — that is v2.24's stand-in,
  which v2.46 retired as v1.57's one-control-two-surfaces defect; the fix pass brought it back as `#phAssetsBack`
  and fix round 2 removed it. **No page of the sheet may open empty**: MAP, the domain tab, STORY and the context
  page leave the library first (`_phSheetSet`), because what they show is what Assets mode hides; MENU works inside
  it. Leaving the library by a domain switch goes through `_carExitAssetsMode`, the one way out (`_applyDomainUI`
  undid only part of the entry before: the toggle kept reading "← Map" and the Base map chip stayed hidden). The
  library is one column on a phone.
- **The narrow DESKTOP bar (a mouse below 860px) drills its submenus in**, like the tablet's narrow bar (`_dskDrill`),
  because a cascade beside its row runs off the right edge there.
- **The v1.19 joystick stays** (the owner's, twice); it stacks above DS-13 and mirrors with Handedness.
- **The 44px floor covers every checkbox's or radio's LABEL, however the label is written** — wrapping it, or naming
  it by `for=` right before or after it (`:has()` rules) — not `label.chk` alone, which missed the icon density
  brush's `<label for>` (13x13 box, 78x32 label). A probe measures a checkbox through `input.labels`.
- **A command that can be disabled says why**: an `enabled` entry in `_DSK_CMDS` carries a `why()` that reads the
  control's real cause — the one writer of its disabled state, named beside it — and returns '' otherwise; the touch
  shells draw it as the row's second line (`_dskWhySet`, §3's #ebc080). The desktop draws no span.
- **Touch shells take away page zoom** (`touch-action: pan-x pan-y` on `body *`): a pinch on the sheet zoomed the
  whole page to 5x. Anything that handles its own gesture keeps a narrower value (the map, sliders, the joystick, the
  sheet's head, the tabs, the drawer). **A scrolling window keeps its × in a sticky head** (Settings, Credits,
  Settlements) with `scroll-padding-top`, so nothing scrolled into view stops under it.
- **Probing touch:** Playwright's touch emulation reports `maxTouchPoints` 1, so this file's own `isMobile` (v1.19)
  is false unless the user agent says Android — the probe sets one. Drag with CDP `Input.dispatchTouchEvent` and let
  the finger REST before lifting: a drag lifted at speed is a fling, and Chrome spends the next tap stopping it (no
  click). Scroll a sheet row into view before tapping it; an unclipped rect below the fold is not pressable.
  **Measure DRAWN rects** (clipped by every ancestor that clips them, never asking `<body>`, whose
  `overflow:hidden` makes everything look clipped) **against the viewport you set**: a mobile emulation widens its
  layout viewport to fit an overflow, so `clientWidth` read 401 on a 360 phone. **Skip the content of a closed
  `<details>`**: Chrome still gives it layout boxes, so a walk counts invisible controls as 44px passes. **Navigate by
  taps** — a probe that reaches a screen by calling `_phSheetSet` or `_setDomain` proves the screen renders, not that
  it can be reached. **Measure what a person reads**: a strip's height means nothing without the map band it leaves
  (the probe's bars: ≥ 50% of the map in portrait, ≥ 200px in landscape, every drawn map control inside the band).
- **RC's older shell probes that pin the v2.31 phone** (`probe_shellui`'s layers-popover and drawer-over-rail checks,
  `probe_domainrail`'s rail-as-drawer-head checks) and `probe_desktopshell.js`'s phone half fail on v2.73 by design:
  the drawer, the ☰ and the rail as its head are what stage 2 replaced. Re-point them at the bar, the sheet and the
  domain tab rather than reading them as regressions.
```
