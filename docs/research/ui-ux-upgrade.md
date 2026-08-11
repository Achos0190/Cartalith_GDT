# UI/UX Upgrade Proposal — Cartalith Gen1

## Status (v0.66)

> **⚠ IA correction (v0.66) — owner decision overrides §3/Stage 2 of this proposal.**
> This document proposed moving Civilization + Cartography into the Explore phase; v0.64
> implemented that and it was **wrong** — the owner's intended information architecture keeps
> them as *Generate branches*. The shipped IA is:
>
> - **Generate** (authoring) — sub-tabs **World | Civilization | Cartography**
>   (geology/climate/erosion · factions/settlements/polity/ways · labels/icons/paint/style),
>   with the pinned Selection inspector shared by the Civ + Carto branches and the tool palette
>   split per branch (all buttons drive the one `_civSetTool` state machine).
> - **Explore** (planning/reading) — Info + Route tools, journeys, journey planner, the canvas
>   filter funnel and timeline.
>
> Everything else in the proposal stands as implemented. Do not re-apply §3's "Civilization and
> Cartography live in Explore" — it is superseded by this correction.

Every stage in the proposal is now implemented (as amended by the correction above). All
DOM/CSS/handler chrome over existing state — engine bit-identical from v0.62 through v0.66,
headless 852 green throughout, Playwright UI smoke grew from 12 → 27 → 41 → 50 assertions
covering every stage below.

