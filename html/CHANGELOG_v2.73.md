## v2.73 (DCC line) — the phone and tablet shells, stage 2: one layout decider, a phone with a bottom bar, one sheet and the pull-out tab, the touch map row (DS-13) and its gestures, and the desktop shell at touch size

Owner: *"Can we upgrade the layout in the html with the new design?"* — then *"A working copy in this repo"* and
*"Everything, staged"* (stage 1 was v2.72's desktop shell; stage 2 is this; stage 3 is the Context panel's modules
and the planner window). Looking at v2.72's phone capture: *"The screenshots on phone don't seem to have the
pull-out tab from the side."* Design: Cartalith_GDT `MOBILE_SHELL_PROPOSAL.md` §2.1, §2.2, §2.4 and §2.5, §3 and §4,
with Rulings DT, DU, DX, DY (MQ-1, Q10–Q12, PQ6), EB and EC (items 1–4, which answered EB's open touch questions),
and the mockup `design/mobile-shell-2026-10-31/Live.dc.html`. §2.3 (New World, kept as shipped, plus auto-populate)
is not in this build, and three other design items are not either: *Not in this build* lists each with its reason
(the first cut cited "§2.1–§2.5" and listed none of them).
`tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs v2.72 **ALL IDENTICAL**. Verification is
`tests/perf/probe_mobileshell.js`: **279 assertions** over the **16 viewports** it lists on its last lines
(phone 360×780, 375×812, 412×915, 844×390 and 740×360, the 412×915 phone turned to 915×412 and squeezed to 412×380
by a keyboard; tablet 768×1024, 1024×768 and 1194×834; the narrow tablet bar at 600×960; desktop 600, 700, 870 and
900 wide and 1366×768). It **navigates the shell only by real input** — touchscreen taps, CDP touches for drags,
holds and two fingers, the mouse, the keyboard — and uses `page.evaluate` for four things, each marked where it
happens: SET-UP a person would take minutes over (a world; a settlement under the finger; the icon density brush
armed from the gallery in one older section — the fix round's own ✋ section arms it by taps; Territory's radius
and faction; every accordion opened for a floor walk, the Settlements window's included; Handedness in storage
before a fresh page), READS of what an input changed, the drawn-rect measure (a control's rect clipped by whatever
clips it, which is what a finger can reach), and RESETS between steps after their assertions (the theme put back,
a window a step opened closed, a selection cleared), on which no assertion rests. Run on v2.72 it fails
**133** and passes 37 (a section that cannot run there fails once, by name); run on this version's
**first cut**, before the fix pass below, it fails **87**; run on the fix pass's final bytes (`52f7d61b…`),
before fix round 2, it fails **38** — each verifier's defect fails its own section on the build it was found in.

- **This entry's first cut overstated its own method, and the probe's header did too** (found by the 2026-11-02
  verifiers). It said "159 assertions, all real taps" and "at seven sizes", and the probe's header said a
  `page.evaluate` was "never [used] to navigate the shell" — but the floor walk, Krita's guard and Handedness's reset
  reached their screens by calling `_phSheetSet`, `_setDomain`, `_civSetTool`, `_setSculptCategory` and
  `_openSettings`, and the probe booted nine viewports. Those steps are taps now, and the probe prints its own
  viewport count. Its overflow check could not fail for any shell element either (`body{overflow:hidden}` made every
  ancestor walk end in "clipped", so 32 of 32 candidates were skipped), and its 44px floor read unclipped rects (a
  button with 10px of itself on screen passed); both now measure drawn rects, against the viewport the probe set.

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
  the narrow bar (below), phone — with the markup position as the home a mode leaves blank: **47 rows, 40 slots**
  (counted from the file's `_SHELL_SLOTS` by a script, 2026-11-02; the fix pass added the scale bar's switch, fix
  round 2 three slots: the top of WORLD › Sculpt, the tablet bar's Commit and Discard, and the Assets toggle's place
  in the app bar). Still one element moved, never cloned: a comment marker records the markup home on the first move
  and the way back is "insert after the marker". Two rows have a phone home that depends on a state, not only on
  the mode: the Assets toggle's is the app bar while the library is open (`pa`), and Sculpt's picker's is the top
  of WORLD › Sculpt in landscape (`pl`); each is re-placed alone by `_shellPlaceRow`, which keeps the table's order
  inside the slot. Seed, ⟳ New seed, Center landmasses, Generate world and View now have ONE home
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
    run Window › Workspace's own commands, which press the rail's buttons. Four dots show the current domain. **The
    fan lies over a scrim** (z 39, as the mockup draws one), so the tap that dismisses it is spent on the scrim. Fix
    round 2: the first cut closed the fan from a capture-phase press and let that press through — with Settlement
    armed, a tap meant only to dismiss the fan dropped a settlement (which `_civDropPlace` does not put on the undo
    stack), with Territory armed it painted 81 cells, and a tap where + is zoomed.
  - **MAP**: under one View caption the 2D/3D switch, the base map's View (Biome · Relief · Height · Shade) and the
    **Scale bar** switch — CARTO › Map style's own `#scaleBarChk` row, moved there on a phone, since the design's MAP
    holds "view (2D/3D, North up, Fit world, grid, scale bar)" — then the layers list with its opacity (the Base map
    chip's own popover, drawn open) and the legend. The design's other MAP rows are in *Not in this build*. The quick
    pins are NOT here — Ruling DY Q10 moved them to the drawer, and that ruling is newer than the brief's summary of
    §2.1.
  - **STORY**: Time (Explore's Timeline section — Add year, the recorded years, the slider, the filters, the
    collapse/recovery simulator) and Journeys (the journey list and the planner). Only the segments this file has:
    there is no conflict entity (the one "conflict" is a collapse-simulator character) and no note editor (v2.11
    shows a native save's vault links read-only), so CONFLICTS and NOTES are not drawn.
  - **MENU**: the eight desktop menus row for row, as Ruling DT Q2 and Ruling DX Q13 ask — the eight `.mb-panel`s
    themselves, moved, so one command model (`_DSK_CMDS`) has two renderings — **with two departures the owner has
    not ruled on**, stated in *Not in this build* for an answer: Window › Status bar and Window › Domain rail are not
    drawn (this file leaves out a row with nothing behind it, and a phone has neither band — but DX Q13's one ruled
    exception is PQ10's two rows, and the mockup draws both), and Window › Left dock and Context panel are commands,
    not checks (PQ13 says the Context panel row stays a check that "shows or hides the selection sheet"). The first
    cut presented the two drops as covered by DT Q2 and DX Q13; neither covers them. They are drawn as drill-in lists
    with **48px rows**;
    Quick first (Save, Settlements, Asset library, Journey planner: rows that run the pins' commands), Status last
    (`#statusBar`, moved). A drill-in level hides its siblings and the row you came through is the way back. No
    accelerator is drawn (none fires on a phone, Ruling DX PQ10). A row that ran a command closes the sheet unless
    the command opened a page of it. **A row that cannot run says why** (§3: "drawn with its reason as a second line
    in #ebc080, never only greyed"; fix round 2 — the first cut only greyed it at .45): each `_DSK_CMDS` entry that can
    be disabled carries a `why` that reads the real cause — the one writer of that control's disabled state — and
    the touch shells draw it under the label (MENU › Edit with nothing done reads *Nothing to undo yet*, *Nothing to
    redo*, *Select a settlement or point of interest first*). The tablet's menus draw it too; the desktop keeps
    v2.72's rows exactly (there, Ctrl+R on a finalized world still raises its reason as an alert, `refused`). The light theme's reason colour is a
    labelled judgement (#7a4f0e, 6.25:1), since #ebc080 on its panel is 1.49:1.
  - **the context page** (Ruling DX PQ13's "selection sheet"): the Context panel aside, moved. It opens itself for
    an Info reading or a NEW label/icon selection — a label's editor otherwise renders on a page nobody is
    looking at. Window › Context panel opens it; Window › Left dock opens the domain page; Window › Timeline and
    Story › Timeline open STORY › Time (on a phone these are commands, not checks).
- **The armed tool's strip (§2.1).** While a tool is armed a strip above the sheet carries its name and one hint
  (`#toolOptsCtx`/`#toolOptsHint`, moved — `_dskToolOptsSync` writes them on every shell), **for Sculpt its feature
  picker as one row of chips** (the very element the desktop bar hosts, in a group, `#phStripTool`, that is drawn for
  Sculpt alone) and **Commit / Discard where the tool drafts** (Sculpt's, Way's, Route's). **Sculpt's other
  settings — the feature's hint, Freehand's mode, Brush size with its km readout, Hardness and Intensity — sit at
  the top of WORLD › Sculpt**: first on the phone's domain page, above its tools, drawn while Sculpt is armed, one
  tap away on the domain tab (§2.1: "Tapping the strip's tool chip opens the full options at the top of the domain
  sheet"; the mockup's domain sheet leads with them too).
  Fix round 2: the first cut put every Sculpt setting in the strip — 323px tall at 412×915 and 226px at 844×390,
  leaving a 100px map band in landscape, against the mockup's 66px strip. **In landscape the picker joins the rest
  at the top of WORLD › Sculpt**, in the side sheet beside the strip, as the mockup's landscape draws it (a state
  in the slot table, `pl`, re-placed on a turn): beside the sheet, a chip row next to Commit and Discard had 41px
  at 740×360. Now the strip is two rows in portrait (three at 360 wide) — 108px at 412×915, 124px at 360×780 —
  and in landscape one row, 58px at 844×390 and 915×412 (two, 74px, at 740×360), leaving 68–81% of the map's height
  above it at those five sizes (measured at seeds 777 and 12345, the same both). **Arming drops
  an open sheet to peek.** At peek the domain tab's tap brings the page back to half (a judgement: closing it there
  would take two taps to reach the tools). The map's row and grip re-place themselves whenever the strip changes
  height (a `ResizeObserver` on it), since a longer hint can arrive after the layout has run.
- **The quick drawer — the pull-out tab (§2.4; Ruling DY Q10, Q11).** A **44×88 grip** on the map's edge AWAY from
  the dominant hand (left by default), at the middle of the map band left above the sheet and strip (so a half sheet
  never covers it and it never meets the scale bar). A tap, or a drag inward past 24px, opens a drawer of the pins —
  **Save, Settlements, Assets, Planner**, the same four elements the desktop bar draws at its right end. While it is
  open **the grip rides the drawer's edge** (the mockup's form), above the drawer and the scrim, so the tab that
  opened it closes it — a tap, or a drag back toward the edge; the scrim, the ×, a swipe on the drawer back toward
  the edge (the drawer is `touch-action: pan-y`, so a sideways swipe reaches it) or Esc close it too; a pin closes it
  after running. In landscape, left-handed, the grip and the drawer take the side sheet's edge rather than sitting
  under it. A visible tab, never an edge swipe (Android takes an edge swipe as Back).
- **The long-press card.** Holding the map **480ms on a phone, 500ms on a tablet** (map_overlay.gd's
  `_PHONE_TOUCH_HOLD_MS` / `_TOUCH_HOLD_MS`, §4) opens the map menu — the right-click handler's own item list,
  extracted as `_civCtxItems` so the hold and the right-click are one implementation: edit, move the viewer to or
  delete a place near the finger, drop a settlement, drop a POI, Info here. **This departs from §2.1 and the
  mockup**, whose card is a sample pin with per-domain actions and the ground's readouts: *Not in this build* says
  why, row by row. It opens beside the finger on the side away from the dominant hand, and never under it (on a
  412px phone it falls back above or below), and it closes when the window's width changes (a rotation), rather
  than staying where portrait put it. Measured on the first cut: the card opened under the finger and the lift
  pressed its first row — a settlement was dropped; now the card is placed clear and the one lift-click is
  swallowed.
- **DS-13, the map's control row (§2.5, Ruling EB).** On every touch shell `#zoomOverlay`'s own four buttons are a
  row at the bottom of the map on the dominant hand's side, 12px in: from the outer edge **⌖ recentre**, the
  **− | + rocker** (two 44px halves, a hairline inset 11px), and **✋ only while a drawing tool is armed** (Sculpt,
  Paint, Territory, Way, Select region, the icon density brush), at the inner end so ⌖ never moves. ± step 1.35 about
  the centre off-LOD (Ruling EB item 2; the existing handlers — under Tiled LOD they keep their pyramid-level
  doubling, in 3D their camera step); ⌖ fits the map. The four glyphs are one size (§2.5, "one glyph size"; fix
  round 2 — − and + were 21px, ✋ 18px by an inline style and ⌖ 16px; 20px is a labelled judgement), and the
  double-tap's ×1.7 eases `1−(1−t)³` over its 240ms, as 06-phone.md §7 specifies (the first cut stepped linearly).
  The row rides 12px above the sheet or the strip; it, the scale bar and the grip hide when the band of map left
  above them is under §3's 120px (§2.5, When it hides; §3: "thin out below 250 / 120 dp of free map" — the mockup
  hides them below 120, and its 250 is computed and drawn by nothing, so only 120 is built), and the v1.19 joystick
  hides when its own stack does not fit. Fix round 2: the first cut hid the row and the grip each at its own size
  and gave the joystick no rule, so with Sculpt armed at 844×390 it sat at y 10..88 under the app bar, whose
  buttons took its taps. On the tablet it sits at the map's bottom right, 12px in and
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
    press is withheld, a drift past 10px **pans** (DS13-Q2, answered by **Ruling EC item 2**, "Yes, drag pans"), a
    lift in place is **replayed** to the tool as the same pointerdown/pointerup/click, so every tool keeps its one
    implementation; a press on a label's or icon's own box goes straight through (it drags that label or icon);
  - a **drawing tool** keeps the finger; with **✋ on** the finger pans, and neither the press nor the browser's
    click after a tap reaches any tool — v2.71's latch only took the first handler, so with Territory armed a drag
    both panned and painted (measured on the first cut, fixed), and the first cut still let a tap's click through,
    so with the icon density brush armed one tap under ✋ placed an icon (fixed in the fix pass). ✋'s
    `aria-pressed` follows the latch;
  - **double-tap zooms ×1.7** at the finger over 240ms, and **double-tap, hold and slide zooms continuously** about
    where it landed — **down in, up out** ("Google Maps style", Ruling EB item 3), doubling every 120px past the slop
    (the mockup's labelled judgement; a device sets its own, MS-10) — and that held press opens no card. Both are on
    with Inspect and Info, and with ✋ on, and **off while a drawing or placing tool is armed** (**Ruling EC item 4**,
    "Off unless ✋ is on"; `MAP_INTERACTION_RESEARCH.md` §6). While they are on, a tap waits 240ms for a second
    press, so one pair of taps never both acts and zooms;
  - **Krita's guard** (§6; **Ruling EC item 3**, "nothing more of that touch reaches a drawing tool"): once two
    fingers have been down, nothing more of that touch reaches a tool until every finger is up, and the tool's drag
    state is cancelled through `#view`'s own pointercancel handler (one list of drag states, not a second copy).
    It does not undo what the FIRST finger already did before the second arrived: a direct-paint tool (Territory,
    Paint) keeps that dab — the 2026-11-02 verifier measured 29 cells for a pinch whose fingers land together or
    20ms apart, against v2.72's 182 and 91 — while Sculpt's stroke, a draft until the lift, adds no stamp. Pinch and
    two-finger pan are the existing touch handlers, unchanged.
  - **Ruling EC (owner, 2026-11-01) answered DS13-Q1, DS13-Q2 and both research findings at exactly the defaults this
    build had made** (its own words: "The HTML's v2.73 already builds items 1–4 as its defaults"), so nothing changed
    when it landed; the first cut of this entry called them pending.
- **The tablet (§2.2): the desktop shell at touch size.** 48px bars, a 48px rail, **44px targets everywhere**, menus
  with **48px rows whose submenus drill in** instead of cascading, the pins at the tool-options bar's right end (Ruling
  DY Q12), the long-press card at 500ms and the DS-13 row (its rocker's halves touching, gaps 10px). **With Sculpt
  armed, its Discard and Commit sit at the end of the tool-options bar** beside its brush (fix round 2: the design's
  tablet bar draws DISCARD and COMMIT beside the feature chips; the first cut left them only at the foot of the left
  dock — at 1194×834 Commit was laid at y 1454 in a 598px dock scroller). Below 1180px the bar takes a third row for
  them while Sculpt is armed. The desktop keeps them in the dock this stage, as v2.72 did. The tool hint
  says Tap and hold for click and right-click but **keeps its keys** (Ctrl+R, Esc): a tablet's menus draw
  accelerators "for a keyboard" (§2.2), and only the phone, which has none, drops them (Ruling DX PQ10). **Below
  1180px the bar is ☰ · File · World · Records ·
  Story** (Ruling DY PQ6): ☰ holds Edit, Assets, Preferences, Window and Help (their panels move into its drill-in
  pages), **World** holds the WORLD domain's rows this file has — Generate world, the Generate | Sculpt category (the
  design's "Pipeline / Sculpt") and Seed & size… — and File leaves out Generate world on that bar (one row per command
  per bar). Measured at 600, 768 and 1024: nothing on the bar overlaps or runs past it (the wordmark yields below
  760px, Find narrows to 120px).
- **The narrow bar on the desktop too, below 860px (a judgement).** MQ-1 moved the phone line to 600, so a 600–860px
  window with a mouse is now the desktop shell — the phone layout until v2.72. The full bar's content — the wordmark,
  the eight menus (469px of it), Undo, Redo and Find — measures **812px** with a mouse at 860–1000px (the header's
  max-content width, measured 2026-11-02; the first cut said "its eight menus measure 802px (v2.72's compact
  padding)", wrong on both counts — that compact padding has never applied, see the fix pass), so at 700px the page
  scrolled sideways; it takes the bar the design made for that width, and **its submenus drill in** like the
  tablet's narrow bar (a cascade beside its row ran off the right edge: File › Export opened at 327..667 in a 600px
  window).
- **The docking rule (§2.2, Ruling DU Q8), on the desktop and the tablet — v2.72's open issue.** A dock stays docked
  only while the map keeps **at least 520px**; otherwise it FLOATS: the Context panel first, as a **44px edge tab**,
  then the left dock, as a 44px strip holding its header. A floating dock starts closed, opens OVER the map with a
  shadow from its chevron or Window's row (the map keeps its width), carries a **pin** (drawn only while it floats,
  in a column of its own between the text and the chevron, as the left dock's header draws it), and closes when the
  map is pressed unless pinned. v2.72 squeezed the map instead: at 1000px, 1000 − 40 − 372 − 304
  left it 284px. Measured: at 1366 both stay docked (650px of map), at 1194 the Context panel floats, at 768 and 700
  both float.
- **The 44px floor, by construction.** One rule set floors every control the touch shells draw — buttons, selects,
  text and number fields, colour wells, summaries, pills, sliders (by `min-height`, which no row's height can
  out-specify: the first cut lost to `.row input[type=range]`'s 22px), and **every checkbox's or radio's label,
  however the label is written**: wrapping it, or naming it by `for=` right before or after it (`:has()`). Fix round
  2: the first cut floored `label.chk` alone and called that "checkbox labels" — the icon density brush, a
  `<label for>` beside a bare box, measured 13×13 with a 78×32 label at 412×915 and 90×15 at 1194×834, and the
  Settlements window's *star fort* and *ruined* and the library's scatter switches were not floored either; the
  probe's walk only measured `label.chk`, so it could not see them. Now the walk measures every checkbox and radio
  through `input.labels` (the largest drawn label, or the bare box when it has none). Measured by that walk, reached
  by taps, of every target as it is DRAWN — scrolled into view, clipped by whatever clips it — on every phone surface:
  the app bar, the bar, the row, the grip, all four domains' pages with every accordion open, WORLD › Sculpt with
  Sculpt armed, MAP, both STORY panes, MENU's root and each of its eight menus and their submenus, the context page
  with a label's editor in it, the drawer, the strip for Sculpt and for Freehand, a settlement's popup, Find's
  results, the long-press card and Settings — **513 target measurements over 31 surfaces, none drawn
  under 44 × 44 on either axis** — and the Settlements window's four tabs at 360×780 (every slider keeping at least
  120px of track there), and on the tablet's bars, rail, dock and map controls. (The first cut's walk read unclipped
  rects over fewer surfaces — "336, none under 44" — and missed the popup's 🎲, cut to 22px on every shell since
  before v2.72; it is whole now.)
- **The Settlements window on a phone: the label above its slider** (fix round 2, a regression the first cut made).
  Its rows give the label a fixed width (150px in Global; 118px and 190px in Types) beside a 62px value and a reset
  ↺ that is hidden until the value changes. Once the floor sized that hidden ↺ at 44px, the Global sliders were 12px
  wide at 360×780 (30px in v2.72) and a 60px finger drag set Settlement threshold from 0.42 down to 0.2; Types' 23
  sliders were 40px in both builds. On a phone each label now sits over its slider: at 360 the narrowest slider is
  170px (Global) and 238px (Types). The ↺ keeps its reserved 44px while hidden — a labelled judgement: a slider that
  widened when its ↺ appeared would move under the finger on the first drag, which `display:none` until the value
  differs (one of the verifiers' two suggestions) would cause.
- **Windows open as full-height sheets on a phone (§2.1).** Settings already did (v2.28); Credits now does; the City
  Viewer, Route Editor and Settlements windows were already full-screen. Every window's × is a 44px target on a touch
  shell, and **stays in reach while the window scrolls** (fix round 2): Credits' and Settlements' × and title are a
  head row that sticks to the top of the window's scroller, as Settings' head already did. Their × used to scroll
  away with the content — after one swipe Credits' was drawn 0px tall and the Settlements × sat at y −553 at 360
  (v2.72 the same) — and a phone has no Esc. A floated sticky × alone was tried first and dropped: it lay over
  whatever scrolled under it, the Settlements tab bar's Types tab included. On the desktop the two heads are plain
  wrappers, and both windows capture pixel-identical to the fix pass's at 1366×768.
- **No page zoom on a touch shell** (fix round 2; v2.72 the same): a two-finger pinch on the phone's sheet or app
  bar zoomed the whole page to 5×, and the sheet is where a second finger lands while the first pinches the map at
  half. `touch-action: pan-x pan-y` on the touch shells keeps every scroller scrolling and takes the browser's pinch
  and double-tap zoom away; the map, the sliders, the joystick, the sheet's head, the tabs and the drawer keep their
  own narrower values. Android's *Force enable zoom* may override it (a device check).
- **A focused field stays in view** when the layout viewport shrinks under it on a phone (fix round 2): the sheet
  body scrolls the field back into its visible part once the sheet's own height transition has run. Measured
  headless by shrinking the layout viewport, as a resizes-content keyboard does: Seed was drawn 0px tall when
  412×915 went to 380, behind the bottom bar. Chrome on Android resizes the visual viewport by default, so a device
  may never do this — a device check.
- **Find on a phone** opens the page that holds the control (the domain page, STORY, MENU's drill-in path or the
  context page) and scrolls it into view; a chosen result closes the search row. Sculpt's Brush size, Hardness and
  Intensity, at the top of WORLD › Sculpt, are revealed by arming Sculpt — the World rail, then the Sculpt category,
  pressed by `#phSculptOpts`' own Find owner (v2.72's rule: Find must own every container a control can sit in) —
  with the domain page opened over them.
- **Assets mode on a phone.** The library is one column — its categories a row of chips that scrolls sideways, the
  grid, the inspector beneath — and while it is open **the Assets toggle itself** (`#assetsHeaderBtn`, relabelled
  "← Map" as v2.46 made it) **moves from the quick drawer to the app bar's start**, by a state-aware slot (`pa` in
  `_SHELL_SLOTS`, re-placed by `_carEnterAssetsMode`/`_carExitAssetsMode`): its drawer home opens from a grip on the
  map, which the library hides. Fix round 2: the fix pass drew a separate "← Map" frame button there
  (`#phAssetsBack`) and justified it with v2.46's rule — but that is the stand-in v2.46 retired (v2.24's
  `#assetsBackBtn`, "v1.57's one-control-two-surfaces defect"); it is gone, and the one toggle is again the visible
  way in and out. **No tab opens an empty page from the library** (fix round 2): MAP, the domain tab and STORY leave
  the library first, by pressing the same toggle, because their pages are the map's controls, the left dock and the
  timeline — all of which the library hides (the domain tab drew an empty sheet; MAP offered a hidden map's view
  controls); MENU works inside the library and stays. This is v0.65's rule on every shell, that switching domain
  leaves the library.
- **Leaving the library by a domain switch restores what entering it hid** (fix round 2, on every shell; v2.72 the
  same): `_applyDomainUI`'s exit undid only part of `_carEnterAssetsMode`, so after Window ▸ Workspace ▸ Civil from
  the library the toggle still read "← Map" and the Base map chip (`#layersFab`) stayed hidden — measured at
  1366×768 in v2.72 and in v2.73 before this fix. It now goes through `_carExitAssetsMode`, the one way out.
- **A settlement's popup on a phone sits under the phone's frame** (z 31: above the map's own controls, below the
  strip, the sheet, the bar, the scrim, the drawer, the fan and the search row) and stays open beneath a page until
  the sheet goes down. The desktop and the tablet keep v0.90's z 61.
- **The phone's landscape is decided with its class**: the decider writes `html[data-orient]` (land | port) under the
  same keyboard guard, and the landscape CSS keys on it, never on `@media (orientation)` — so a keyboard that leaves
  a typing portrait phone shorter than it is wide no longer turns its bottom bar into the landscape rail.
- **Seen and fixed on the way:** Select region (Region export) is a drawing tool, and arming it left the tool-options
  bar naming the domain — it now names Region select and its hint, on every shell. The status bar's two chips told
  a phone to use "the cog's Settings": there is no cog on any shell now, and Preferences reaches every page.
- **The fix pass (2026-11-02): two independent verifiers' defects, each reproduced, fixed and given a probe section
  that fails on the first cut.** Blocker: on the tablet the floating Context panel's new pin sat in the chevron's
  grid cell and covered it completely, so a tap meant to close the panel pinned it (the pin has its own column).
  Majors: the strip drew Sculpt's picker and brush under every tool; Freehand left DS-13 36px under the strip;
  a settlement's popup (z 61) covered the sheet, so MENU's rows took the popup's taps at 6–8 points; the quick
  drawer could not be closed by any drag (its grip hid while it was open, and the browser cancelled the drawer's
  swipe after two moves); on a 360–390px phone the asset library widened the page to 401px, pushed the bottom bar
  below the screen and left no visible way out; left-handed in landscape the grip sat under the side sheet; with ✋
  on, one tap still placed an icon with the density brush; and the prose items below. Minors fixed: the tablet's
  DS-13 spacing (halves 6px apart, gaps 16 — stage 1's column gap out-specified the row), the tablet hint dropping
  its keys, the narrow desktop bar's cascades off the right edge, a long-press card surviving a rotation, the
  keyboard guard keeping the class but not the layout, Find unable to reveal the strip's brush settings, ✋'s
  `aria-pressed` staying false, MENU › File › Export's path labels in three columns on a phone, MAP's two VIEW
  captions, the popup's 🎲 cut to 22px on every shell (since before v2.72), the desktop tool-options bar wrapping
  onto a second row at 860–940px (v2.72 the same; the hint now takes a zero basis at every width), every gesture
  figure without a source (each now names one or says it is a judgement, Ruling BK), "pending" for what Ruling EC
  had answered, and stale comments (the tool rail band, `_toolRailSync`, the version chip, the File block, the menu
  bar's header, the old drawer's handler, an end-of-line comment, the strip observer's reason it cannot loop) and the
  grip's tooltip, which told only how to open the drawer it now also closes. **Not changed, with the reason:** the
  1000px rule meant to tighten the desktop's menu buttons has never applied (the base rule after it has the same
  specificity) — left as v2.72 shipped it and annotated, since making it live would change stage 1's desktop at
  861–1000px and the full bar fits there; the long-press card's and MAP's departures from the design are stated
  in *Not in this build* rather than built (the card is one implementation shared with the right-click menu).
- **Fix round 2 (2026-11-02): two more independent verifiers' 23 findings on the fix pass's final bytes.** Each
  was reproduced before it was changed, and each fix has a probe section (or a stated reason) that fails on those
  bytes. Majors, all fixed and described in their bullets above: Sculpt's phone strip carried every Sculpt setting;
  the Settlements window's sliders shrank to 12px at 360 (a regression of the first cut); the domain fan's
  dismissing tap reached the map under it; ✋ went stale when the icon gallery armed or disarmed placement with the
  density brush on (`_carIconGalleryPick` and the category select now end in `_dskToolOptsSync`, so CLAUDE's "every
  way to arm a tool ends there" is true again); a phone in Assets mode opened an empty domain page and offered MAP's
  controls for a hidden map; the fix pass's "← Map" was the retired stand-in; the 44px floor missed every checkbox
  label but `label.chk`; and a menu row that cannot run did not say why. Minors fixed: the joystick's missing hide
  rule and §3's thin-out; Sculpt's Commit and Discard in the tablet's bar; page pinch-zoom on the touch shells; two
  phone hints that named a dock and an Esc a phone does not have ("the dock's Tools" now names the domain's page,
  and STORY › Journeys says tap ✓ Commit route — the older hints that say "click" for a tap are pre-existing and left);
  windows whose × scrolled away; a focused field left behind the bar by a resizes-content keyboard; DS-13's three
  glyph sizes and its linear double-tap animation; the docs' "§2.1–§2.5" (§2.3 and two more design items were
  neither built nor listed); MENU's drops presented as ruled when they are departures; the strip's missing *Not in
  this build* row for the other tools' settings; 22 new functions without a comment of their own (the file's
  67 new functions now all have one, counted by a script over v2.72 and v2.73), the stale "cog's Settings
  window" heading and `_phCtxAuto('info')`'s ignored argument; and `probe_desktopshell`'s NEW SEED assertion, which
  failed once under load and could not say why (its `settle` now also waits for the busy overlay to drop, and the
  assertion prints its three parts). Stated rather than changed: the app bar's 56px and the docks' widths, as
  departures from §3's figures (*Not in this build*). Found on the way and fixed: the quick drawer's pin-close read
  the DOM after the click, which the Assets pin's own click now changes (the drawer and its scrim stayed open over
  the library), so it reads the event's path; and leaving the library by a domain switch (above). Noted, not a
  shell fault: the two 844×390 captures (`v273_phone844x390_landscape`, `…_left_hand_sheet`) carry a 1px darker
  row across the map at the side sheet's head line (y≈133 — (72,96,90) against (61,119,129) above it). It is a
  headless-capture compositor artifact: `display:none` on `#phSheet` removes it, removing the head's border or
  box-shadow does not, a clipped capture of the same region shows no seam, and the map's own pixels there are
  unchanged with the sheet closed. Re-captured on the final bytes it persists unchanged, so it is left for a
  device check (listed under *What still needs a person* in the README), not chased in the CSS.
- **Unchanged by construction:** no engine line moved, so the headless suites and the render hashes are v2.72's.
  Stage 1's desktop at 1366 and wider keeps its layout (the docks stay docked, the row stays a column); four
  fixes reach it on purpose. Two draw the same where there was room — the tool hint's zero flex basis, and the
  popup's name field, which may now shrink so the 🎲 beside it is whole. Two are behaviour: an open right-click
  menu closes when the window's width changes, as the long-press card does, since both are one element placed in
  window coordinates (`_civCtxHide` from the resize handler) — measured with a mouse, a right-click menu open at
  1366 × 768 and the window narrowed to 1280: v2.72 leaves it open, v2.73 closes it; a height-only change keeps it
  open — and (fix round 2) leaving the asset library by a domain switch now relabels the toggle and brings the Base
  map chip back, as the toggle's own way out always did. Everything else fix round 2 changed is keyed to the touch
  shells: the desktop's menus draw no reason line, its Credits and Settlements windows capture pixel-identical to
  the fix pass's at 1366×768, and so do its map, WORLD › Sculpt with Sculpt armed, and the File and Edit menus open,
  at 1366×768 and 1600×900 — the one differing patch is Sculpt's stamp-seed field, a random number per session.

**`tests/perf/probe_desktopshell.js` (v2.72's probe) on v2.73**, with v2.71 as the comparison file: **154 passed, 4 failed** — the two version checks (it expects v2.72; this is v2.73) and the two phone sections that pin v2.71's shell (the ☰ drawer, the document bar) and now time out, all by design — against
v2.72's 174/0 (fix round 2: run twice, alone on the machine; the 2026-11-02 verifiers' two runs under load gave
153/5 both times, the fifth being *"NEW SEED is clickable in the dock and generates"*, which passed in isolation
every time — its `settle` now also waits for the busy overlay to drop, and the assertion prints its three parts,
so a red run classifies itself; re-run before believing it). Every difference, classified:
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
| `#settlementsHeaderBtn`, `#assetsHeaderBtn`, `#pinSave`, `#pinPlanner` | header (the two toggles); pins not drawn | the quick drawer; the Assets toggle moves to the app bar's start while the library is open |
| `#docReadout`, `#phaseChip` | not drawn; header | the world pill |
| `#undoBtn`, `#redoBtn` | header | app bar |
| `#findWrap` | header | the search row (`#phSearch`) |
| `#fileWrap`/`#fileMenu`, the other seven `.mb-panel`s (`#mbEdit`, `#mbAssets`, `#mbRecords`, `#mbStory`, `#mbPrefs`, `#mbWindow`, `#mbHelp`) | File ▾ in the header; menus not drawn | MENU, row for row but for Window's Status bar and Domain rail (not drawn: the phone has neither band) |
| `#seedN`/`#seedLab`, `#reseedBtn`, `#centerBtn`, `#genBtn` | document bar | WORLD › Generate › Run (now their markup home on every shell) |
| `#modeSeg` (View) | document bar | the Base map chip's popover — the MAP page (now its markup home) |
| `#layersFab` (popover open), `#viewDimSeg`, `#legend` | map overlays | the MAP page |
| `#scaleBarChkRow` (the Scale bar switch, newly id'd) | CARTO › Map style | the MAP page's view rows (desktop and tablet: CARTO › Map style, as before) |
| `#toolRail` band; `#sculptFeatureSeg`, `#sculptFeatHint`, `#sculptModeSeg`, the brush rows (`#sBrushRow`, `#sBrushKm`, `#sHardRow`, `#sIntenRow`) | band under the document bar; WORLD › Sculpt | band drawn on no shell; the picker in the tool strip, the rest at the top of WORLD › Sculpt (`#phSculptOpts`) |
| `#toolOptsCtx`, `#toolOptsHint` | not drawn | the tool strip |
| `#sculptCommitBtn`, `#sculptDiscardBtn`, `#civCommitWayBtn`, `#civCommitRouteBtn` | in the dock | the tool strip, the armed tool's only (the tablet: Sculpt's two at the tool-options bar's end) |
| `#undoBtn`'s hold (the design's undo history, with Redo inside it) | — | not built; `#redoBtn` is its own app-bar button instead (a departure: no undo-history list exists to hold it) |
| `#explTimelineSection`, `#explJourneysSec` (newly id'd), `#civPlannerSec` | Explore | STORY › Time and Journeys |
| `#statusBar` (with `#verTag`) | bottom band | MENU › Status |
| `#zoomOverlay` (`#zoomIn`, `#zoomOut`, `#panBtn`, `#zoomReset`) | bottom-right column, touch only | DS-13's row (also the tablet's) |
| `#sculptNavpad` (v1.19 joystick) | bottom right | above the DS-13 row, mirrors with it; hides when its stack does not fit the map band |

**New elements** (the frame only; each opens an existing control or runs a `_DSK_CMDS` command): `#phWorldPill`,
`#phSearchBtn`/`#phSearch`/`#phSearchClose`, `#phBar` and its four tabs, `#phSheet` and its five pages, `#phMenuRoot`,
`#phStrip` (its Sculpt group `#phStripTool`), `#phSculptOpts` (the top of WORLD › Sculpt on a phone), `#phGrip`,
`#phDrawer`, `#phFan` (over v2.71's own `#scrim`, reused); `#mbMoreWrap` (☰) and `#mbWorldWrap` (World) for the
narrow bar; `#dockLeftPin` and `#ctxPinBtn` for the docking rule; the `.cfm-head` and `.cr-head` wrappers (the
Settlements and Credits windows' sticky heads); empty slots `#slotPhAppAssets` (the Assets toggle's place in the app
bar) and `#slotToolAct` (the tablet bar's Commit and Discard); Preferences › Touch › Handedness. The fix pass's
`#phAssetsBack` is gone (fix round 2). The slot table is now **47 rows into 40 slots** (counted from the file's
`_SHELL_SLOTS` by a script, 2026-11-02).

**Not in this build** — each checked by grep, by concept

| Design item | Why it is left out |
|---|---|
| STORY › Conflicts, Notes; MENU's Records › notes rows | No conflict entity and no note editor in this file (above) |
| Two-finger tap undo, three-finger tap redo (Ruling DU Q5) | Not in stage 2's brief; no multi-finger tap handler exists here. The touch layer leaves two-finger touches to the existing pinch |
| Map rotation, the compass (and MAP's North up) | The 2D map never rotates (`viewT` holds a scale and a pan only), so it is always north-up. The 3D view's orbit camera has an azimuth (`_cam3d.az`), which ⌖ resets to its default view, not to north; no compass exists (grep `compass`: prose only) |
| The edge swipe for the inspector, haptics, *Pen draws, finger pans*, UI scale, Touch & gestures settings | None exist; the browser exposes no haptics to a file:// page |
| The cartwheel and its ring dock (tablet, §2.4) | The HTML has no cartwheel |
| World › Reset generation parameters, Reset one stage | Neither exists (CHANGELOG v2.72) |
| System back closing the top layer (§4) | A browser's Back leaves the page; this file does not intercept history. Esc closes, in order, the fan, the drawer, the search row and the sheet |
| Holding Undo for its history, with Redo inside it (§2.1, Ruling DU Q6); the tool chip's fan | No undo-history list exists to hold them (`undoStack` has no view; v2.72) and no tool fan exists. **A departure, stated:** the app bar draws Redo as its own button (the same `#redoBtn`), so the one redo command stays one tap away |
| The long-press card's sample pin, its per-domain actions (WORLD: Sculpt here, Cross-section from here, Paint with this biome, Centre here, Measure from here; CIVIL and CARTO alike) and its ground readouts (§2.1; Live.dc.html's SAMPLE sheet) | The card is the right-click menu's own item list (`_civCtxItems`): one implementation for a hold and a right-click. The design's rows would either join the desktop's right-click menu too (a change to stage 1 nobody asked for) or split the list in two. No sample-pin entity, cross-section tool or Measure tool exists (grep `samplePin`, `crossSection`, `data-civtool="measure"`: 0 matches each); Ruling EC item 5 ports Measure in a later version. The reading the card's Info here takes goes to the context page, the Info tool's own panel |
| MAP's Fit world, grid, the quick visibility switches (settlements, ways, landmarks, rivers, labels, political, hillshade) and the six Measure modes (§2.1) | Fit world is DS-13's ⌖, on the map, one tap. There is no graticule (grep `graticule`: 0) and no Measure tool (EC item 5). The switches that exist live in their own panels — rivers in WORLD › Generate › Ecology, hillshade in CARTO › Map view, territory, settlements and roads in Explore's filter chip on the map — and region labels and landmarks have none; moving them here would empty those panels on a phone. The Scale bar IS on MAP (moved, above) |
| MENU › Window › Status bar and Domain rail | **A departure the owner has not ruled on** (fix round 2; the first cut called it covered by Ruling DT Q2 and DX Q13): DX Q13 says the phone's MENU repeats the desktop menus row for row, its one ruled exception is PQ10's two rows, and the mockup's phone Window menu draws both. Left out because this file leaves out a row with nothing behind it, and a phone draws neither band — its status is MENU › Status and its rail is the bottom bar, which never hides. The two rows are `.dk-only` (a code comment at `winStatus`/`winRail` says why). For the owner: draw them (as inert checks), or rule the drop |
| MENU › Window › Context panel and Left dock as checks (Ruling DX PQ13) | **A departure the owner has not ruled on** (fix round 2): PQ13 keeps the Context panel row a check on the phone that "shows or hides the selection sheet". This build draws both rows as commands (they open the context page and the domain page), because the one sheet that shows MENU cannot show either page at the same time, so a check read from MENU would always read off. For the owner: keep them commands, or say how the check should read |
| §2.3 — New World, kept as shipped, plus AUTO-POPULATE (Ruling DT Q4) | §2.3 describes the native shell's New World dialog (`new_world_dialog.gd`, NWP-3) and four auto-populate switches after the terrain. This file's start card (the setup gate, *New world — base settings*) is its own and unchanged since v2.72, and no auto-populate switch exists on it (grep *Auto-populate the world*: 2 matches, both alerts sending the user to CIVIL's *Auto-populate world* button). Not built: a later version |
| §2.1 — Session mode hides WORLD from the fan and the swipe | No session mode exists in this file (grep *session mode*, *sessionMode*: 0) |
| §2.4 — the Assistant chip, first in the drawer and in MENU's Quick | No assistant exists in this file (grep *assistant*: 1 match, a comment saying there is none). §2.4's other phone rule, "no grip until something is pinned", never applies: the drawer always holds its four pins |
| The strip's other tools' settings as chips (§2.1, "its main options as chips"; Ruling DZ item 6: the strip "holds that tool's settings") — the paint brush, Territory's radius and faction, Way's type and snap, POI's type | Not built (fix round 2 states it; v2.72 listed it for the desktop bar): those settings stay on the domain page, one tap away on the domain tab, at half. Each tool's strip hint names the page they are on ("the Civil page's Tools"); only Sculpt's picker is in the strip |
| The app bar at 52 under a 28 status inset (§3) | **A departure** (a labelled judgement): 56px plus the safe-area inset — Material's phone top app bar, which keeps 6px around the 44px targets where 52 would keep 4 |
| The docks at §3's tablet widths (left 340/360, right 312/340; 400/380 from 1920) | **A departure**: the tablet is the desktop shell at touch size and keeps stage 1's 372/304 (400/328 from 1440), so one dock geometry serves both and the docking rule's sums are the desktop's |
| A swipe up on a tab opens its sheet full (§4) | Not built: a tab's tap opens its page at half, and the handle's tap or a drag on the head takes it to full |
| A submenu of only radio rows drawn inline as chips (§2.1) | Theme, Units, Handedness and Workspace drill in like every submenu: MENU is the desktop's panels themselves (one model, two renderings — Ruling DT Q2), and chips would be a third layout of the same rows |
| Tapping the strip's tool chip opens the tool's full options at the top of the domain sheet (§2.1) | The strip draws no tool chip (Sculpt's chips are its features). The domain tab's tap opens the domain page at half, and for Sculpt its full options ARE at the top of that page now (fix round 2) |
| The latch's toast (§2.5) | No map toast exists in this file (grep: the only toast element is the asset library's `#alToast`) |
| The landscape sheet "or wide (up to 600)", and from 1920px wider docks and the bar's readouts (§3) | Not built: the landscape sheet is 300–400px (42%), and stage 1's desktop is unchanged above 1440 |
| §3's second thin-out figure, "250 dp of free map" | The mockup computes it (`more: free >= 250`, Shell.dc.html) and draws nothing with it; only its 120 is built — the row, the scale bar and the grip hide below it (fix round 2) |
| The tablet's medium-width rail at 64px (§3) | **A departure:** the rail is 48px, the desktop's 40px raised to the touch floor |
| Search "runs any menu row" (§2.1) | Find lists controls and map entities; Ruling EC item 5 ports "Find runs commands" in a later version |
| The mockup's MENU look — Quick as chips, rows with an icon, a subtitle and a count, the drilled level's back arrow in the sheet head — the domain tab's own icon (▲ here), and rounded corners | **Departures:** the rows are the desktop panels' own rows (one model), the tab's glyph is the rail button's, and corners stay square until Ruling EC item 7 (rounded, as the design) lands in a later version. 844×390, 768×1024 and 1194×834 follow the mockup's structure closely (the 2026-11-02 verifier's side-by-side) |

**Ruling EC (owner, 2026-11-01) answered what this entry's first cut called pending** — DS13-Q1 (✋ only with a
drawing tool: "Only with drawing tools"), DS13-Q2 (one finger pans with a tap-only tool: "Yes, drag pans"),
Krita's guard ("Yes, ignore it") and thumb zoom with a drawing or placing tool ("Off unless ✋ is on") — at exactly
the defaults built here; nothing changed when it landed. Each lives in one function (`_ds13DrawKey`, the `tap`
branch of `_tchDown`, `_tchMultiStart`, `_tchThumbOn`).

**What needs a person on a device:** real touch (the gesture timings — hold 480/500ms, the 240ms tap wait, the 120px
doubling, every labelled judgement among the phone frame's gesture figures — on glass, MS-10); a sideways swipe on
the open quick drawer (headless, `touch-action: pan-y` lets it through; a real browser's gesture arbitration is the
claim); `env(safe-area-inset-*)` under a notch and a gesture bar (headless insets read 0); an on-screen keyboard on a
phone and on a landscape tablet (the keyboard guard now keeps the layout as well as the class — proved headless by
shrinking the layout viewport, which is what a resizes-content keyboard does); the asset library on a real 360dp
phone (a mobile browser may zoom out rather than widen the layout viewport, as the emulation did); Android's own
long-press menu and the click after a fling (both handled, both only provable on a device); rotation of a real
phone; WebGL in 3D; a screen reader over the bar, the sheet and the fan.
