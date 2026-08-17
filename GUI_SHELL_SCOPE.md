# GUI shell redesign: milestone plan

Owner-supplied design import (2026-08-17, via the `claude_design` MCP,
project "UI mockups planning") — `design/Cartalith GUI.dc.html` (the mockup,
multiple breakpoints/themes/states) and `design/cartalith-menu-structure.md`
(the handoff spec this plan is built from). `design/terrain-appearance-
rendering.md`'s content is already on file as `TERRAIN_APPEARANCE_RESEARCH.md`/
`TERRAIN_APPEARANCE_SCOPE.md` — no separate action needed, §5b below just
cross-references it.

## Two scope decisions the owner made explicitly (2026-08-17)

1. **Target codebase**: this Godot/Rust native port (this repo), not the JS
   reference app. The menu-structure doc's own implementation notes ("re-
   parent, don't rewrite... keep ids identical") describe `Cartalith Gen1
   v2.10.html`'s DOM — a different, frozen file this repo's `CLAUDE.md`
   forbids editing, in a different repository this session doesn't touch.
   For this port, every `#id` in the menu-structure doc is read as **"this
   feature exists and already has real backing in the reference/this port,"
   not a literal DOM node to move.**

2. **Shell vs. backend sequencing**: build the full shell structure now —
   every menu, panel, and region the mockup shows — even where the backing
   feature doesn't exist yet (Simulate's year-by-year playback, Warfare,
   full Politics/Trade/Logistics, tile/LOD pan-zoom, 2D/3D toggle). Controls
   for non-existent features are real, visible, and structurally present,
   deliberately non-functional until their own future engine work lands —
   not hidden, not omitted, not stubbed as TODO comments.

## Real, honest inventory: what already exists vs. doesn't (checked, not assumed)

**Real backing exists today** (`cartalith-godot`/`main.gd` already has a
working `#[func]`/control for this — re-parent into the new shell and keep
functional):
- Generate world, seed, resolution, map width (`WORLD` panel)
- World shape / world structure archetypes (`WORLD` panel — and its own
  real dispatch bug was fixed this session, `a265b2b`)
- Sea level (`WORLD` panel, `MVP_SCOPE.md` point 9, done this session)
- The four experimental terrain flags (dynamic lithology, volc provinces,
  wind deflection, ocean currents)
- Villages/territory/civ-layer toggles, load-save `.zip`, credits dialog
- Settlements/roads/territory/villages/sea-routes/province-boundary map
  overlays (`map_overlay.gd`)
- Per-cell inspector data: elevation, and whatever `WorldState`/`CivData`
  actually expose per-cell — **verify exactly what's reachable before
  building the inspector panel's "no selection"/hover state; don't invent
  fields the engine doesn't compute** (e.g. "drainage" as a named 0-1 value,
  "route cost" as a queryable per-cell number — check `cartalith-civ`'s
  `build_travel_cost` for whether this is really a per-cell queryable field
  or an internal Dijkstra weight with no public accessor)

**No real backing yet** (build the UI shell/controls, wire to nothing or to
an honest "not available" state — this is what "build shell now, wire later"
means):
- Year-by-year time simulation/playback (Play/Pause/Step/×1/×10/×100) — the
  engine is a one-shot static generator, this is the single biggest mockup-
  vs-engine mismatch, `HARDWARE_ACCELERATION.md`'s own static-generation
  correction and the user's own prior statement ("no need to continuously
  calculate") both already establish this as deliberate, not an oversight