| Item | Status |
|---|---|
| §4.1 Finalize flow | **Done in v0.62** (`state.finalized`, bake-all, lock, un-finalize) |
| §4.6 Right-click context menu | **Done in v0.62** |
| Editable settlement/POI inspectors (name/history/pop) | **Done in v0.62** |
| §4.4 Map-style presets | **Done v0.63** (Default/Antique/Ink/Watercolor/Print; Default bit-identical) |
| §4.3 Progressive disclosure | **Done v0.63** (Map style → 2 Advanced accordions; Tectonics coupling → Advanced) |
| §4.2 Layers popover | **Done v0.63** (grouped canvas FAB, proxies to hidden `#debugSeg`, MRU pins, curated Explore subset) |
| §3 Phase signal (tint + chip) | **Done v0.63** (on `state.finalized`; Unity play-mode tint + header chip) |
| §4.9 Onboarding empty-state | **Done v0.63** (first-run card, localStorage-dismissed) |
| §4.10 Small fixes | **Done v0.63** (stale Export hint; 360px sidebar ≥1440px); **extended v0.65** (per-layer hotkeys — B/T/F/S/W/R/0 bare-key shortcuts for the Layers popover's curated Explore subset, badge shown per item, guarded against firing while typing in any input) |
| **Stage 2 — IA re-homing** | **Done v0.64, corrected v0.66** — the durable parts: Edit tab retired (Tiles&LOD → Generate→World, Undo → header); "Places & roads (terrain)" retired outright (engine functions kept, UI gone, closing a real landmine — it shared `state.places` with civ settlements, so its "Clear places" could silently wipe them); Assets/Export header-level (v0.65). The **reversed** part: v0.64 also retired the Generate sub-tab bar and moved Civilization + Cartography into Explore — v0.66 restored the sub-tab bar (World \| Civilization \| Cartography) and moved both back under Generate per the correction note above. |
| §4.5 Tool-first palette | **Done v0.64, re-scoped v0.66** — Label + Icon folded into the `_civTool` state machine (were a separate checkbox/gallery system, `_carDisarmOtherTools`), closing a pre-existing gap where civtools and label/icon/paint weren't mutually exclusive; Icon's family-picker + gallery are the tool's contextual options block (Dungeondraft pattern), hidden until Icon is active. v0.66 splits the presentation per branch — Civ: Inspect·Settlement·POI·Territory·Way; Carto: Inspect·Label·Icon; Explore: Info·Route — all buttons share the one `[data-civtool]` wiring, so mutual exclusion holds across every branch and tab. |
| §4.7 Pinned selection inspector | **Done v0.64 "lite", completed v0.65** — v0.64 shipped a pinned summary card only, with the actual edit form still inline in the settlement/POI/label lists. v0.65 relocated the full edit forms (name/kind/pop/history/…) into the pinned inspector itself; `_civRenderSettlementList`/`_civRenderPoiList`/`_civRenderLabelList` now render rows + selection highlight only. `_civSelectedRowRefs` preserves the old inline version's live-row-patching optimization by handing the selected row's DOM refs to whichever editor the inspector renders. Extended to a third group — the Placed-Icons list's per-instance editor — so selection is single across all three (place/label/icon instance); picking one clears the others. A stale bug was caught and fixed along the way: the label list's delete handler only refreshed the label list itself, leaving a deleted label's editor stuck on screen. |
| §4.8 Global header undo / danger accents | **Done v0.64** — Undo (with live step-count text) moved to the header, always visible; `.al-danger` accent applied consistently to 8 destructive one-click Clear buttons; the three the proposal named by name (Clear territory/ways/places) additionally gained a confirm-when-non-empty guard (none had any confirmation before). |
| Assets/Export promoted to header-level utilities | **Done v0.65** — the tab bar is now a genuine two-position Forge/Atlas phase switch (just Generate + Explore). Export became a header dropdown (`#exportWrap`, mirrors Import ▾ but stays open across internal clicks since it's a form, not a one-shot action list); Assets became a plain header button (`_carEnterAssetsMode`) entering the same full-viewport Asset Library takeover as before. Exiting is automatic — clicking Generate/Explore always restores the canvas. |

**Remaining, deliberately out of scope**: the 29 debug views remain ungrouped only inside the
Layers popover's per-item list (grouping already landed in v0.63; hotkeys cover a curated 7-item
subset, not all 29) — no further reduction planned. Everything else the original proposal and its
Stage-2 write-up called for is now shipped.

---

# UI/UX Upgrade Proposal — Cartalith Gen1

*July 2026. Grounded in the `<body>` markup of `Cartalith Gen1 v0.61.html` (lines 263–1165) and web research
on comparable tools. Companion to `docs/research/ui-unified-tool.md` (2026-06 merge-era research) and
`docs/research/map-painting-ux.md`. UI/UX chrome and workflow only — no engine or canvas-render changes.*

## 0. Scope & constraints (unchanged, restated)

- One HTML file, zero dependencies, no framework, must open from `file://`.
- Existing dark aesthetic (`--bg/--panel/--accent` vars, mono small-caps headers) stays.
- Sidebar may change width/side but the layout stays **canvas-first**; mobile slide-in `aside` stays.
- Assume upcoming: **Polity** section (Economy+Politics merged, timeline slider), editable
  settlement/POI inspectors as collapsible lists, right-click viewport context menu, and a
  **"Bake all LOD layers"** finalize button that locks the Generate side.

## 1. Research — patterns from comparable tools

| Tool | Pattern worth borrowing | Where it lands here |
|---|---|---|
| **Azgaar's FMG** | *Layers menu with named presets* (political/biomes/heightmap…), per-layer hotkeys + tooltips; *Style editor* = one element-dropdown + opacity/color, with **style presets** (watercolor, cyberpunk…); *Erase / Keep / Risk* heightmap-edit modes = an explicit contract about what regeneration destroys; immediate vs deferred settings split | §4.2 Layers popover, §4.4 style presets, §3 finalize contract |
| **Wonderdraft** | Tool palette grouped by theme; **contextual Tool Options panel** — only the active tool's options are visible; pervasive single-key shortcuts | §4.5 tool-first Explore phase |
| **Inkarnate (2.0)** | Layers panel collapsible to an **icon-only rail**; **right-click context menu carries all object actions**; "advanced settings moved to modals"; object list drives z-order | §4.6 context menu, §4.2 rail |
| **Dungeondraft** | Themed tool tabs (Terrain/Objects/Design); *contextual controls* appear on select; snap toggle; praised as "unintimidating" — options count per screen is kept low | §4.5, §4.7 inspector |
| **Campaign Cartographer 3+** | **Anti-pattern.** CAD command order (tool → element → "Do it"), dense dated toolbars; community threads call it "very unwelcoming". Lesson: never make the user learn a private grammar; follow platform conventions | §2 audit lens |
| **QGIS** | Layer tree: checkbox visibility, drag z-order, groups, **visibility presets**; legend filtered by map content | §4.2 grouped layers + presets |
| **Figma / Photoshop** | Select on canvas → **right-panel properties inspector**; right-click context menu for context ops; left = structure, right = properties of selection | §4.7 inspector |
| **Unity / Godot** | Global **mode switch** (edit ⇄ play) that changes what is editable; Unity *tints the whole editor* in play mode so the state is unmissable | §3 phase tint/lock |
| **NN/g progressive disclosure** | Core layer (few, frequent, safe controls) vs advanced layer (rare, risky, expert); deferring advanced features measurably speeds initial task completion | §4.3 core/Advanced split |

Sources: Azgaar Quick-Start wiki & azgaar.wordpress.com "Styling the map"; wonderdraft.net + community manuals;
Inkarnate 2.0 layer/update announcements + loreteller.com layers guide; dungeondraft-encyclopaedia.gitbook.io;
ProFantasy community forum ("New to CC3+ … very unwelcoming"); docs.qgis.org General Tools + Lutra visibility-presets
post; help.figma.com right-sidebar & select-layers docs; Unity ProBuilder/play-mode docs; nngroup.com/articles/progressive-disclosure.

## 2. Audit of the current sidebar (v0.61)

### What already works — keep it
- **Consistent control idioms**: `.row` label/range/val, `.seg` segments, `.sec`+`h2`, `.hint`, `.acc`/`.cat-acc`
  accordions. Any redesign should reuse these, not invent new ones.
- **Header Import ▾ menu** (`#importMenuBtn`) — the v0.100 consolidation was the right move.
- **Explore filter funnel FAB** (`#explFilterFab` on the canvas) — already the Azgaar/QGIS "layers live on the
  map" idea; the proposal generalizes it rather than replacing it.
- **Inline expanding lists** (`#civSettlementList`, `#carLabelList` — "click a row to expand its editor directly
  underneath") — exactly the collapsible-inspector pattern the upcoming settlement/POI editors need.
- Educational `.hint` blocks; witty busy overlay; `#dbgOpacity` layer-opacity blend; mobile slide-in aside;
  `#manualTerrainChk` arming gate (a local prototype of the phase lock).

### What hurts — the five findings
1. **The phase journey is invisible.** Top tabs `Generate | Explore | Edit | Assets | Export` (`#tabBar`) mix
   *phases* (Generate, Explore) with *utilities* (Edit, Assets, Export). Civilization and Cartography — pure
   post-finalize activities — live under **Generate** sub-tabs (`#genSubBar`), while **Edit** holds Tiles & LOD
   and Undo, which belong to the Generate/Finalize side. Nothing tells the user "simulation is done, now decorate."
2. **Wall of sliders.** Tectonics is 13 rows + 2 checkboxes at one level; **Map style is ~29 sliders/toggles in one
   `.sec`** (17 Rendering + 10 Painter NPR + overlays). No core/advanced split anywhere (NN/g conditional
   disclosure absent). CC3+ shows where "everything visible" ends up.
3. **The 30-button debug picker** (`#debugSeg`: 29 views + Off, 6-col grid of cryptic labels "Oro", "Velo",
   "Carry Cap"). It mixes *user-facing map layers* (Biomes, Terrain, Flow, Settlement, Wildlife) with *developer
   diagnostics* (Stress, Age, Geoid, Bounds). Ungrouped, unsearchable, and duplicated conceptually with the
   Explore funnel and the `#modeSeg` view modes.
4. **Deep nesting + duplicated concepts.** Up to 5 levels (tab → sub-tab → `cat-acc` → `sec` → `acc`:
   e.g. Generate → World → Hydrology → Erosion → "Velocity (momentum)"). Two road systems (Edit → *Places &
   roads* vs Civ → *Infrastructure*) whose hints must explain each other; route drawing splits across Explore
   (journeys) and Civ (ways). Stale cross-references prove the IA outgrew its labels — the Export hint still says
   tile export is in "the Explore → Atlas tab" (line ~1127) when it actually lives under **Edit**.
5. **Weak safety & orientation affordances.** Destructive ops (`#genBtn`, `#reseedBtn`, erosion buttons) sit
   visually equal to view toggles; Undo is a button buried in the Edit tab (`#undoBtn`), invisible from where
   edits happen; there is no first-run empty state — a new user lands on the Geology accordion.

## 3. Design anchor — the three-phase journey

Make the owner's journey the *spine of the UI*: **① Generate → ② Finalize → ③ Explore/Narrate**.
One new state field (`state.phase: 'generate' | 'explore'`) plus a one-time **Finalize** transition.

### The phase rail
Replace `#tabBar`'s five tabs with a two-position **phase switch + utilities**:

```
┌────────────────────────────────────────────┐
│  ● Forge (Generate)   ─▶   ○ Atlas (Explore)│   ← phase switch (2 positions)
│  [Assets] [Export]  …header-level utilities │   ← moved next to Import ▾
└────────────────────────────────────────────┘
```

- **Forge/Generate phase** shows: today's *World* sub-tab content (Geology, Hydrology, Climate, Ecology,
  Manual Terrain, 3D view) + Tiles & LOD + Undo. Cursor/tools: sculpt, feature brush, region select.
- **Finalize** is not a tab — it is a **gated action** at the bottom of the Generate panel (see §4.1):
  bake all LOD layers → atlas, snapshot params, set `state.phase='explore'`.
- **Atlas/Explore phase** shows: Civilization (→ *Polity*), Cartography (names, icons, paint, style),
  Journeys/Planner, filters. The Generate panel collapses to a locked summary card.

### What morphs per phase (the contract, à la Azgaar Erase/Keep/Risk)
| | Generate phase | Explore phase |
|---|---|---|
| Sidebar | World params, erosion ops, LOD/tiles, undo | Polity, settlements, names, icons, paint, style, journeys |
| Canvas tools | sculpt/feature/region-select | inspect/place/route/paint-tint; right-click menu |
| Debug/Layers | full 29-view picker (grouped) | curated subset (Biomes, Terrain, Flow, Settlement, Wildlife, Flood) |
| Locked | — | `#genBtn`/`#reseedBtn`/erosion/planet/tectonics disabled + padlock badge |
| Signal | default accent | **header/aside border tint shifts** (Unity play-mode tint) + "World finalized ✓" chip |
| Escape hatch | — | **"Reopen simulation…"** confirm dialog listing consequences (atlas + civ layers may invalidate) |

This keeps both audiences honest: the generator can't silently clobber cartography (Azgaar's lock lesson), and
the cartographer is never one mis-click from re-rolling the planet.

