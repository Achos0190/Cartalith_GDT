## v2.73 (DCC line) — the phone and tablet shells, stage 2: one layout decider, a phone with a bottom bar, one sheet and the pull-out tab, the touch map row (DS-13) and its gestures, and the desktop shell at touch size

Owner: *"Can we upgrade the layout in the html with the new design?"* — then *"A working copy in this repo"* and
*"Everything, staged"* (stage 1 was v2.72's desktop shell; stage 2 is this; stage 3 is the Context panel's modules
and the planner window). Looking at v2.72's phone capture: *"The screenshots on phone don't seem to have the
pull-out tab from the side."* Design: Cartalith_GDT `MOBILE_SHELL_PROPOSAL.md` §2.1–§2.5, §3 and §4, with Rulings
DT, DU, DX, DY (MQ-1, Q10–Q12, PQ6) and EB, and the mockup `design/mobile-shell-2026-10-31/Live.dc.html`.
`tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs v2.72 **ALL IDENTICAL**. Verification is
`tests/perf/probe_mobileshell.js` (**159 assertions**, all real taps, CDP touches and measured geometry
at phone 360×780, 412×915 and 844×390, tablet 768×1024 and 1194×834, the narrow bar at 600, 768 and 1024, and
desktop 700×900 and 1366×768; run on v2.72 it fails **88** — a section that cannot run there fails
once, by name — and passes 30, each a guard v2.72 already meets: no overflow and no duplicate id at any size, the
menu bar at 1194 and 1366, ± stepping 1.35 and ⌖ fitting, a tap selecting, a double-tap not zooming with a drawing
tool armed, the bar not overlapping at 600 and 768, the desktop's docks and zoom column at 1366, a mouse drag not
panning, and no page errors).

- **A flaky suite assertion, disclosed not fixed.** One `tests/run.sh` run of the final bytes failed *"smoothed
  bathymetry variance <= raw seabed variance (9.0e-3 <= 8.9e-3)"*; the next three runs of the same bytes passed
  1280/0. That test builds its world on an unpinned seed — its two variances moved between 4.1e-3 and 1.2e-2 across
  runs of both v2.72 (4.7e-3, 1.2e-2, 4.1e-3) and v2.73 (5.3e-3 to 9.0e-3) — and `test_tail.js`'s own comment there
  records it failing about one run in three on v1.30–v1.32. No engine line moved (`hash_gen1.js` ALL IDENTICAL). If it
  goes red, re-run before believing it, as for v2.25's SST assertion.
- **ONE decider for the layout class (`_shellClassOf`).** MQ-1, answered by Ruling DY item 13: *under 600 wide is
  a phone; under 500 tall in landscape is a phone landscape; everything else is a tablet* — CSS px for dp. This file
  adds the third class the native shell does not need: a window that is not a phone and whose PRIMARY pointer is fine
  keeps stage 1's **desktop**; a coarse one gets the same shell at touch size, the **tablet**. It is re-run on every
  resize, rotation and change of primary pointer, with **hysteresis** (once a class is in force its line moves 16px
  away, so a window dragged onto 600, or a landscape phone whose address bar crosses 500, does not flap) and a
  **keyboard guard** (while a text field has focus and the width has not changed, the class is kept — where a browser
  shrinks the layout viewport for an on-screen keyboard, a landscape tablet would otherwise turn into a phone under the
  caret). It reads the
  LAYOUT viewport (`documentElement.clientWidth`), never `innerWidth`: measured, a 360px phone read **504** at boot,
  because `innerWidth` is the visual viewport, which a mobile browser widens and zooms out whenever anything
  overflows — and it was given the tablet shell. It runs from the top of script block 1 (`_shellEarly`, hoisted), so
  the first paint is already the right shell.
- **CSS asks the class, never the width.** Every v2.24–v2.72 rule that sat in `@media (max-width:860px)` (the phone)
  or `@media not all and (max-width:860px)` (stage 1's desktop) is now keyed on `html[data-shell]`, wrapped in
  `:where()` so it adds **zero specificity** and the cascade is the one the media query gave. Re-keying v2.72 that
  way changed **0 computed styles over 2 305 elements** at 1366×768 and at 1600×900 (a computed-style dump of every
  element, v2.72 against itself re-keyed). The CSS and the slot table therefore cannot disagree about which shell is
  in force, which two deciders (a media query and a script) would. Width-responsive tweaks that are not a class
  (the status chips at 1560/1180/900, the docks at 1440) stay media queries.
- **One control, one home per shell (`_SHELL_SLOTS`, `_shellPlace`).** v2.72's `_DSK_SLOTS` (one desktop slot per
  control across 860px) is generalised to a home per layout MODE — desktop, tablet (defaults to the desktop's),
  the narrow bar (below), phone — with the markup position as the home a mode leaves blank: **46 rows, 36 slots**.
  Still one element moved, never cloned: a comment marker records the markup home on the first move and the way
  back is "insert after the marker". Seed, ⟳ New seed, Center landmasses, Generate world and View now have ONE home
  on every shell (WORLD › Generate › Run; the Base map chip's popover), so their markup moved there and the v2.24
  document bar is gone.
- **The phone (MOBILE_SHELL_PROPOSAL.md §2.1).** A **bottom bar** — MAP · the domain tab · STORY · MENU, 64px of
  targets plus `env(safe-area-inset-bottom)` — that becomes an 80px **left rail** in landscape; an **app bar** of
  the world pill (`#docReadout`, moved: *seed · grid · extent*, or *no world*; it opens MENU › File), Undo, Redo and
  Search (a row over the app bar holding the one Find box, `#findWrap`, moved); and **one sheet** with detents —
  peek **76px** (Ruling DU Q9), half (50% of the space above the bar), full (to just under the app bar) — a head that
  drags between them and a handle that toggles half and full on a tap. In landscape the sheet docks on the right at
  300–400px (§3). Each destination is a page of the sheet, filled by the slot table:
  - **the domain tab** reads GENERATE, CIVIL, CARTO or EXPLORE (still a domain until stage 3); its page is **the left
    dock itself**. A tap opens it; a **380ms hold opens the domain fan** (slide and lift, or tap); a **sideways
    swipe** cycles; the sheet title's **▾** opens the same fan, so the hold is never the only way (§2.1). Fan rows
    run Window › Workspace's own commands, which press the rail's buttons. Four dots show the current domain.
  - **MAP**: the 2D/3D switch, View and the layers list with its opacity (the Base map chip's own popover, drawn
    open) and the legend. The quick pins are NOT here — Ruling DY Q10 moved them to the drawer, and that ruling
    is newer than the brief's summary of §2.1.
  - **STORY**: Time (Explore's Timeline section — Add year, the recorded years, the slider, the filters, the
    collapse/recovery simulator) and Journeys (the journey list and the planner). Only the segments this file has:
    there is no conflict entity (the one "conflict" is a collapse-simulator character) and no note editor (v2.11
    shows a native save's vault links read-only), so CONFLICTS and NOTES are not drawn.
  - **MENU**: the eight desktop menus **row for row** (Ruling DT Q2, Ruling DX Q13) — the eight `.mb-panel`s
    themselves, moved, so one command model (`_DSK_CMDS`) has two renderings — as drill-in lists with **48px rows**;
    Quick first (Save, Settlements, Asset library, Journey planner: rows that run the pins' commands), Status last
    (`#statusBar`, moved). A drill-in level hides its siblings and the row you came through is the way back. No
    accelerator is drawn (none fires on a phone, Ruling DX PQ10). A row that ran a command closes the sheet unless
    the command opened a page of it.
  - **the context page** (Ruling DX PQ13's "selection sheet"): the Context panel aside, moved. It opens itself for
    an Info reading or a NEW label/icon selection — a label's editor otherwise renders on a page nobody is
    looking at. Window › Context panel opens it; Window › Left dock opens the domain page; Window › Timeline and
    Story › Timeline open STORY › Time (on a phone these are commands, not checks).
- **The armed tool's strip (§2.1).** While a tool is armed a strip above the sheet carries its name and one hint
  (`#toolOptsCtx`/`#toolOptsHint`, moved — `_dskToolOptsSync` writes them on every shell), the sculpt tool's own
  settings (the very rows the desktop bar hosts) and **Commit / Discard where the tool drafts** (Sculpt's, Way's,
  Route's). **Arming drops an open sheet to peek.** At peek the domain tab's tap brings the page back to half (a
  judgement: closing it there would take two taps to reach the tools).
- **The quick drawer — the pull-out tab (§2.4; Ruling DY Q10, Q11).** A **44×88 grip** on the map's edge AWAY from
  the dominant hand (left by default), at the middle of the map band left above the sheet and strip (so a half sheet
  never covers it and it never meets the scale bar). A tap, or a drag inward past 24px, opens a drawer of the pins —
  **Save, Settlements, Assets, Planner**, the same four elements the desktop bar draws at its right end. The scrim,
  the ×, a swipe back toward the edge or Esc closes it; a pin closes it after running. A visible tab, never an edge
  swipe (Android takes an edge swipe as Back).
- **The long-press card.** Holding the map **480ms on a phone, 500ms on a tablet** (map_overlay.gd's
  `_PHONE_TOUCH_HOLD_MS` / `_TOUCH_HOLD_MS`, §4) opens the map menu — the right-click handler's own item list,
  extracted as `_civCtxItems` so the hold and the right-click are one implementation: edit, move the viewer to or
  delete a place near the finger, drop a settlement, drop a POI, Info here. It opens beside the finger on the side
  away from the dominant hand, and never under it (on a 412px phone it falls back above or below). Measured on the
  first cut: the card opened under the finger and the lift pressed its first row — a settlement was dropped; now
  the card is placed clear and the one lift-click is swallowed.
- **DS-13, the map's control row (§2.5, Ruling EB).** On every touch shell `#zoomOverlay`'s own four buttons are a
  row at the bottom of the map on the dominant hand's side, 12px in: from the outer edge **⌖ recentre**, the
  **− | + rocker** (two 44px halves, a hairline inset 11px), and **✋ only while a drawing tool is armed** (Sculpt,
  Paint, Territory, Way, Select region, the icon density brush), at the inner end so ⌖ never moves. ± step 1.35 about
  the centre off-LOD (Ruling EB item 2; the existing handlers — under Tiled LOD they keep their pyramid-level
  doubling, in 3D their camera step); ⌖ fits the map. The row rides 12px above the sheet or the strip and hides when
  the band left is shorter than it (§2.5, When it hides). On the tablet it sits at the map's bottom right, 12px in and
  14px up, with the 2D/3D square above it (no 3D in the row). **The latch clears whenever a tool is armed or
  disarmed** (§2.5). The scale bar takes the other side. **Preferences › Handedness** (Right / Left; per viewer, in
  `localStorage` behind try/catch — the page works without it) mirrors the row, the grip, the scale bar, the joystick
  and the card. v2.72 left Handedness out because nothing read it.
- **The v1.19 pan joystick is kept** (the owner's, twice — v1.19, and v1.22: *"on mobile/tablet i think we should
  just have it in all views"*). ✋ takes over its stated role (one-finger panning while a tool holds the finger), but
  removing an owner-requested control is the owner's call; it now stacks above the DS-13 row on the dominant hand's
  side and mirrors with it.
- **Touch gestures (phone and tablet; mouse and pen untouched).** One capture layer on the map decides what a TOUCH
  means before any tool sees it:
  - a **tap-only tool** (Inspect, Info, Settlement, POI, Route, Label, Icon) gets a press only once it is a tap — the
    press is withheld, a drift past 10px **pans** (DS13-Q2's default — **pending the owner**, Ruling EB item 1), a
    lift in place is **replayed** to the tool as the same pointerdown/pointerup/click, so every tool keeps its one
    implementation; a press on a label's or icon's own box goes straight through (it drags that label or icon);
  - a **drawing tool** keeps the finger; with **✋ on** the finger pans, and the press now reaches **no** tool —
    v2.71's latch only took the first handler, so with Territory armed a drag both panned and painted (measured on
    the first cut, fixed);
  - **double-tap zooms ×1.7** at the finger over 240ms, and **double-tap, hold and slide zooms continuously** about
    where it landed — **down in, up out** ("Google Maps style", Ruling EB item 3), doubling every 120px past the slop
    (the mockup's labelled judgement; a device sets its own, MS-10) — and that held press opens no card. Both are on
    with Inspect and Info, and with ✋ on, and **off while a drawing or placing tool is armed** — the placeholder
    `MAP_INTERACTION_RESEARCH.md` §6 put to the owner, **pending DS13-Q1**. While they are on, a tap waits 240ms for a
    second press, so one pair of taps never both acts and zooms;
  - **Krita's guard** (§6): once two fingers have been down, nothing of that touch reaches a tool until every finger
    is up, and a stroke in progress is cancelled (through `#view`'s own pointercancel handler — one list of drag
    states, not a second copy). Pinch and two-finger pan are the existing touch handlers, unchanged.
- **The tablet (§2.2): the desktop shell at touch size.** 48px bars, a 48px rail, **44px targets everywhere**, menus
  with **48px rows whose submenus drill in** instead of cascading, the pins at the tool-options bar's right end (Ruling
  DY Q12), the long-press card at 500ms and the DS-13 row. **Below 1180px the bar is ☰ · File · World · Records ·
  Story** (Ruling DY PQ6): ☰ holds Edit, Assets, Preferences, Window and Help (their panels move into its drill-in
  pages), **World** holds the WORLD domain's rows this file has — Generate world, the Generate | Sculpt category (the
  design's "Pipeline / Sculpt") and Seed & size… — and File leaves out Generate world on that bar (one row per command
  per bar). Measured at 600, 768 and 1024: nothing on the bar overlaps or runs past it (the wordmark yields below
  760px, Find narrows to 120px).
- **The narrow bar on the desktop too, below 860px (a judgement).** MQ-1 moved the phone line to 600, so a 600–860px
  window with a mouse is now the desktop shell — the phone layout until v2.72. Its eight menus measure **802px**
  (v2.72's compact padding), so at 700px the page scrolled sideways; it takes the bar the design made for that width.
- **The docking rule (§2.2, Ruling DU Q8), on the desktop and the tablet — v2.72's open issue.** A dock stays docked
  only while the map keeps **at least 520px**; otherwise it FLOATS: the Context panel first, as a **44px edge tab**,
  then the left dock, as a 44px strip holding its header. A floating dock starts closed, opens OVER the map with a
  shadow from its chevron or Window's row (the map keeps its width), carries a **pin** (drawn only while it floats),
  and closes when the map is pressed unless pinned. v2.72 squeezed the map instead: at 1000px, 1000 − 40 − 372 − 304
  left it 284px. Measured: at 1366 both stay docked (650px of map), at 1194 the Context panel floats, at 768 and 700
  both float.
- **The 44px floor, by construction.** One rule set floors every control the touch shells draw — buttons, selects,
  text and number fields, colour wells, summaries, checkbox labels, pills, and sliders (by `min-height`, which no
  row's height can out-specify: the first cut lost to `.row input[type=range]`'s 22px). Measured by a walk of every
  drawn target on every phone surface — the app bar, the bar, the row, the grip, all four domains' pages with every
  accordion open, MAP, STORY, MENU, the context page, the drawer, the strip and Settings — **336 target measurements,
  none under 44 × 44 on either axis**, and on the tablet's bars, rail, dock and map controls.
- **Windows open as full-height sheets on a phone (§2.1).** Settings already did (v2.28); Credits now does; the City
  Viewer, Route Editor and Settlements windows were already full-screen. Every window's × is a 44px target on a touch
  shell.
- **Find on a phone** opens the page that holds the control (the domain page, STORY, MENU's drill-in path or the
  context page) and scrolls it into view; a chosen result closes the search row.
- **Seen and fixed on the way:** Select region (Region export) is a drawing tool, and arming it left the tool-options
  bar naming the domain — it now names Region select and its hint, on every shell. The status bar's two chips told
  a phone to use "the cog's Settings": there is no cog on any shell now, and Preferences reaches every page.
- **Unchanged by construction:** no engine line moved, so the headless suites and the render hashes are v2.72's.
  Stage 1's desktop at 1366 and wider is untouched (the docks stay docked there, the row stays a column).

**`tests/perf/probe_desktopshell.js` (v2.72's probe) on v2.73**, with v2.71 as the comparison file: 154 passed,
4 failed, against v2.72's 174/0. Every difference, classified:
- *"the version is v2.72 (the tag and the title)"* and *"Help carries the version"* — the version is v2.73. By
  design.
- *"phone 390x760"* and *"phone 412x915: the section ran to the end"* (18 assertions in v2.72's run, 9 per size) —
  that half pins v2.71's phone (the ☰ drawer, the document bar, the header order, the map to the pixel), which stage
  2 replaces; it stops at its first tap on `#panelToggle`, which no shell draws now. By design.
- Every desktop assertion (the menu bar, the keys, the tool-options bar, the rail, the docks, the Context panel, the
  timeline strip, the status bar, the map controls, Find, the busy-key rule) passes unchanged.

**RC's shell and UI probes, run on both files** (8: `shellui`, `domainrail`, `cogmodal`, `assetsbtn`,
`settleparams`, `geninfo`, `worldreset`, `typepreview`). Six give v2.72's result exactly (`cogmodal` 33/0, `geninfo`
21/0, `worldreset` 35/0, `typepreview` 41/0, and `assetsbtn` 13/1 and `settleparams` 28/2, whose failures v2.72
already classified). **Seven assertions change, each pinning the v2.31 phone that stage 2 replaced**:
- `probe_shellui` (18/1 → 16/3): *"the layers popover is a scroller sized to the VISIBLE viewport"* — on a phone the
  popover is the MAP page's, drawn open inside the sheet, and the sheet's body is the scroller; the same probe's *"…and
  its last layer can actually be scrolled into view"*, the behaviour the check was written for, still passes. *"the
  dock sheet draws ABOVE the domain rail"* — there is no drawer and no rail on the phone now.
- `probe_domainrail` (19/1 → 14/6): *"the rail is the drawer's first band"*, *"…spanning the open drawer"*, *"every
  button clears the 44px touch minimum"*, *"…with its glyph AND its label still inside the row"* and *"the drawer
  scrolls"* all measure the rail as v2.31's drawer head on a phone; the domain tab and its fan replace it there, and
  the tablet's rail is 48px wide (this release's own probe measures every phone and tablet target, both axes). The
  sixth failure, *"…still between the tools dock and the map"*, is v2.72's (the stage-1 rail is leftmost).

**What moved where on a phone** (desktop and tablet: v2.72's homes, plus the narrow bar below)

| Control (ids kept) | v2.72 phone (v2.71's layout) | v2.73 phone |
|---|---|---|
| `#panelToggle` (☰) and its drawer of both docks | header; one fixed sheet from the right | not drawn (kept by id); each dock is a page of the sheet |
| `#domainRail` | the drawer's sticky head | not drawn; the domain tab and its fan press its buttons |
| `#dockLeft` (the left dock, newly id'd) | in the drawer | the domain page |
| `#dockRight` (Context panel) | in the drawer, below the left dock | the context page |
| `h1`, `.tag`, `#cogBtn` | header | not drawn (MENU's Preferences and Help reach Settings) |
| `#settlementsHeaderBtn`, `#assetsHeaderBtn`, `#pinSave`, `#pinPlanner` | header (the two toggles); pins not drawn | the quick drawer |
| `#docReadout`, `#phaseChip` | not drawn; header | the world pill |
| `#undoBtn`, `#redoBtn` | header | app bar |
| `#findWrap` | header | the search row (`#phSearch`) |
| `#fileWrap`/`#fileMenu`, the other seven `.mb-panel`s | File ▾ in the header; menus not drawn | MENU, row for row |
| `#seedN`/`#seedLab`, `#reseedBtn`, `#centerBtn`, `#genBtn` | document bar | WORLD › Generate › Run (now their markup home on every shell) |
| `#modeSeg` (View) | document bar | the Base map chip's popover — the MAP page (now its markup home) |
| `#layersFab` (popover open), `#viewDimSeg`, `#legend` | map overlays | the MAP page |
| `#toolRail` band; `#sculptFeatureSeg`, `#sculptFeatHint`, `#sculptModeSeg`, brush rows | band under the document bar; WORLD › Sculpt | band drawn on no shell; the tool strip |
| `#toolOptsCtx`, `#toolOptsHint` | not drawn | the tool strip |
| `#sculptCommitBtn`, `#sculptDiscardBtn`, `#civCommitWayBtn`, `#civCommitRouteBtn` | in the dock | the tool strip, the armed tool's only |
| `#explTimelineSection`, `#explJourneysSec` (newly id'd), `#civPlannerSec` | Explore | STORY › Time and Journeys |
| `#statusBar` (with `#verTag`) | bottom band | MENU › Status |
| `#zoomOverlay` (`#zoomIn`, `#zoomOut`, `#panBtn`, `#zoomReset`) | bottom-right column, touch only | DS-13's row (also the tablet's) |
| `#sculptNavpad` (v1.19 joystick) | bottom right | above the DS-13 row, mirrors with it |

**New elements** (the frame only; each opens an existing control or runs a `_DSK_CMDS` command): `#phWorldPill`,
`#phSearchBtn`/`#phSearch`/`#phSearchClose`, `#phBar` and its four tabs, `#phSheet` and its five pages, `#phMenuRoot`,
`#phStrip`, `#phGrip`, `#phDrawer`, `#phFan`; `#mbMoreWrap` (☰) and `#mbWorldWrap` (World) for the narrow bar;
`#dockLeftPin` and `#ctxPinBtn` for the docking rule; Preferences › Touch › Handedness.

**Not in this build** — each checked by grep, by concept

| Design item | Why it is left out |
|---|---|
| STORY › Conflicts, Notes; MENU's Records › notes rows | No conflict entity and no note editor in this file (above) |
| Two-finger tap undo, three-finger tap redo (Ruling DU Q5) | Not in stage 2's brief; no multi-finger tap handler exists here. The touch layer leaves two-finger touches to the existing pinch |
| Map rotation, the compass | No map rotation exists here |
| The edge swipe for the inspector, haptics, *Pen draws, finger pans*, UI scale, Touch & gestures settings | None exist; the browser exposes no haptics to a file:// page |
| The cartwheel and its ring dock (tablet, §2.4) | The HTML has no cartwheel |
| World › Reset generation parameters, Reset one stage | Neither exists (CHANGELOG v2.72) |
| System back closing the top layer (§4) | A browser's Back leaves the page; this file does not intercept history. Esc closes, in order, the fan, the drawer, the search row and the sheet |
| Holding Undo for its history; the tool chip's fan | No undo history list (v2.72) and no tool fan exist |

**Pending the owner — labelled, not decided:** DS13-Q1 (✋ only with a drawing tool) and DS13-Q2 (one finger pans
with a tap-only tool) are built at their stated defaults (Ruling EB item 1 awaits the answer); thumb zoom and the
double-tap are off with a drawing or placing tool armed unless ✋ is on — `MAP_INTERACTION_RESEARCH.md` §6's
second finding, put to the owner, not settled by it. Each is one function to change (`_tchThumbOn`, `_ds13DrawKey`,
the `tap` branch of `_tchDown`).

**What needs a person on a device:** real touch (the gesture timings — hold 480/500ms, the 240ms tap wait, the 120px
doubling — on glass, MS-10); `env(safe-area-inset-*)` under a notch and a gesture bar (headless insets read 0); an
on-screen keyboard on a landscape tablet (the keyboard guard); Android's own long-press menu and the click after a
fling (both handled, both only provable on a device); rotation of a real phone; WebGL in 3D; a screen reader over the
bar, the sheet and the fan.