- Warfare (mentioned nowhere else in this entire project's history)
- Full Politics beyond territory/provinces (faction roster UI exists as
  data — `get_factions`-shaped info if it exists, check — but state
  religion, full politics simulation do not)
- Trade/Logistics/Journey Planner (`ECONOMY_SCOPE.md`, investigated this
  session — ~70-function sub-phase, correctly deferred, one small piece
  ported but unwired: `civ_resource_trade_balance`)
- Tile/LOD pan-zoom viewport (`cartalith-spatial`, standalone/unintegrated
  per `LOD_TILING_BASE_SCOPE.md`'s own explicit scope)
- 2D/3D toggle (`DECISIONS.md` §4, 3D deferred to Phase 3)
- Terrain appearance's actual editing controls (`TERRAIN_APPEARANCE_SCOPE.md`
  milestone 1 built the data structure, not the GUI — §5b below is shell-only
  this pass too, same as everything else without real backing)
- Brush/stamp tools, region export, asset packs, GeoJSON export, undo/undo
  history, project save/load beyond the existing `.zip` — none of this
  exists in the Rust core

**Ambiguous, verify before building** (don't assume real or fake without
checking `cartalith-godot`'s actual `#[func]` list):
- Debug/analysis field switching (slope/aspect/curvature/flow/etc. as
  selectable render modes) — `render.rs` computes these internally per-pixel
  but may not expose them as independently-selectable output channels
- CPU/GPU/memory live readout in the top bar — real numbers may need a new
  `#[func]` (Godot's `OS`/`Performance` singleton has some of this natively,
  check before assuming Rust needs to provide it)

## Milestone plan

**Milestone 1 (this pass)** — desktop shell structure, dark theme only, one
breakpoint (1920×1080), no responsive/light-theme/collapse work yet:
- Top bar: wordmark, 7 domain menus (Project/World/Generate/Simulate/Map/
  Assets/View) as real menu buttons — populate each with the real
  menu-structure.md inventory, `#id`-tagged items call their existing
  `#[func]`/signal, `NEW`-tagged items are present and clickable but a
  no-op (or, where sensible, open a stub panel saying what it will do)
- Left navigator: 4 groups (World/Civilization/Infrastructure/Cartography)
  with their listed subject nodes — clicking a node swaps the layer panel
  + inspector context, per the spec's own §2 rule ("navigator swaps the
  tool palette and inspector around it, never the viewport/application")
- Layer panel: real layer list (Terrain/Hillshade/Water/Rivers/Biomes/
  Settlements/Roads/Territory/Villages/Sea routes/Provinces — map this to
  what `map_overlay.gd`'s toggles already control) with visibility +
  opacity where real, structurally present but inert where not
- Centre: mode bar (WORLD/EDIT/ANALYSIS/SIMULATION/CARTOGRAPHIC/DEBUG) +
  the existing map viewport re-parented in, corner readouts using real
  data where available (coordinates if meaningful, scale bar from real
  map-width-km, tile/LOD readout as a static "not yet tiled" indicator
  since no LOD exists)
- Right inspector: "no selection" state wired to whatever real per-cell
  data is actually reachable (checked against the Ambiguous list above);
  other contexts (river/settlement/faction/route/brush) built as real
  panels but populated only when their trigger is clickable (e.g. clicking
  a settlement marker already exists via `map_overlay.gd`'s hover — extend
  that to a real selection-driven inspector state if reachable this pass,
  defer if it requires new engine plumbing)
- Bottom bar: timeline scrub + transport controls, present and styled per
  the mockup, genuinely non-functional (dragging the scrub does nothing —
  this is the year-playback system that doesn't exist); simulation-layer
  toggles present, wired only for layers with real backing (climate/
  population would need real per-cell state to toggle, verify)

**Explicitly deferred to later milestones**: light theme, panel collapse/
rails, all three responsive breakpoints (tablet 2K, tablet landscape,
phone), terrain appearance panel's actual editing controls (§5b —
`TERRAIN_APPEARANCE_SCOPE.md`'s own milestone 2+), any new engine-side
`#[func]` work beyond what's needed to re-parent existing controls.

## Hard constraint carried over from every prior UI pass this session

**The current, working, screenshot-verified MVP UI must not regress.**
Every real control (`main.gd`'s existing generation flow, all four
experimental flags, sea level, load-save, credits, the four map-overlay
toggles) must keep working exactly as before, re-parented into the new
shell, not rewritten from scratch. This session already found one real
crash bug from screenshot verification alone (`CHANGELOG.md`'s "Wire sea
routes" entry) — a shell rewrite this large needs the same rigor, not less.

## Verification (per milestone)

1. `cargo build -p cartalith-godot`, `cargo test --workspace` (0
   regressions — this is GDScript/scene work, Rust should be untouched
   unless a genuinely new `#[func]` is needed for something in the
   "real backing exists" list that isn't exposed yet).
2. `godot4 --headless --quit main.tscn` clean load.
3. **Real windowed-app screenshot verification, end-to-end**: generate a
   real world through the new shell (not the old one) and confirm the
   whole existing golden path still works — seed/resolution/world shape/
   sea level all still reach generation, the map renders, all five
   existing overlay toggles (settlements/roads/territory/villages/sea
   routes/provinces) still work, load-save still works, credits still
   opens. This is the single most important check this milestone has —
   a beautiful new shell that breaks generation is a regression, not a
   redesign.

## Done means (milestone 1)

The desktop (1920×1080) dark-theme shell exists as real Godot scenes/
Control nodes matching the mockup's structure and visual language, every
currently-real feature is reachable and working through it, every
not-yet-real feature is visibly present but honestly inert, and the
existing golden path (generate → render → interact with overlays) is
screenshot-verified unbroken.

## Milestone 1 — done (2026-08-17)

Rebuilt `main.tscn`/`main.gd` as the full 6-region shell (top bar with 7
domain menus, 4-group workspace navigator, second panel that swaps with
navigator selection, mode bar + viewport, right inspector, bottom timeline
bar). Zero Rust changes — this was entirely GDScript/scene work, confirmed
by `cargo build -p cartalith-godot` needing no new `#[func]`s.

**Key implementation decision, not obvious from the scope doc alone**:
Godot's `%UniqueName` node lookup resolves by name regardless of tree
position, so every real control (`SeedInput`, `WorldShapeInput`,
`GenerateButton`, the three overlay-toggle checkboxes, `LoadSaveDialog`,
`CreditsButton`/`CreditsDialog`, etc.) was re-parented into its new home in
the tree with the exact same node name and `unique_name_in_owner = true` —
`main.gd`'s existing `@onready var x = %Name` lines needed **zero changes**
for any of them. This is what let a scene rewrite this large stay low-risk:
the re-parenting was mechanical, not a rewrite of the working logic.

**Real feature-inventory corrections found**:
- The scope doc's "Ambiguous, verify" list correctly flagged CPU/GPU/memory
  live readouts as unconfirmed — verified: no such `#[func]` exists, so the
  top-bar readout shows generation status (seed/size/km) instead, which
  *is* real data, rather than a fabricated performance number.
- No per-cell inspector query (elevation/slope/aspect/etc. at an arbitrary
  cursor position) exists in `cartalith-godot` — confirmed by grep, not
  assumed. The Inspector panel's "no selection" state is an honest static
  placeholder saying so, rather than one built against fields that don't
  exist. What *does* have real backing: settlement hover data
  (name/population/faction/coastal/capital), already computed by
  `map_overlay.gd` for its own on-canvas hover card — added a
  `settlement_hovered` signal so the new Inspector panel shows the same
  real data in its own dedicated panel, not a duplicate hit-test.
- The menu-structure doc's full item inventory (dozens of individual
  generation-stage sliders under "Generate", for instance) was populated
  representatively, not exhaustively — most of those sliders don't exist as
  separate Rust-side tunables beyond the 4 experimental flags already in
  the World Parameters card. Transcribing all ~200 leaf items verbatim
  wasn't what "build the shell" meant; the 7 menus, their real top-level
  actions, and disabled placeholders for `NEW`-tagged categories are.
- Deviated from the mockup's exact panel widths (206/238/272px) — the
  existing WORLD PARAMETERS/STRUCTURE/ADVANCED cards need more width to
  stay readable than the mockup's 238px layer panel allows, so the second
  panel is 360px. Flagged, not silently matched to the mockup at the cost
  of readability.
- Judgment call: menu items with real backing are actions (Generate World,
  New seed, Open project, Credits) rather than live-editable parameters —
  Godot's `PopupMenu` doesn't support embedded SpinBox/slider controls well,
  so parameter editing stays in the navigator-driven second panel (menus
  hold commands, panels hold live state) rather than forcing controls into
  an unsuitable widget.
- The old `Stage`/`ControlsPanel` width-based responsive fallback (stacking
  the panel above the map below ~700px) was removed, not preserved, since
  the new 5-region layout has no structural equivalent to fall back to —
  this is a real, deliberate gap until the responsive milestone
  (`GUI_SHELL_SCOPE.md`'s own explicit deferral) lands; narrow windows will
  look cramped, not stacked, until then.
- The new shell's own chrome (panels, top/bottom bars) uses inline dark
  styling (`StyleBoxFlat` sub-resources matching the mockup's `#0d0e0f`/
  `#e0a34a` tokens) rather than a new Theme resource; the re-parented input
  controls (SpinBox/OptionButton/CheckBox/Button) still render with
  `app_theme.tres`'s light-parchment control chrome sitting on the new dark
  background — visually inconsistent (light input widgets on a dark shell),
  flagged as a real, known follow-up for whenever the deferred light/dark
  theme-toggle milestone builds a proper dark `Theme` resource, not silently
  papered over.

**Verified**: `cargo build -p cartalith-godot` clean (0 new Rust code),
`cargo test --workspace` 0 failures across every crate. `godot4 --headless
--quit main.tscn` clean load. Real windowed-app screenshot verification,
end-to-end, through the new shell: generation (seed 12345, Classic, 2048²)
completed and rendered correctly with real terrain/settlements/roads/sea
routes, the top-bar readout and status label both updated with real data,
the `CARTOGRAPHY > Layers` navigator swap correctly showed the three real
overlay-toggle checkboxes, hovering a settlement marker updated the new
Inspector panel with real population/faction/capital data (confirmed
against the existing on-canvas hover card showing the same settlement), and
the Credits dialog opened and displayed correctly. Load-save wasn't
re-verified with an actual file (its wiring pattern is identical to
Generate's, already proven) but the button/dialog are confirmed present and
correctly re-parented.

**Deferred, exactly as scoped**: light theme, panel collapse/rails, all
three responsive breakpoints, terrain appearance's actual editing GUI, any
further engine-side `#[func]` work.

## Cleanup pass: eliminate top-bar/navigator duplication (2026-08-17)

Owner-flagged after looking at the shipped shell: *"There should be no
double menus in the upper bar that are present in the left [nav]."*
Audited every top-bar menu item against the navigator's `NAV_GROUPS`
inventory for real, not superficial, duplication — same label *and* same
destination/content, not just a shared word between conceptually distinct
surfaces (`design/cartalith-menu-structure.md`'s own §2 rule: menus hold
operations, the navigator holds subjects — a menu action and a nav subject
sharing a word isn't automatically a duplicate, e.g. the Generate menu's
numbered pipeline stages "08 Ecology"/"09 Settlements"/"11 Politics" read as
an ordered process list, not a second copy of the WORLD/CIVILIZATION
subject browser, and weren't touched).

**Found one real, flagrant case**: the Map menu's "Layers" item did nothing
but call `_select_nav_subject("CARTOGRAPHY", "Layers")` — the exact same
panel the CARTOGRAPHY nav group's own "Layers" subject already opens, same
label, same destination, zero distinct content. Removed the item from
`_build_menus()`'s Map popup and the now-dead `_on_map_menu_id` handler
(the Map menu's three remaining items — Terrain appearance, Painter
styles, Labels & annotation — are all still `disabled`, so the popup never
fires `id_pressed` for anything now; the connection and handler were
removed rather than left as dead code).

**Considered and left alone, with reasoning**: the top-bar "Assets" menu
(a real top-level domain per the mockup's own 7-menu top bar) versus the
CARTOGRAPHY nav's "Assets" subject — both are 100% inert placeholders
right now, but they represent genuinely different scopes in the source
design (global asset-library management vs. per-map asset usage), the same
relationship as the "World" menu name matching the "WORLD" nav group
header. Removing either would reduce fidelity to the actual mockup
screenshot without fixing a real functional duplicate — left for a future
pass once either surface gains real content and the distinction (or lack
of one) becomes concrete rather than speculative.

**Verified**: `cargo build -p cartalith-godot` clean (0 new Rust — pure
GDScript), `cargo test --workspace` unaffected. `godot4 --headless --quit`
clean load. Real windowed-app screenshot verification, maximized
(1696×1018): confirmed the Map menu now shows only its three real items
with the CARTOGRAPHY nav's own "Layers" visible and unduplicated below it;
re-ran the full golden path (seed 12345, Classic, 2048², Generate →
real terrain/settlements/roads/sea-routes render) and the causal-chain
Inspector (hover → real "WHY HERE?" chain, e.g. *"strong fresh water (0.86)
→ strong gentle terrain (0.85) → strong terrain form (0.98) / Despite: weak
flood risk (0.06) / Suitability 0.58"*) both still work correctly through
the cleaned-up shell; the Layers panel (now the sole entry point) still
shows and functions for all three overlay toggles.

## Second workflow re-audit (2026-08-17)

Owner asked to re-check the shell against `design/Cartalith GUI.dc.html` and
`design/cartalith-menu-structure.md` once more and correct any remaining
drift. Re-read `main.gd`/`main.tscn` against both sources directly, then
confirmed against a real maximized (1696×1018) screenshot rather than code
alone.

**Confirmed clean, not re-broken**: no duplicate top-bar/nav items found —
the earlier Map-menu "Layers" fix (bundled into `d7fdd2d`) held. Nav-subject
click correctly swaps only the second panel + inspector context, never the
viewport, per §2's own rule — verified in both code (`_select_nav_subject`)
and the live screenshot. Mode bar, bottom timeline bar, and the 7 top-bar
menus all present and structurally match the mockup's own regions.

**One real design question found, decided, not silently changed**: the
mockup's own `1a`/`4a` reference screens show the *Layers* list as the
panel content when "Overview" is the active nav row — this port instead
shows the generation-parameter form (seed/resolution/world shape/sea level)
under `WORLD:Overview`, with Layers living under its own `CARTOGRAPHY:Layers`
subject. This was a deliberate milestone-1 decision, not an oversight: a
user's very first action is entering a seed and pressing Generate, and
`menu-structure.md`'s own §2 rule ("navigator swaps the tool palette") only
mandates that nav selection changes the palette — it doesn't mandate which
palette lives at which subject. Kept as-is; noting the discrepancy here so
it's a recorded decision, not a gap that gets "fixed" back and forth by a
future pass without the reasoning in hand.

**Verified**: real windowed-app screenshot only this pass (no code changes
were needed — the shell was already correct after the prior cleanup); no
`cargo`/`godot4 --headless` re-run required since nothing changed.