## 4. Component-level recommendations

Each tagged **(effort S/M/L · impact low/med/high)**. IDs refer to v0.61 markup.

### 4.1 Finalize flow — the missing middle **(M · high)**
- New `.sec` at the bottom of the Generate panel: **"Finalize world"** — one accent button
  (`#finalizeBtn`) + readout of what will bake (uses the existing `#tileSizeEst` estimator and
  `#lodBakeBtn`/atlas machinery; "bake all" = iterate `bakeVisibleTiles` over the tile pyramid).
- Progress via the existing `#busy`/`#bakeProg` idioms; on completion write `state.phase='explore'`,
  flip the phase switch, show a one-time toast: *"World finalized — you're in the Atlas now."*
- Locking = add `body.phase-explore` class; CSS `pointer-events:none; opacity:.45` on Generate sections +
  a 🔒 row explaining "Reopen simulation" (`#reopenSimBtn`, `confirm()` in the existing
  `confirmRegenerate()` mold). Serialize `phase` in `params.json` like any state key.
- Cite: Azgaar heightmap Erase/Keep/Risk modes; Unity play-mode tint.

### 4.2 Debug picker → grouped "Layers" popover on the canvas **(M · high)**
- Promote `#debugSeg` out of the sidebar accordion into a second canvas FAB next to `#explFilterFab`
  (same `.dropdown` popover idiom, ◇ icon). Inside: **grouped list with full names**, one radio per view —
  groups: *Climate* (Temp, Köppen, Rain, Wind, Ocean), *Tectonics* (Plates, Bounds, Tect, Orog, Stress, Age,
  Geoid), *Hydrology* (Flow, Strahler, Velocity, Tides, Fjord, Flood), *Surface* (Biomes, Terrain, Lithology,
  Landforms, Soil, Water), *Civilization* (Resources, Carry, Settlement, Wildlife, Wind-throw).
