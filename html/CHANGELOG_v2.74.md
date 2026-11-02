## v2.74 (DCC line) — the fix pass and the design alignment: the Sun slider's cache key, the Settlements rows, Credits under Settings, AA contrast in both themes, the browser's own controls, the clippings, no render behind the start card, the shell's resize cost, the planner's toll fix, rounded controls on square surfaces, one washed on-state, and one type for the dock's controls, its labels and the windows' headings

Owner: *"can you check and improve the visuals and performance of the html? Maybe even upgrade some features based of
the godot version"*, then, on the order: *"Fixes and feature ports first then stage 3."* (Ruling EF, Cartalith_GDT
`LARGE_ITEM_RULINGS.md`). This version is the first of those: the fix pass of the 2026-11-01 audits of v2.72
(`html/audits/v2.72-2026-11-01/PLAN.md` §5 in Cartalith_GDT, each finding re-opened at its symbol in this file
first — v2.73 had moved several) and the plan's design alignment (§8: V24's washed on-state and the dock's type,
V25, V26, V29, V30), with Ruling EC item 6, the planner's toll fix (*"Apply Godot's fix"*), and item 7, the
corners (*"Rounded, as the design"*). **The owner questions, by number:** Q1 (the order) is Ruling EF's; Q2 (the
toll fix) and Q4 (the corners) are Ruling EC items 6 and 7, built; every other question takes the default the plan
states for it, as Ruling EF lists — **Q3** the light accent `#8b550d`; **Q5** the washed on-state, a fill kept only
on tool-mode segments; **Q6** the place popup's dark palette unchanged (its light theme fixed); **Q7** no icon swap;
**Q8** no font embedded and the status-bar chips kept; **Q9** the darker box behind the scale bar and legend;
**Q10(a)** a flat backdrop behind the start card and **Q10(b)** a busy overlay that takes every input on the map
(both shown to the owner in the captures); **Q11** the view still resets on a domain switch (P8 is not built);
Q10(c)–(e), Q12–Q14 belong to later versions. `tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs
v2.73 **ALL IDENTICAL**. Verification is `tests/perf/probe_fixpass.js` — ⟦FIXPASS⟧ — and the contrast gate
`tests/perf/census_v274.js`: the visual audit's own WCAG census over its seven views (a place selected on the map,
the Settlements screen, the Asset Library, the route editor, the City viewer, Settings' five pages, Credits), both
themes, 1600×900 and 1366×768 — **0 enabled text runs below AA of 2 414** (v2.73: **561** of 2 494).

- **The Sun slider did nothing, and two more controls served a stale image (P7).** The render cache's key,
  `_civBakeKey`, read `state.sun`, but the slider writes `state.sunAz` — a field the state literal has never had,
  so the key never moved and the cached map came back. Checking EVERY key part against the state literal (the
  audit's instruction, and MISTAKES' "write a staleness/cache key") found a second: `state.shadeOnHypso` is the
  CHECKBOX's id, the state field is `state.shadeHypso`, so "Hillshade the tinted map" never re-shaded Relief view.
  And a sweep that changes every render-calling control once (the 47 the file binds to `render()`, in Biome and in
  Relief view) and compares the cached frame with a cache-cleared render found a third input the key never had:
  "Show rivers in biome view" (`state.showRivers`; the LOD tile key `_lodRenderKey` has always carried it). On
  v2.73 the sweep finds four stale pairs (Sun in both views, Show rivers, Hillshade); on v2.74 none of the 94
  control changes. A real click on the Sun slider's track moved `sunAz` to 80° and left the map unchanged on v2.73;
  it redraws now. A miss renders exactly as before, so no output moved — only which renders are reused. Once fixed,
  dragging the Sun slider costs a full render per input, as the other 36 look sliders do, until the speed pass
  (v2.75, Ruling EC item 8's preview).
- **CIVIL ▸ Settlements' rows ran together into one string (V01)** — since v1.16. `_stEnsurePool` writes
  `display:flex` into each pooled row's cssText and `_stUpdateVisible` deleted it with `row.style.display=''`, so
  every visible row computed `block` (0 of 21 flex on v2.73, all of them now). Restoring the rows showed the
  table's head and its columns 1–10px apart: a cell without `overflow:hidden` kept its content's min-width, which
  differs between the head's labels and a row's numbers, and the head is 2px wider than the bordered list under
  it. Every cell may now shrink to its share (and ellipsises), and the head takes the list's 1px side borders, so
  both lay out the same width — the columns line up within 1px.
- **Credits opened BEHIND Settings (V10)** — z-index 50 against Settings' 74, and on a phone the cog's About page
  is the only route to Credits. Now 76 (under the map menu's 80 and the gate's 90): after Settings ▸ About ▸ ⓘ the
  window's centre is Credits' at 1600×900 in both themes and at 412×915, where a real tap on its × closes it (it
  could not be reached before).
- **The armed pins (V04).** At rest, "← Map" — the visible way out of the Asset Library (v2.46) — was accent text
  on the theme layer's accent fill, a blank amber block. Both armed pins (Assets, Settlements) are now the theme
  layer's rule 3, a wash plus an edge (the Settlements pin drew the saturated fill with `--accInk`: readable, but
  the fill rule 3 forbids). The label measures ≥ 4.5:1 at rest in both themes.
- **The three windows' × in the light theme (V05)** — the City viewer's, the route editor's and the Settlements
  screen's — were dark ink on a hard-coded dark chip: 1.07–1.08:1. They take the panel tokens (14.72 dark, 16.67
  light; on hover the accent edge and ink, as Credits' does).
- **The Settlements screen and the route editor drew straight on a 90% scrim (V06).** The map's labels showed
  through the Settlements screen's slider rows, and in light 22 of its text runs fell below AA ('World overview'
  2.4:1). Its column is a panel with side rules now, and the route editor (full-screen and exclusive) draws on
  `--bg`.
- **On a phone the Settlements screen drew every faction name 0px wide (V23)** — needing 42–99px, behind two pills
  and the stats that do not shrink. The row wraps and the name keeps 120px: at 412×915 every name is drawn at least
  as wide as its text. And before any settlement existed the overview read *"Largest by population: Aurelia (0)"*,
  an argmax over zeros. Only a faction with people can be the largest now; otherwise the name is dashed with its
  reason — *no settlements yet*, or *no faction holds a populated settlement* when the only settlements are
  Unclaimed — both branches exercised by the probe.
- **The loading overlay dimmed the map while zoom, the 2D/3D square and the Base map chip stayed bright and
  pressable above it (V11; owner default Q10(b)).** `.busy` had no z-index. At 30 it lies over everything drawn on
  the map — zoom, 3D and Base map (3 of 3 now, 0 of 3 on v2.73), DS-13, the phone's grip, a floating dock — and
  under every menu panel (45/46), the place popup (61; 31 on a phone), the phone's strip, sheet, bar and drawer and
  every window. It covers the map only: refusing the docks' own input mid-generate is P1(C), v2.75's.
- **`#assetsPanel` has been unreachable in Assets mode since v2.24** (filed in Cartalith_GDT's `OUTSTANDING_WORK.md`
  by stage 1): the library's own side panel since v0.65 — the pack's name, author, licence and stats; Export pack,
  Validate, Apply, Import pack, Clear — lives in the left dock, and v2.24's `body.assets-mode .dock{display:none}`
  hid every dock. Fixed at that rule: on the desktop and the tablet the LEFT dock stays in Assets mode showing that
  panel alone (and its head, which now reads *Assets › Pack*), while everything v2.24 meant to hide stays hidden —
  the Context panel, the rail, the timeline strip, and the dock's tools and domain bodies. The docking rule is asked
  of the library: the dock floats over it (closed, a 44px strip whose chevron opens it) when docking would leave the
  library less than its rail, its inspector and two tile columns (`AL_STAGE_MIN`, 824px, read from those rules) —
  docked at 1366, floating at the tablet's 1194. Validate runs on a real click (and tap) and its report appears. The
  tool-options bar's hint in this mode, *"The library has the stage"*, now adds *"its pack panel on the left"* there.
  Find had listed the panel's labelled fields and could reveal none (no owner since v2.24's rail replaced the
  Assets tab); it has an owner now, which presses the Assets toggle only when the library is not already open.
  **Not on a phone** — its sheet's domain page leaves the library first (v2.73's rule that no page opens empty), so
  there the pack's metadata and Validate are still not reachable; MENU › Assets reaches Export, Open pack, Apply
  and Clear.
