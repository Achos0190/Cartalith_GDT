# CLAUDE.md additions for v2.74 (paste into Cartalith_RC's root `CLAUDE.md`)

Two pastes. Neither changes anything else in the file. (v2.73's two pastes go in first.)

## 1 · The file table

**Replace** the opening of the `Cartalith v2.73 DCC test.html` row,
`**The DCC shell line's current head, not a mainline version.**`, with `Previous DCC-line file.` (the rest of
that row stays as it is — the convention every older row follows), and **insert this row above it**:

```
| `Cartalith v2.74 DCC test.html` | **The DCC shell line's current head, not a mainline version.** Owner: *"can you check and improve the visuals and performance of the html? Maybe even upgrade some features based of the godot version"*, then *"Fixes and feature ports first then stage 3"* (Cartalith_GDT Ruling EF). **THE FIX PASS AND THE DESIGN ALIGNMENT of the 2026-11-01 audits of v2.72**, each finding re-opened at its symbol here first, with **Ruling EC item 6, the planner's toll fix** (`_jpClaimedAt` `> 0`, as Godot's `jp_claimed_at`: no sea or Unclaimed cell reads as claimed, so water stages lose the claimed-land floor and a route between claimed and unclaimed ground pays its frontier toll — measured and disclosed in the CHANGELOG) and **item 7, rounded corners** (*"Rounded, as the design"*). **THE RENDER CACHE KEY NAMES STATE FIELDS**: `_civBakeKey` read `state.sun` (the Sun slider writes `sunAz`) and `state.shadeOnHypso` (a checkbox's id) and lacked `state.showRivers` — three controls served a stale image; a sweep of all 47 render-calling controls finds none now. **No render behind the start card** (a flat fill; Skip renders), **the busy overlay above the map's controls** (z 30), **Credits above Settings** (76), **#assetsPanel reachable in Assets mode**, **AA contrast in both themes** (`--faint` re-based, light `--accent` `#8b550d`, `color-scheme` per theme, panel-backed windows; the WCAG census over the audit's seven views at 1600 and 1366 in both themes: 0 enabled text runs below AA, 561 on v2.73), the clippings, and the shell's forced layouts per class crossing 9 → 8. **SURFACES ARE SQUARE, CONTROLS ARE ROUNDED**: v2.23's `*{border-radius:0 !important}` is gone; five tokens from the design (`--r-btn` 16/24, `--r-tile` 14/20, `--r-field` 12, `--r-menu` 10, `--r-pill` 999px, pointer/touch), each of the 101 radius declarations it had silenced decided where it is written. **ONE ON-STATE**: an armed segment, pill or row is WASHED (a chip of wash2 with a 1px accent edge, drawn inside the segment); the fill stays only on a tool-mode segment (owner default Q5). The dock's controls take the design's mono type, one section-label rule over nine selectors, one window-heading rule, and the timeline strip's filters are washed chips around real checkboxes. `tests/run.sh` 1280/0, `run_um.sh` 882/0, `hash_gen1.js` vs v2.73 **ALL IDENTICAL**. Verify with `tests/perf/probe_fixpass.js` (pass v2.73 second for the truncation census; it prints its count — see CHANGELOG v2.74) and `tests/perf/census_v274.js`. |
```

## 2 · The section

**Insert above** `### The phone and tablet shells, stage 2: one decider, one sheet, the pull-out tab, DS-13 and the touch layer (v2.73, DCC-line file only)`
(DCC-line sections run newest first):

```markdown
### The fix pass and the design alignment: a cache key names state fields, AA in both themes, rounded controls on square surfaces (v2.74, DCC-line file only)

Owner: *"can you check and improve the visuals and performance of the html?"*, then *"Fixes and feature ports first
then stage 3"* (Cartalith_GDT Ruling EF). The fix pass and the design alignment of the 2026-11-01 audits of v2.72
(`html/audits/v2.72-2026-11-01/` in Cartalith_GDT), with Ruling EC item 6 (the toll fix) and item 7 (rounded corners,
*"Rounded, as the design"*). `tests/run.sh` **1280/0**, `run_um.sh` **882/0**, `hash_gen1.js` vs v2.73 **ALL
IDENTICAL**; verification is `tests/perf/probe_fixpass.js` (every finding at its bar; on v2.73 most of it fails —
each assertion that passes there says "as before" in its own text; give v2.73 as a second argument and it also
takes the truncation census on both) and the contrast gate `tests/perf/census_v274.js` (the visual audit's WCAG
census, its arithmetic unchanged).

- **Every part of a cache key must name a field the state really has.** `_civBakeKey` read `state.sun` and
  `state.shadeOnHypso`; the state has `sunAz` and `shadeHypso`. A misnamed part is `undefined` in every key, so its
  control silently serves the cached image. **And a render input missing from the key is the same defect**:
  `state.showRivers` was never in it. The probe checks both ways — every `state.X` in the key against the state
  literal, and every render-calling control (found in the SOURCE: `bind(...render())` and `change` listeners that
  call `render()`) changed once and compared with a cache-cleared render, in Biome and Relief view. Add a new render
  input to the key in the same change, or its control does nothing on screen. (`_lodRenderKey`, the LOD tiles' key,
  still lacks `state.bioBlend` — found, left for the speed pass.)
- **Re-basing a token re-opens every pair built on it.** `--faint` (dark `#7d8285`, light `#676762`) clears AA on
  the four grounds and still FAILS on a wash (dark 4.17/3.63, light wash2 4.17): never set faint text on a wash. The
  light washes keep the old accent's hue on purpose (on `#8b550d` the accent on wash2 would be 4.41). An ink on the
  accent fill is `--accInk`, never a literal; an ink on a DATA colour (a faction's) is chosen per colour by contrast
  (`_civInkOn`). Never fade text that is already the lowest ink (`opacity` on `--faint` put two lines below AA).
  Run `tests/perf/census_v274.js` after any colour change: 0 enabled runs below AA is the bar.
- **The boot does not render the empty world.** The browser branch sizes `#view` to GW×GH and fills it flat
  (`BOOT_OCEAN`); "Skip for now" renders it once (`_bootRenderPending`, cleared by any render). Generate, Load and
  Import render anyway. A probe that reads the canvas before generating sees the flat fill, not an ocean — generate
  first, as every RC probe already does. The headless branch is untouched.
- **The busy overlay is z 30, over everything drawn on the map and under every menu, popup, phone frame band and
  window.** A new control floating on the map must sit below 30, or it stays pressable mid-generate; a new layer
  that must stay usable during a generate (a menu, a window) must sit above it.
- **In Assets mode the left dock stays, holding `#assetsPanel` alone** (desktop and tablet; the v2.24 rule's v2.74
  note): its head reads *Assets › Pack*, and the docking rule asks the LIBRARY to keep `AL_STAGE_MIN` (its rail,
  inspector and two tile columns), floating the dock over it otherwise. `_carEnterAssetsMode`/`_carExitAssetsMode`
  re-run `_shellDockSync` and `_dskDockHeadSync`. Anything else added to the left dock is hidden in this mode by
  that rule; the phone still cannot reach the panel (its domain page leaves the library first).
- **Never measure a band right after moving controls, or in a `resize` listener that runs before the controls
  move.** `--chrome-top`/`--canvas-top` are published in the refit's frame (`_refitViewportNow`, first), every band
  change schedules that refit, and `_syncChromeTop` reads both bands before writing either and writes only a changed
  value. A timeline trace names the script behind each forced layout (`Layout` events' `stackTrace`); the probe
  asserts none comes from `_shellPlace` or from a pre-placement listener.
- **Restyle an active state by resetting the border shorthand first**, then drawing the one side
  (`.subtab.on`, `.set-nav button.on`): the theme layer's `button.on` border fills every side an `.on` rule leaves.
- **Surfaces are square, controls are rounded — and the radius lives in five tokens** (the theme layer's radius
  block, read off `design/mobile-shell-2026-10-31`: pointer value, then touch): `--r-btn` 16/24 (every button that
  draws a box; a one-line button is a pill, a wrapped one keeps its corners), `--r-tile` 14/20 (a selectable tile),
  `--r-field` 12 (inputs, selects, textareas), `--r-menu` 10 (the menu bar's buttons, Undo/Redo, the strip's
  TIMELINE toggle), `--r-pill` 999px (chips, pins, pills, map buttons). Panels, docks, bands, windows, menus,
  popovers, cards, wells, toasts and the map are square (Ruling EC item 7: "Panels and bands stay square"); rows,
  tabs and the rail are squared by one rule, because a rounded full-width row curves its own divider. **Never bring
  back a universal radius reset**: it hides every rounded control, and an `!important` one beats inline styles too —
  that is how 101 declarations (66 in the stylesheet, 35 inline in the markup and the JS templates) were silenced
  from v2.23 to v2.73. A new control takes a token; a new surface writes no radius at all; a new inline style writes
  none either (a stylesheet rule cannot beat it).
- **A segmented control is ONE rounded container that clips its square children** (`.seg`, `overflow:hidden`), so its
  inner dividers stay straight. **Its armed segment is washed — a chip drawn 2px inside it (`::before` under the
  label), never a ring on the segment's own box**: the container's curve cuts a box's edge off at the end segments,
  and an inset chip leaves the segment's width and every divider where they were. A segment sized to its label
  needs side padding for the chip (`#reParty`'s Carriage switch had 1px). **The fill belongs to a tool-mode segment
  only** — the one that says which map tool is armed (`[data-civtool]`, Sculpt's feature picker), as the port keeps
  its SCULPT / PAINT / MEASURE segments filled (owner default Q5; the 2026-09-06 ruling, item 7, washed everything
  else). A new segmented control is washed unless it arms a tool; an armed pill or list row is washed too.
- **One rule each for the dock's control type, the section label and the window heading.** Dock segments: mono 10px,
  weight 500, .08em, each segment's basis its own label on one line (at an equal share mono labels broke inside their
  segment at 1366). Dock action buttons (`.dock .btnrow > button`): mono, weight 500, uppercase, .12em (.14em and
  10.5px on `.accent`). Section labels: the nine selectors in the D4 block's rule (mono 9.5px, 500, .2em, uppercase,
  `--dim`), location labels kept in the accent. Window headings: the theme layer's rule (mono 11px, 500, .2em,
  uppercase, accent), the City viewer's place name exempt. **A new label or heading joins its rule's selector list —
  never a type of its own**: that is how one role came to be drawn eight ways.
- **The strip's filters are their LABELS drawn as chips; the checkbox stays inside, visually hidden but focusable**
  (`opacity:0`, 1px, never `display:none`, which takes it out of the tab order), so Tab, Space, the ring
  (`:has(input:focus-visible)` on the chip) and a screen reader's "checkbox" all still work.
- **Mono uppercase is about 1.3x wider: a row that runs out of room wraps.** After any change to dock type, take the
  truncation census (`probe_fixpass.js` with a before file) and read its list of rows that wrap.
```