- Keep `#dbgOpacity` in the popover header; pin the **3 most-recently-used** views at top (QGIS visibility-preset
  spirit, S-sized version). Keep all `data-d` values — this is a re-housing, not a rewrite.
- In Explore phase the popover shows only the curated subset (§3 table). The legacy 6-col grid can remain as a
  hidden fallback for one version.
- Cite: Azgaar Layers tab + presets; QGIS layer panel; Inkarnate icon-rail collapse.

### 4.3 Progressive disclosure pass — core vs Advanced **(M · high)**
- Adopt one rule: **a `.sec` shows ≤6 rows; everything else goes into an `<details class="adv">Advanced</details>`**
  (reuse `.acc` styling, label it "Advanced ▾").
- Tectonics (`#plates #vel #warp #seedN` + archetype = core; `#sigma #alpha #beta #age #flexure #hetero #resist
  #foldI #trenchD #faultB` = Advanced).
- Planet (core: `#pg #prot #ptilt`; Advanced: geoid + tide rows). Weather (core: `#weatherBtn #wIters #rainK`;
  Advanced: the other six). Erosion sub-accordions already do this — leave them.
- Cite: NN/g progressive disclosure (core/advanced layers); Inkarnate "advanced settings into modals".

### 4.4 Map style: preset row first, sliders second **(S–M · high)**
- Top of the *Map style* `.sec`: a preset `.seg` — **Default · Antique · Ink · Watercolor · Print** — each preset
  a plain JS object setting the existing `state.viz` keys (e.g. Antique = parchment 60 + sepia 40 + icons on).
  The ~29 sliders (`#parch … #pointillismR`) move under two `Advanced` accordions (Rendering / Painter NPR).
  A preset is just a starting point; touching any slider marks the seg "Custom".