- **The place popup ran 33px onto the expanded timeline strip at 1366×768 (V16)**, its `min(80vh,660px)` cap
  (614px) taller than the 587px map it is clamped into. On the desktop and the tablet `_civOpenPlacePopup` caps it at
  the map's height less the clamp's margins; over four places the worst overlap is now ≤ 0 (33.4px on v2.73). The
  phone keeps its CSS cap.
- **Contrast in both themes (V08, V09, the census).** `--faint` failed AA in both themes (3.73–4.09:1 dark, 4.03–
  4.46 light). Re-based to `#7d8285` dark (4.54–4.97 on the four grounds) and `#676762` light (4.52–5.45); both
  still fail on a wash (dark 4.17/3.63, light wash2 4.17), so no text may be faint on a wash, and the step to `--dim`
  survives (4.54 against 5.62 on `--panel2`). The light `--accent` `#a4650f` (3.75–4.22) is `#8b550d` (owner default
  Q3): 4.52–5.91 on all six grounds, and `--accInk` on it 5.91. The light washes deliberately keep the old hue — a
  wash on `#8b550d` would drop the accent on wash2 to 4.41. Two literal inks on the accent fill (`.pill.on` 3.73,
  the Layers row 3.92 on the old light accent; 2.85 and 3.00 on the new) became `--accInk` — and the design
  alignment below then washed both on-states, so no ink sits on an accent fill there at all. The census then found two
  more the audit had not named, both fixed: the CIVIL dock's faction picker drew white names on every faction's own
  colour (Sythe Dominion 2.40, Draumr League 2.55, Veldmark 3.32 — the colours are data, so the ink is now whichever
  of white and `#141005` measures higher on each), and the route editor's "(land)" note, faded to .55 (2.59 dark,
  2.34 light), is `--dim`.
