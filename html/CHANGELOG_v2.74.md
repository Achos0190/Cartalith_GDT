## v2.74 (DCC line) — the fix pass: the Sun slider's cache key, the Settlements rows, Credits under Settings, AA contrast in both themes, the browser's own controls, the clippings, no render behind the start card, the shell's resize cost, and the planner's toll fix

Owner: *"can you check and improve the visuals and performance of the html? Maybe even upgrade some features based of
the godot version"*, then, on the order: *"Fixes and feature ports first then stage 3."* (Ruling EF, Cartalith_GDT
`LARGE_ITEM_RULINGS.md`). This version is the first of those: the fix pass of the 2026-11-01 audits of v2.72
(`html/audits/v2.72-2026-11-01/PLAN.md` §5 in Cartalith_GDT, each finding re-opened at its symbol in this file
first — v2.73 had moved several), and Ruling EC item 6, the planner's toll fix ("Apply Godot's fix"). The owner
questions the plan put take Ruling EF's stated defaults: the light accent `#8b550d` (Q3), the place popup's dark
palette unchanged (Q6), the darker box behind the scale bar and legend (Q9), a flat backdrop behind the start
card and a busy overlay that takes every input on the map (Q10a, Q10b), and the view still resetting on a domain
switch (Q11 — P8 is not built). `tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs v2.73 **ALL
IDENTICAL**. Verification is `tests/perf/probe_v274fix.js` (**131 assertions**; on v2.73 it fails **92**) and the
contrast gate `tests/perf/census_v274.js`: the visual audit's own WCAG census over its seven views (a place
selected on the map, the Settlements screen, the Asset Library, the route editor, the City viewer, Settings' five
pages, Credits), both themes, 1600×900 and 1366×768 — **0 enabled text runs below AA of 2 408** (v2.73: **561**).

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
  the Layers row 3.92 on the old light accent; 2.85 and 3.00 on the new) are `--accInk`. The census then found two
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
  spaces in `content:'▸  '`; it keeps 8px. The label rule is v2.77's.
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

**Measured** (this 4-CPU machine, which another agent may share; timings are indicative and the counters are the
gate): `tests/run.sh` 1280/0, `run_um.sh` 882/0, `hash_gen1.js` v2.73 → v2.74 ALL IDENTICAL (default, geoid,
waves, ao, icons). `probe_v274fix.js`: v2.74 131/0, v2.73 39 passed and 92 failed (each passing one is labelled
"as before" or set-up in its own text). `census_v274.js`: v2.74 0 of 2 408 enabled runs below AA (2 more per place
view, disabled controls' labels, are exempt and counted apart), v2.73 561 of 2 494. The load (P4), alone on the
machine, a fresh browser per sample, n=6 each, interleaved: the longest task 2 697 [2 325..2 971] ms on v2.73 against
332 [313..557] on v2.74, `DOMContentLoaded` 3 123 [2 748..3 509] against 858 [773..1 096] — disjoint. The shell's
own probes: `probe_mobileshell.js` 279/0 (as v2.73); `probe_desktopshell.js` 154/4, the same four by-design failures
as v2.73 (the two version checks, the two sections pinning v2.71's phone); RC's eight shell probes give v2.73's
results exactly (`shellui` 16/3, `domainrail` 14/6, `cogmodal` 33/0, `assetsbtn` 13/1, `settleparams` 28/2,
`geninfo` 21/0, `worldreset` 35/0, `typepreview` 41/0 — v2.73's CHANGELOG classifies each failure).

**Two flaky suite assertions, disclosed not fixed.** Of four `tests/run.sh` runs over this version's bytes, two passed
1280/0 (the final bytes' last run among them) and two failed ONE assertion each, a different one each time: *smoothed
bathymetry variance <= raw seabed variance (5.8e-3 <= 5.8e-3)* — v2.73's known flake — and *traced length sane vs
boundary-cell count (4721 vs 2208)*. Both build their world on an unpinned seed (`state.tect.seed` is
`Math.random()` at load, and `test_tail.js` generates on it). With `Math.random` pinned (seeds 101, 202 and 303), v2.73
and v2.74 print the same 1 280 assertion lines, timings aside, and pass 1280/0 each; `hash_gen1.js` is ALL IDENTICAL.
If either goes red, re-run before believing it, as for v2.25's SST assertion.

**Found, not fixed:**
- `_lodRenderKey` (the LOD tiles' cache key) has no `state.bioBlend`, which `renderBiomeTileRGBA` reads — the same
  class as P7, in the LOD cache: in the Tiled LOD view the Relief ↔ biome slider would serve stale tiles. The LOD
  cache is v2.75's speed pass; not changed here.
- The phone has no way to the pack panel (above).

**Not in this build** (Ruling EF's order): the corner radii (Ruling EC item 7) and the design alignment (V24, V25's
label rule, V26, V29, V30) — the second lane of v2.74; the speed pass (P1–P3, P5, P6, P9, P10, the slider preview) —
v2.75; P8, keeping the view across a domain switch — owner default Q11, not built; V12's dark palette (Q6), V27's
icons and V31's fonts (Q7, Q8) — no change until the owner answers.

**What needs a person on a device:** one native select list and one number spinner in each theme (V02); Segoe UI
on Windows for the widths V14, V15 (one feature hidden at 1366 with Inter) and V18 (359px of 371 with Inter)
depend on; Credits from Settings ▸ About on a real phone (V10); the flat backdrop behind the start card (Q10a) and
the overlay taking the map's input mid-generate (Q10b), to be shown to the owner.