- Cite: Azgaar style presets ("cyberpunk to watercolor") over its element-by-element editor.

### 4.5 Explore phase = tool-first, not section-first **(M · med)**
- In Explore, lead the sidebar with one persistent **tool palette** row (`.seg`): *Inspect · Info · Settlement ·
  POI · Label · Icon · Route · Way · Territory paint* — merging today's scattered `data-civtool` segs
  (`#civToolSeg`, Economy's territory button, Infrastructure's draw_way, Explore's route). Below it, a
  **contextual options block** shows only the active tool's options (Wonderdraft Tool Options; Dungeondraft
  contextual controls).
- Sections then reduce to content lists: **Polity** (factions `#civFactionPicker` + merged Economy/Politics +
  timeline `#civTimelinePanel`), **Settlements** (`#civSettlementList`), **Names & icons** (`#carLabelList`,
  `#carIconList`), **Journeys** (`#civJourneyList`, planner `#civPlannerSec`), **Map style** (§4.4).
- Retire the *Edit → Places & roads* section (`#placeChk #roadsBtn`) in favor of Civ Infrastructure ways —
  one road system, one mental model (fixes audit #4). Keep the engine functions; remove the duplicate UI.
- The **Polity timeline** additionally docks as a slim slider along the canvas bottom when the Polity section or
  timeline filter is active — Cartalith V1.914's bottom timeline dock precedent; `#explTimelineSlider` already
  exists in the funnel popover and can be mirrored.

### 4.6 Right-click context menu on the viewport **(M · high)**
- One `contextmenu` handler on `.canvas-wrap`, hit-testing in this order: settlement/POI → label → icon →
  way/route → map cell. Menu items per hit (Inkarnate/Figma pattern):
  - Settlement: *Rename · Edit details… (opens inspector §4.7) · Set faction · Start route here · Delete*.
  - Empty land (Explore): *Drop settlement · Drop POI · Add label · Place icon*.
  - Empty land (Generate): *Sculpt here (arms brush) · Set sea level to this height · Inspect cell (Info)*.
- Same `.dropdown` CSS; long-press (~500 ms) triggers it on touch. Esc/blur closes. This also gives every
  canvas-only feature a second, discoverable path (fixes CC3+-style "invisible grammar" risk).

### 4.7 Selection inspector — one place for "properties of the selected thing" **(M · med)**
- A pinned `.sec#inspector` at the **top** of the Explore sidebar: empty state "Select something on the map";
  on selection it renders the existing inline editors (settlement editor, label editor, icon editor) — the
  upcoming editable settlement/POI inspectors (name/history/population) slot directly here. Lists (§4.5)
  stay for browsing; the inspector is where clicking-the-map lands you (Figma left=structure / right=properties).
- Reuse `#settleInfo` popup content for the read-only Info tool; "Edit details…" promotes it into the inspector.

### 4.8 Global undo + safety affordances **(S · med)**
- Move Undo from the Edit tab into the **header** (`↩ Undo` + step count, next to Import ▾), always visible,
  tooltip shows what it will undo (`#undoMem` text). Keep Ctrl-Z routing.
- Destructive buttons get the `.al-danger`-style red accent consistently (`#reseedBtn` keeps `confirmRegenerate()`;
  add the same to Clear territory/ways/places when non-empty).

### 4.9 Onboarding / empty state **(S · med)**
- First run (no `field`, no save): a centered card over the canvas — three buttons: **Generate a world**
  (fires `#genBtn`), **Load project** (`#loadZipBtn`), **Import heightmap** (`#loadBtn`) — plus a one-line
  phase-journey diagram (Forge → Finalize → Atlas). Dismiss forever via localStorage flag; pure DOM, ~40 lines.
- Cite: Azgaar's guided quick-start need (its own wiki exists because the app lacks this in-product).

### 4.10 Small fixes, immediate **(S · low–med each)**
- Fix the stale Export hint ("Explore → Atlas tab" → wherever Tiles lands after §3).
- Add per-layer hotkeys to the Layers popover tooltips (Azgaar); `B`iomes, `F`low, `T`emp…
- Sidebar width: allow 360px at ≥1440px viewport (`@media`), keeping 324px baseline; canvas-first preserved.
- Keep `.seg.grid` CSS — the Layers popover reuses it for icon rows if needed.

## 5. Staged rollout (each independently shippable + verifiable)

**Stage 1 — Phase model + Finalize (S/M).** Add `state.phase`, the phase switch UI, `#finalizeBtn` wiring to the
existing atlas bake, lock-CSS + "Reopen simulation". No sections move yet — Generate/Explore tabs simply gain the
lock behavior. *Test:* headless — `state.phase` serializes/merges; finalize on a tiny grid marks atlas chunks;
locked ops early-return. Browser — tint + lock visible, reopen works.

**Stage 2 — IA re-homing (M).** Move Civilization+Cartography panels under Explore; move Tiles & LOD + Undo under
Generate; retire Edit tab (its two survivors re-homed) and Places & roads UI; Assets/Export become header-level.
Pure DOM moves in the `#tabBar` handler's mold (v0.124 precedent: "DOM-move only ⇒ bit-identical"). *Test:* all
`getElementById` wiring intact (grep-audit ids), generate/render hashes unchanged.

**Stage 3 — Layers popover (M).** Re-house `#debugSeg` as the grouped canvas popover with MRU pins + curated
Explore subset. *Test:* every `data-d` still reachable; default view Off; render bit-identical when Off.

**Stage 4 — Disclosure + presets (M).** Core/Advanced split per §4.3; Map-style preset row per §4.4. *Test:*
presets are pure `state.viz` writes (assert known checksums per preset at a pinned seed); Advanced-hidden
defaults bit-identical.

**Stage 5 — Selection & narration chrome (L).** Context menu, pinned inspector, global undo in header, onboarding
card, Polity bottom-dock timeline. *Test:* hit-test order unit-testable headlessly; menus/inspector are
browser-verified; no serialization of transient UI (invariant 6).

## 6. Risks / non-goals
- **No engine or renderer changes** — every stage is DOM/CSS/handler work over existing state keys and ops.
- Phase lock must never brick a save: `phase` merges with a `'generate'` default in `loadZip` so legacy projects
  open unlocked.
- The 29 debug views stay — grouping is presentation; removing views is out of scope.
- Mobile: phase switch and Layers popover must stay reachable in the ≤860px slide-in layout; context menu = long-press.

## Sources
- Azgaar FMG: github.com/Azgaar/Fantasy-Map-Generator/wiki/Quick-Start-Tutorial; azgaar.wordpress.com/2018/05/06/styling-the-map
- Wonderdraft: wonderdraft.net; community manual (Tool Options panel workflow)
- Inkarnate 2.0: inkarnate.com/updates; loreteller.com/learn/inkarnate-layers-guide; Threads @inkarnaterpg custom-layers announcement
- Dungeondraft: dungeondraft-encyclopaedia.gitbook.io (Terrain tab, Contextual Controls); encounterlibrary.com basics
- CC3+: forum.profantasy.com/discussion/7589 ("very unwelcoming"); blackgate.com CC3+ review
- QGIS: docs.qgis.org General Tools (layer panel); lutraconsulting.co.uk visibility-presets post
- Figma: help.figma.com right-sidebar / select-layers / context-menu docs
- Unity: docs.unity3d.com ProBuilder edit-mode toolbar; play-mode tint convention
- NN/g: nngroup.com/articles/progressive-disclosure