- **The browser's own controls follow the theme (V02).** There was no `color-scheme` anywhere: on the dark UI an
  unchecked box was a white square, a checked one Chrome blue, slider tracks and select lists stayed light.
  `:root` computes `dark` / `light` now, and checked boxes and radios paint the accent. Native select popups and
  number spinners cannot be captured headless — a person opens one of each per theme.
- **The light-theme place popup (V12, first half).** In light the popup stayed dark (v0.90's literal palette) while
  its pills, hints and fields took light-theme tokens: trait pills 3.51:1, '0–1' and '(auto)' 2.16. In light it
  takes the theme's colours now (0 popup text runs below AA). The dark palette is unchanged (owner default Q6).
  Its Name row already fits in both themes — v2.73's fix pass did that (`.row > input{min-width:0}`); the probe
  keeps it measured.
- **One 2px side for an active sub-tab and Settings page (V03).** The theme layer's `button.on{border:1px solid
  var(--accent)}` filled the three sides each `.on` rule did not recolour, so both drew a full accent box (1px on
  all four, since v2.23). The border shorthand is reset first now: the sub-tab is underlined, the Settings page
  barred on the left (underlined on a phone, where the list is a row of tabs) — 2px on one side, 0 on three.
- **The design's focus ring on every control (V19)** — 2px solid accent, 2px out (Desktop.dc.html's
  `.dk-chip:focus-visible`); controls reached by Tab drew Chrome's 1px auto ring. Menu rows and the rail keep their
  inset ring. Text fields draw it too, a labelled judgement (the design states a ring for its chips only).
- **Text fields (V20)** took the browser's Arial 13.33px with 1px of padding; they take the UI font with 5px 7px
  now, and numbers and selects stay mono.
- **The scale bar and legend over bright terrain (V21; owner default Q9, the darker box).** .85 and .86 boxes, the
  legend's text `--muted`: computed over white snow 8.82 and 6.01:1 (2.91 and 2.77 before), over sand 9.89 and 6.68.
  The light theme's boxes are unchanged. The halo trial is v2.77's.
- **Empty asset slots were 154px white squares on the dark library (V07)**, the user's preview colour (white by
  default) — 71 of 71 empty slots at 1600×900. An empty slot is an `--ins` well now; a slot with art keeps the
  preview colour. A browser without `:has()` keeps the old white.
- **File ▸ Export ▸ Export GeoJSON looked like a line of text (V13)**, stripped by v0.87's `.dropdown button`
  reset; it is a button again (1px border on `--panel2`), on every shell.
- **The status readout cut a chip mid-text (V14)** — at 1600 one chip of eight was cut. The readout wraps onto a
  second line that is never drawn, so a chip either fits or is not drawn; 0 chips cut at 1366, 1600, 1920 and 2560.
  The phone's MENU › Status wraps them in full, as before.
- **Arming Freehand hid the sculpt picker (V15).** `_dskToolOptsSync` centres the armed feature and Freehand is the
  last, so 9 of 13 features scrolled out of sight at 1600 and 11 at 1366, with nothing saying the row scrolls. The
  picker has row one to itself now (the mode strip leads row two, the spacer stops growing, the chips are 7px a side
  instead of 9): 0 hidden at 1600, 1 at 1366 (the bar stays 75px, the idle pins end where they did). Freehand's hint
  said "pick a sub-mode below"; it names the mode strip.
- **The New World card (V17):** six shape cells in one row clipped 'Archipelago' (and 'Pangaea' at 412); three
  columns now, none clipped at 1366×768 or 412×915. The sticky Back/Generate bar sat 24px above the card's edge with
  scrolled content showing through; it meets it.
- **CIVIL's five sub-tabs fit the dock at 1366 (V18)** — they needed 387px of 371, so the dock scrolled sideways.
  Tighter tracking and padding bring them to 359px, and each tab's basis is its own label (at an equal fifth,
  SETTLEMENTS' 86px of text was cut inside a 74px button). Measured with Inter; a person re-checks Segoe UI.
- **The map's chips share a bottom edge and the Context panel's empty hint is whole (V22).** The Explore filter
  was 36px beside the Base map chip's 32. The empty hint ("Inspect a settlement, label or icon on the map, or a row
  in a list", 397px in a 249–273px line) was cut at every size and repeated the body below it; it reads *Shows
  what you select*.
- **The category arrow (V25, the arrow only)** was glued to its label ('▸GEOLOGY') — a flex summary collapses the
  spaces in `content:'▸  '`; it keeps 8px. The label rule itself is below (V25).
- **Stale copy (V28).** The strip's empty note told a user looking at the open strip to open it (it says *press Add
  year below* while open now, and follows the chevron); WORLD's "World has no map tools — shape terrain in Sculpt"
  is hidden while Sculpt is open; Freehand's hint (above).
- **No render behind the start card (P4; owner default Q10(a)).** Every page load rendered the empty 2048×1311
  ocean behind the gate — 1846–1960ms (the audit's medians), inside one long task, so the gate's buttons were dead
  for about two seconds. The browser branch now sizes `#view` as that render would (so the layout and the cover zoom
  are the same) and fills it flat with the boot render's mean colour (rgb 73,102,118); "Skip for now" renders the
  empty world once, Generate, Load and Import render anyway, and any render clears the flag. 0 renders before the
  user leaves the gate (1 on v2.73); after Skip the canvas hashes identically to v2.73's (`c6bef66b`, with
  `Math.random` pinned so both boot the same seed). The load's longest task: see *Measured*. The headless branch
  (`tests/run.sh`) is untouched.
