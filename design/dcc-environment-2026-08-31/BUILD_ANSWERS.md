# Answers to the six specification passes

*Owner-supplied, imported verbatim from the design project 2026-08-31 as the
reply to `OPEN-QUESTIONS-FOR-DESIGN.md`. Copied here so the build can cite a
reason without a round trip.*

---

Decisions are in the files; this records them so the build can cite a reason. Dated 2026-08-31.
Files: `Cartalith DCC Environment.dc.html` (windows/tablet), `Cartalith Android.dc.html` (phone/tablet),
`cartalith-dcc-parts.js`, `landmark-glyphs.js`.

## 1 · The 256 KiB cap

Split, not stripped — the terrain draw stays, because a flat placeholder would have removed the only
evidence the docks are reading a real field.

- `Cartalith DCC Environment.dc.html` — **239,712 bytes** (was 276,676). Under the cap with ~22 KB of margin.
- `cartalith-dcc-parts.js` — 53,889 bytes. Heavy method bodies: map drawing (`draw`, `drawExtra*`,
  `_terrPaint`, `_updateHud`, `sampleData`), static data tables (`CAD`, `RAMPS`, `LMFAMS`, `scFeats`,
  `PSTAGES`, `FACTIONS`, `PRESETS`, `RND`, `AND`), menu rows (`menuRowsFor`), and the settled
  `vals5` / `vals6` / `valsCarto` blocks.
- Mechanism: `window.CDCC = { … }`, each entry called with the component as `this` by a one-line
  delegate left in the logic class (`draw(...a){return window.CDCC.draw.apply(this,a)}`). No behaviour
  change; the class is still the only stateful object. `<script src="./cartalith-dcc-parts.js">` sits
  first in `<helmet>`, so it is loaded before the class is applied.
- Both new files are plain `.js` — fetch them alongside the DC.

Everything §2 listed as missing (`ldPipe`, the run block, `rdTitle`, `sampleRows`, `regionRows`,
`tbLabel`, the measure handlers, `vpContext`, `scrimBg`, `mapCursor`, `layersBtn*`, `l.bg`/`l.col`,
`masterOpPct`, `tlShow`/`tlCollapsed`/`tlExpanded`) was present — it was inside the truncated tail.
Re-fetch and they are all there. `statusMid`, `railChev`, and undo trimming were in the same region;
see below for the two of those that changed anyway.

## 2 · Decisions

**2.1 CARTO — four rail nodes, four destinations.** Four panels, matching how CIVIL resolved.
Modes `style` · `labels` · `icons` · `terrain`, `state.cartoCat`, accordion headers in the left dock
with live counts, same grammar as the CIVIL categories. LAYERS & STYLE and TERRAIN APPEARANCE are the
existing bodies; LABELS and ICONS are new and real:

- **LABELS** — five classes (Continental · Region · Settlement · Water · Landmark) with per-class
  size / halo / tracking and a drawn count, plus collision culling with its suppressed count.
- **ICONS** — family chips (PLACES · TREES · SEA MARKS · POI), slots-filled line, icon scale, minimum
  spacing, and three placement rules (avoid label boxes · enforce min spacing · snap sea marks to coast).

**2.2 `statusMid`.** `last pass 09 Ecology · 101 ms · repaint 84 ms · autosave 14:02`. The stage name is
the last stage whose `stageState()` is `resolved`, so the middle of the bar always says what the map
currently rests on. Autosave shows `off` when autosave is off.

**2.3 Tablet threshold — shortest-width ≥ 900 dp gets the desktop-parity shell.**
Below 900 dp: the phone shell with touch density. Not Android's 600 dp, because the desktop shell has a
hard floor: 48 rail + 400 dock = 448 dp of chrome before any map. At 800 dp that leaves a 352 dp map —
narrower than the dock beside it, which is not the Windows GUI in any useful sense; at 900 dp it leaves
452 dp, and the map is the larger pane again. So `TABLET 800` is deliberately phone-shaped, and it is on
the correct side of the line. The Environment's `TABLET PORTRAIT` frame (1600 dp shortest width) and
`TABLET 2560` are both above it. Stated in both files' headers.