- **The shell's resize and boot layouts (V272-1, V272-2; v2.73's `_shellPlace` replaced v2.72's `_dskPlace`).**
  A timeline trace of one 1400↔800 crossing named the script that forced each layout: `_shellPlace` measured the
  bands right after moving the rows, and two window `resize` listeners (`_syncChromeTop`, v0.87's map fill) measured
  before `_dskInit`'s own listener re-placed the controls. The bands are published in the refit's frame now, first
  (reading the layout the fill reads anyway), the two listeners schedule that refit, the boot no longer measures
  before placing, and `_syncChromeTop` reads both bands before writing either and writes only a changed value.
  Forced layouts per crossing: 9 → 8 (desktop at 860, tablet at 1180), 11.5 → 9.5 (desktop ↔ phone at 600), and 8
  → 7 on a resize that crosses nothing; at boot 16 → 14, and `_syncChromeTop`'s boot time 28 [27..37] → 0 [0..1] ms
  (CDP counters, n=5 per version, interleaved).
- **The planner's toll fix (owner, Ruling EC item 6: "Apply Godot's fix").** `_jpClaimedAt` tested
  `civTerritory[…]>=0` on a `Uint8Array` whose 0 is Unclaimed, so every cell read as claimed — open sea included —
  and the two things built on it never fired: the claimed-land floor applied to every stage, and the frontier toll
  (`JP_COST_TOLL_PER_BORDER`, 6 day-wages) counted 0 borders on every journey. It is `> 0` now, as the native port's
  `cartalith-civ::jp_claimed_at`. **What it changes, measured on the feature audit's journeys** (seeds 12345 and
  777 at 256, after Auto-populate and Auto polity; v2.73 against v2.74, the same worlds — `hash_gen1` is ALL
  IDENTICAL): no sea cell reads as claimed (0 of 12 258 and 10 939; all of them before) and no unclaimed land cell
  (0 of 152 and 7 235); every faction's cell still does. At seed 12345, 12 of the 77 stages change tier, and at seed
  777 41 of 84 — every one a coastal, bay or river stage, which takes no claimed-land floor now (the territory raster
  covers land only) and falls from *Sparse Settlements* to *Ruined Region*; on these routes no land stage changed (the
  land they cross is claimed). All six of seed 777's journeys gain one frontier toll (6 day-wages each; 0 before),
  and every journey that changes tier takes longer: seed 777 21.3 → 24.5, 21.8 → 24.3, 25.4 → 27.5, 20.3 → 22.1,
  19.1 → 20.6 and 16.1 → 18.9 days; seed 12345 12.7 → 14.1, 14.3 → 14.9, 16.6 → 16.8, 17.6 → 17.8, 19.8 → 20.3,
  18.3 → 18.5. Unclaimed wilderness on land reads *Ruined Region* as in Godot. The toll counts a change between
  claimed and UNCLAIMED ground, not between two factions (the native port's `jp_journey_cost` counts the same way).
  The route editor says both now: *a stretch through a faction's land never reads below Sparse Settlements, while
  unclaimed land and water can read lower*, and *one levy each time the route passes between a faction's land and
  unclaimed land or water*. World generation does not read this function. No RC suite test pins the old output
  (`smoke_gen1.js`, `probe_travel.js`, `probe_landsurface.js` and `probe_rivernav.js` build fixed stages or
  monkey-patch `_jpDeriveStages`; grepped for `_jpClaimedAt`, `claimedFrac`, `tolls`, *Ruined Region*).
- **Rounded controls on square surfaces (V26; owner, Ruling EC item 7: *"Rounded, as the design"*).** v2.23's
  `*{border-radius:0 !important}` — with a comment calling the mockup flat — squared every corner in the file, inline
  styles included. The mockup is not flat: it rounds its controls and keeps its panels and bands square, and the
  native port's DccTheme had already moved off radius 0 (`btn_radius`). The reset is gone, and the radius lives in
  five tokens in the theme layer, **pointer / touch**: `--r-btn` **16 / 12** (every button that draws a box),
  `--r-tile` **14 / 12** (a selectable tile: the icon gallery, the asset cards and their variants), `--r-field`
  **12** (inputs, selects, textareas), `--r-menu` **10** (the menu bar's buttons, Undo/Redo, the strip's TIMELINE
  toggle) and `--r-pill` **999px** (chips, pins, pills, the map's round buttons, the search field, the Base map
  chip, the 2D/3D square). **Which source each follows, and why:** at a pointer, the design's own radii
  (design/mobile-shell-2026-10-31, `d_vals.js` `densVars` and `shell.tpl.html`: buttons `--r24` 16px on 32px, chips
  and pins `--r22` 15px on 30px — pills — fields `.clr-in` 12px, menu buttons and the timeline toggle 10px, tool
  tiles `--r20` 14px) — the newer canvas wins over the port's pointer 8, which DccTheme measured on the 2026-09-07
  canvas; at touch, **the port's touch radius, 12** (DccTheme `btn_radius` [8, 12]; inside the ruling's "about 10–16
  px"), not the design's touch pills (`--r24` 24px on 48px buttons): hit-testing follows `border-radius`, so a 44px
  pill loses its corners as targets, and v2.73's guarantee that a finger 4px in from a 44px control's corner still
  meets it — its probe presses there — failed at 22 (the floating dock's chevron at three tablet sizes, DS-13's −)
  and holds at 12. Fields are 12 at both densities in the design, the port and here. DS-13's round buttons keep the
  design's shape — ⌖ round, − | + one pill whose halves meet square at the hairline — and their whole 44px box as
  the target (an empty `::after`). Panels, docks, bands, windows' outer frames, menus, popovers, cards, wells,
  toasts, the scale bar, the legend and the map stay square (Ruling EC item 7: "Panels and bands stay square"); rows,
  tabs and the rail are squared by one rule, since a rounded full-width row curves its own divider. A segmented
  control is one rounded container that clips its square children, so its inner dividers stay straight (the
  design's zoom column and rocker are built the same way); `.seg.grid`'s six-column borders are re-checked by the
  probe (unchanged), and New World's three-column shape grid stops drawing a divider after its third cell (it doubled
  the container's edge inside the curve). The pull-out tab on a phone rounds the side facing the map (the design's
  `gripRadius`, 14px). The shell block's comment that called the mockup flat is corrected (Ruling BK).
- **Every radius the reset had silenced, decided where it is written: 101 declarations** — 66 in the stylesheet
  (68 `border-radius` in the `<style>` block, less the reset and its re-assertion of the joystick's circles) and 35
  inline (7 in the markup, 28 in the JS templates; counted with a scan of the file for `border-radius`, 2026-11-02).
  The audit counted 67 in v2.72's stylesheet. **Kept, 26:** the swatches, dots and keycaps (the legend's, the
  faction colours in four JS templates, the Strahler legend's, the slicer's transparency swatch, the Layers rows'
  key, the asset library's background swatches), the progress bar and its two pseudo-elements, the planner's
  stacked bar, the joystick's two circles, the thumbnails inside tiles (the pack grid's, the icon gallery's and its
  "none" glyph, the asset card's thumb), the version, phase and asset badges and tags (already pills), and the three
  explicit squares (`.seg button`, the sub-tabs, the asset rail's rows). **Changed to a token, 10:** `button` (6px),
  `input[type=number]` and `select` (5px), `.seg` (6px), `.pill` (999px, now the token), the Explore filter and the
  map's `.zbtn` (8px, now pills), the icon gallery's options, the asset cards and their variants (5–8px, now tiles).
  **Removed, 65:** 26 from controls whose class token now decides them — the windows' × buttons, the City viewer's
  edit button, the old `.menubtn` and `.dropdown button`, the slicer's zoom buttons, the panel toggle, the Layers
  rows (squared as rows) and the map's context-menu rows, and every field that wrote its own 4–10px (the place
  popup's, New World's numbers, the route editor's stops, the asset library's search, metadata and cell names, the
  sculpt seed, the pack's name, author and licence, a place's History, a faction's rename, the variant and slicer
  names); and 39 from surfaces, now square — the legend, the scale bar, the resolution overlay, the place popup and
  its city picture, the settlement info card, the City viewer's legend, the route editor's map, profile and columns,
  the Credits and start cards, New World's legend, the File dropdown, the Find results and their rows, the asset
  inspector's preview, the slicer's dialog, canvas, drop zone and name list, the toast, the Settlements table's
  viewport, the type preview's canvas, eight civ cards and boxes in the dock's lists, the planner's blocked box, stage
  cards, details and calculation trace, the map's context menu, and the phone's Credits card. `.find-hit` no longer
  sets a 3px corner on whatever Find reveals; its outline follows the control's own.
- **One on-state: the wash (V24 a; owner default Q5; Cartalith_GDT `LARGE_ITEM_RULINGS.md` 2026-09-06 item 7,
  "Resolved to the WASHED treatment").** 26 of the docks' segments — and every other segmented control, the
  Settlements window's tabs and the route editor's Carriage switch among them — drew v2.23's saturated accent fill,
  which the theme layer's own rule 3 forbids. An armed segment is now the design's chip — wash2, a 1px accent edge,
  accent ink — drawn 2px INSIDE the segment (`::before`, under the label): a ring on the segment's own box would be
  cut off along the container's rounded end, and a chip inside leaves the segment's width and every divider where
  they were. **The fill stays on tool-mode segments only** — the Tools palettes (`[data-civtool]`: Inspect,
  Settlement, POI, Territory, Way, Label, Icon, Info, Route) and Sculpt's feature picker — as the port keeps its
  tool bar's SCULPT / PAINT / MEASURE segments filled. An armed pill (a recorded year, a place's trait) and the Base
  map popover's picked row are washed too; no other armed control draws the accent fill (the probe walks the docks,
  the sculpt bar, the Settlements window and Settings). The ink on the chip measures ≥ 4.5:1 in both themes (dark
  accent on wash2 over `--panel2`; light `#8b550d` on wash2 over `#fbfaf7`). The Carriage switch gets 10px a side,
  room for its chip (it had 1px).
- **The dock's type (V24 a–c).** The dock's segments take the design's chip type — mono 10px, weight 500, .08em (its
  category chip: IBM Plex Mono 10px/500/0.8px) — with each segment's basis its own label on one line: at an equal
  share mono's wider labels broke inside their segments at 1366 ("⌂" over "Settlement"). Every dock action button
  (`.dock .btnrow > button`; 8 drew sans sentence case at 12px in the audit's census: Commit way, Clear all labels,
  Simulate…) is mono, weight 500, uppercase, .12em (.14em at 10.5px on a primary), as Run's already were. CIVIL's
  five sub-tabs are mono 10px, weight 500 — the armed one too. **Bake ALL levels** and **Un-finalize** run the dock's
  full width (Bake was 224px of 371 at 1366); the probe bakes for real to measure Un-finalize.
- **One section label (V25).** One rule over the nine selectors the audit listed — the dock's section heads and
  sublabels, the strip's RECORDED, the Layers popover's groups, the asset rail's heads, New World's labels, the City
  viewer's and the route editor's column heads: mono 9.5px, weight 500, .2em, uppercase, `--dim` (the design's
  "section label TOOLS": 9.5px/500/1.9px, `#8d9296`). The same role had been drawn about eight ways (9–10.5px,
  .06–.22em, weights 400–700, `--dim`, `--faint` or accent); the two that were faint move up to `--dim`. Location
  labels keep the accent (the dock's path, the tool bar's context, the Context panel's kind, a menu's caption).
  Long help that might move into `details.adv` is content, not CSS, and is not moved.