**2.4 The three unscaled values — all three now scale.** They were oversights.
`--railExpW` 200 → 264 px · `--popW` 238 → 300 px · `--hero` 26 → 30 px · `--hero2` 22 → 26 px on touch
frames. The hero readouts are now tokens, so the ratio to the 11 px mono around them holds.

**2.5 Rail interaction.** Clicking the already-active domain **toggles the expansion column** (it is the
only affordance in reach when the panel is closed). Clicking a node sets the domain, sets that domain's
mode, and closes the expansion — consistent with `setDomain`. The chevron is **one glyph, `▸`, rotated
0° / 180°**, matching the dock accordions; `›`/`‹` is retired.

**2.6 `scrimBg`.** Defined, and it does differ: dark `rgba(13,14,15,.62)`, light `rgba(244,242,238,.72)`.

## 3 · Declared but inert — the calls

| What | Call | What changed |
| --- | --- | --- |
| 9 colour ramps | intended | Note on the control: a ramp sets the legend and export LUT; the viewport keeps its built-in relief ramp in this build |
| 6 simulation layer toggles | intended | Note on the timeline: they record which layer you want; no layer renders yet |
| 8 Data-manager routes | oversight | Not in this file — the Environment has no data window at all (`win:data` toasts). Lives in `Cartalith DCC Shell.dc.html`; build it there or ask and I will add the window here |
| Stale-field rendering | oversight | **Wired.** Fields owned by a stale stage now render read-only as `label · —` |
| Undo depth 1–50 | intended | Already trims: `pushUndo` slices to `prefs.undoDepth` |
| 3D viewport toggle | oversight | **Wired the missing half** — Preferences ▸ Graphics now has `relief exaggeration 1× / 2× / 4×`, so the toast points somewhere real |
| 13 of 24 asset families | oversight | Same as the data window: not in the Environment. Fix in the Shell file |
| Asset slot grids · per-slot data | oversight | Same file |
| Landmark viewshed | intended | Stated on the panel: once visibility analysis lands, `score = 0.6 × prominence + 0.4 × visible land area inside 30 km`, caps unchanged |

## 4 · Phone rulings

Changed in the file:

- **Safe areas.** Both status bands and the gesture band are now `max(env(safe-area-inset-*), mock)` —
  the mock value is the floor, the real inset wins when it is larger. Nothing else reserves space.
- **System back.** Yes, the same chain as Escape: modal → search → menu → inspector → sheet. Bound via
  `popstate` with a re-pushed state, so back never leaves the app while anything is open. At the root it
  falls through to the system.
- **Keyboard avoidance.** `visualViewport` resize writes `state.kb`, and `dockBottom` adds it, so the
  LABEL bar and everything docked to the bottom ride above the IME.
- **Light theme.** `--warn` `--block` `--good` `--water` now have light values
  (`#9a6a12` `#a04437` `#4e6f3f` `#3f6675`).
- **Empty states.** Search has a no-results row that also says what search does not cover.
- **Persistence.** Device, theme and units persist to `localStorage['cartalith.shell']` and restore on
  load. Coach marks keep their own key.
- **Haptics.** One table, `_haptic(kind)`: sample 12 ms · detent 8 ms · tool arm 10 ms ·
  verdict `[14,40,14]` · back 6 ms · blocked `[20,60,20]`. Wired at tool arm, detent snap and back.
- **Focus.** A `:focus-visible` ring (2 px accent, 2 px offset) applies shell-wide.

Deliberate, disclose rather than build:

- **The map canvas staying dark in light theme.** Light chrome over a dark map is the intended pairing;
  a light map is a style preset, not a theme consequence.
- **Rotation with a sheet open in landscape.** Portrait re-snaps because it owns detents; the landscape
  drawer has no detents to re-snap to.
- **Generation-failure and storage-full states.** Not designed yet — ask and I will add both.
- **Content descriptions and dynamic type.** Not in this prototype. The 48 dp audit is partly done
  (rows and chips are 48; a few glyph buttons are 44) — say the word and I will sweep every target.