- **The timeline strip's toggles (V29; the 2026-09-06 ruling, item 7, that the timeline's pill on-state matches the
  WORLD dock's).** The recorded years are pills, washed while active. The three filters — Exist only, Ghost removed,
  Highlight new — were native checkboxes with sans labels; each LABEL is now the design's chip (a pill, a 1px edge,
  mono 10px, `--muted` ink), washed while checked. The checkbox stays inside it, visually hidden but focusable: Tab
  still reaches it, Space still toggles it, the focus ring is drawn on the chip, and the label is kept (a screen
  reader still announces a checkbox — a person checks that). The play button is the washed accent control, its
  label kept. A year pill's ✕ (remove the year) drew the label at .6 opacity, 2.39:1 on the old fill and 3.75 off
  it; it takes `--dim`, or the pill's ink when armed. The phone's STORY › Time keeps the long-label checkboxes.
- **One window heading (V30).** The five windows' headings drew six styles (Settlements sans 19px bold, the route
  editor sans 17px, Credits mono 14px, Settings mono 12px at .1em, the start card and New World mono 13px). One rule
  now: mono 11px, weight 500, .2em, uppercase, accent — the design's dialog title (`font:500 11px 'IBM Plex
  Mono';letter-spacing:.2em`, its titles in capitals). The audit's plan said 12px, Settings' size; the drawing says
  11 and wins. The City viewer's heading is a place's name and keeps its 16px sans (exempt). Records ▸ Factions…
  still opens the window titled "Settlements": renaming one of the two is the owner's call (the default is no change).
- **Two more contrast failures**, found by taking the census over views the audit's seven did not include (the strip
  expanded, the Settlements window's Types tab): the year pills' ✕ (above), and the Types tab's quarters line
  (`_umTypePreviewStats`), the hint's `--faint` at .75 opacity, 3.19:1 — now the hint's own ink, 4.79.
- **The truncation census and the rows that wrap.** The visual audit's census (`lib.js` `__aud.truncs`), taken by
  `probe_fixpass.js` against v2.73 over every dock with its sections open, CIVIL's five sub-tabs, the sculpt bar, a
  place's popup, the Settlements window's four tabs, Settings' five pages, Credits, File ▸ Export, the asset library,
  the strip and New World, at 1366×768 and 1600×900 on seeds 12345 and 777: **no truncation v2.73 did not have**
  (12 truncations at 1366 and 14 at 1600 on both seeds, against v2.73's 22 and 23). The Settlements table is left out of that comparison: the fix pass made each of its cells ellipsise so
  its head lines up with its rows (V01), and at 1366 two of its heads are cut ("Faction" needs 38px of 36, "Econ.
  role" 51). **The rows of buttons that wrap**, every one, on both seeds: Sculpt's eight presets now wrap as whole
  segments onto two lines at 1366 and 1600 (before, "Rolling Hills", "Volcanic Isle" and "Glacial Valley" broke
  inside their segments); CIVIL ▸ Generation's AUTO-POPULATE WORLD | CLEAR PLACES & ROUTES takes two lines at 1366
  (one at 1600); WORLD › Sculpt's Deselect | Hide/show | Move up | Move down | Delete took two lines before and
  still does; New World's six shapes are the fix pass's three-by-two grid. Section labels wrap too where the .2em
  tracking makes them longer than the dock ("STEP 1 · POPULATE SETTLEMENTS (blank = automatic)").

**Measured** on the final bytes (this 4-CPU machine, which another agent may share; timings are indicative and the
counters are the gate): `tests/run.sh` 1280/0, `run_um.sh` 882/0 and `hash_gen1.js` v2.73 → v2.74 ALL IDENTICAL (default, geoid, waves, ao, icons). `probe_fixpass.js`: ⟦FIXPASS_DETAIL⟧. `census_v274.js`: v2.74 0 of 2 414 enabled runs below AA (2 more per place view, disabled controls' labels, are exempt and counted apart; the fix pass's bytes counted 2 408), v2.73 561 of 2 494. The
load (P4), alone on the machine, a fresh browser per sample, n=6 each, interleaved (the fix pass's bytes): the
longest task 2 697 [2 325..2 971] ms on v2.73 against 332 [313..557] on v2.74, `DOMContentLoaded` 3 123 [2 748..3 509]
against 858 [773..1 096] — disjoint. The shell's own probes on the final bytes: ⟦SHELL⟧.

**Two flaky suite assertions, disclosed not fixed.** Of four `tests/run.sh` runs over this version's bytes, two passed
1280/0 (the final bytes' last run among them) and two failed ONE assertion each, a different one each time: *smoothed
bathymetry variance <= raw seabed variance (5.8e-3 <= 5.8e-3)* — v2.73's known flake — and *traced length sane vs
boundary-cell count (4721 vs 2208)*. Both build their world on an unpinned seed (`state.tect.seed` is
`Math.random()` at load, and `test_tail.js` generates on it). With `Math.random` pinned (seeds 101, 202 and 303), v2.73
and v2.74 print the same 1 280 assertion lines, timings aside, and pass 1280/0 each; `hash_gen1.js` is ALL IDENTICAL.
If either goes red, re-run before believing it, as for v2.25's SST assertion. ⟦FLAKE⟧

**Where this departs from the design** (design/mobile-shell-2026-10-31; MISTAKES' "it matches the design" — the
whole list, not the rows that agree):
- **Windows, menus, popovers, cards and callouts are square.** The design rounds its windows (12px at a pointer,
  18px at touch, a phone sheet's top corners 22px), its menus (a dropdown's lower corners 16px, column menus 10px,
  the tab and context menus 12px), its popovers (search 20px, edit history 16px, the module list 14px) and its
  cards, messages and alerts (12px). The owner's answer keeps panels and bands square (Ruling EC item 7), so every
  surface here is.
- **The phone's quick drawer is square**; the design rounds its outer side 18px (its pull-out tab here does round,
  14px, as drawn). **The domain fan's rows** are rows in a panel; the design floats each domain as an 18px card.
- **Touch controls are rounded rectangles (12px), not the design's touch pills** (24px on 48px buttons, 22px on
  44px chips): the port's touch radius, chosen so a finger at a 44px control's corner still meets it (above).
  Pills that stay pills at touch (`.pill`, the pins, the Base map chip, the 2D/3D square, DS-13) are the design's.
- **Segmented controls are joined**, with dividers, and the armed segment carries the chip inside it; the design
  draws its category chips apart, with gaps, and its tools as 64px tiles with icons.
- **Tool palettes keep a filled armed segment** (owner default Q5, as the port does); the design washes even its
  tool tiles.
- **The desktop's zoom + and −** are two separate round buttons; the design joins them in one pill column. (On
  touch, DS-13's − | + is one pill, as drawn.)
- **Undo and Redo are boxed** (10px, the theme's button box); the design draws them unboxed, with a 10px hover.
- **Fields are 26–28px tall at a pointer** against the design's 30px, so the same 12px corner reads a little rounder.
- **CIVIL's sub-tabs track at .02em**, not the design's .18em (1.8px): at .18em the five need 380px of the 371 the
  dock has at 1366.
- **Segment labels keep their own case** ("Region", "1K", "km"); the design's chips are written in capitals.
- **The window heading is 11px, as drawn**; the audit's plan had said 12px.
- **Type faces** stay the system stacks (owner default Q8): the design loads IBM Plex Mono and Helvetica Neue, so
  every width measured here (DejaVu Sans Mono and Inter on this machine) moves on another OS. **Icons** stay as they
  are (Q7): the design draws monochrome line icons where the file has emoji.
- **The status bar keeps its chips** (Q8); the design's holds three items.

**Found, not fixed:**
- `_lodRenderKey` (the LOD tiles' cache key) has no `state.bioBlend`, which `renderBiomeTileRGBA` reads — the same
  class as P7, in the LOD cache: in the Tiled LOD view the Relief ↔ biome slider would serve stale tiles. The LOD
  cache is v2.75's speed pass; not changed here.
- The phone has no way to the pack panel (above).
- At 1366 two heads of CIVIL ▸ Settlements' table are cut by the V01 fix's per-cell ellipsis: "Faction" (38px of
  36) and "Econ. role" (51 of 36). The columns line up, which is what V01 chose; a shorter label or a narrower
  column elsewhere would show them whole.

**Not in this build** (Ruling EF's order): the speed pass (P1–P3, P5, P6, P9, P10, the slider preview) — v2.75;
P8, keeping the view across a domain switch — owner default Q11, not built; V12's dark palette (Q6), V27's icons and
V31's fonts and status bar (Q7, Q8) — no change until the owner answers; V21's halo trial (the plan's §8, under
Q9's default the box stays) and the plan's re-measure of the 1100-px desktop window — not done here; moving long
help into `details.adv` (V25's note) — a content change, not made.

**What needs a person on a device:** one native select list and one number spinner in each theme (V02); Segoe UI
and Consolas on Windows for the widths measured here with Inter and DejaVu Sans Mono — the status chips (V14), the
sculpt picker with Freehand armed (V15, one feature out of sight at 1366), CIVIL's mono sub-tabs (V18/V24, they fit
1366 here), the dock's mono segments and action buttons and the rows they wrap; Credits from Settings ▸ About on a
real phone (V10); a screen reader over the strip's three filter chips (V29: it should still announce checkboxes);
a finger on DS-13's round buttons and the 12px touch controls on a real tablet and phone; the flat backdrop behind
the start card (Q10a) and the overlay taking the map's input mid-generate (Q10b), to be shown to the owner.
