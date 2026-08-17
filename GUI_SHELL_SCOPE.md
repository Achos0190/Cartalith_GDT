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

**One real design question found, decided, not silently changed** (this
entry originally recorded a decision that a follow-up re-audit,
2026-08-17, found to be a misreading — corrected below rather than
silently overwritten, per this project's own "record the new reasoning"
discipline):

The prior pass's own note claimed the mockup shows Layers as the panel
content *replacing* Overview when Layers is selected. Re-reading turn
`1a` directly (the primary 1920×1080 dark reference every other doc
cites, not a secondary theme variant) shows the opposite: `1a`'s own
one-line description reads *"Strict hairline · **docked layer list** ·
point-sample inspector"*, and its actual markup shows "Overview" active
(amber-highlighted) in the 206px WORKSPACE navigator **at the same time**
a separate 238px LAYERS panel sits immediately beside it, populated with
real entries (Terrain 100, Hillshade 64, Water 100, Rivers 90...). The
Layers panel is a permanent, always-visible third column — "docked" — not
a navigator destination that swaps other content out. This matches every
other turn checked this session (turn 4, the light-theme mirror, shows
the identical simultaneous-panels structure).

**Fixed 2026-08-17**: `LayersContent` (settlements/territory/province
toggles) extracted out of `SecondPanel`'s swappable slot into a new
permanent `LayersPanel` sibling column in `main.tscn` (238px, matching the
mockup exactly), always visible regardless of which navigator subject is
selected. `_select_nav_subject` in `main.gd` no longer touches its
visibility. `NAV_REAL_SUBJECTS` reduced to `["WORLD:Overview"]` only;
clicking `CARTOGRAPHY:Layers` now shows an honest placeholder pointing at
the permanent panel ("Layer visibility is always available in the LAYERS
panel to the right...") rather than the generic "not wired yet" text,
which would have been actively misleading now that layers *are* real.

**Verified**: `cargo build -p cartalith-godot` clean (pure `.tscn`/`.gd`
change, 0 Rust edits), `cargo test --workspace` 0 regressions, `godot4
--headless --quit main.tscn` clean load. Real windowed-app screenshots:
confirmed the Layers panel renders permanently with its own header and all
three real toggles at the default `WORLD:Overview` state, and — the actual
behavioral test — clicking `Terrain` under WORLD correctly swapped the
parameter panel to the honest placeholder while the Layers panel and its
checkboxes stayed exactly in place, unaffected. A full Generate-button
click-through was attempted but blocked by this session's own documented
UI-automation flakiness (synthetic clicks silently dropped when the test
window loses true foreground — also hit by the concurrent terrain-
appearance fork the same day); not a regression from this change, which
never touches generation wiring, and not re-chased given the structural
fix itself was already directly, visually confirmed working.

## GUI decluttering pass: target information architecture (2026-08-17)

Owner-supplied design-lead plan ("Cartalith Godot Shell — Target
Information Architecture", built from real research into the reference
app, the shell as it stood, and the mockup) implemented in full — real
menu/panel restructuring, real control relocation, real dark-theme
restyling, not a token gesture. The plan's own §0 resolution: the reference
app decides *what exists and where it lives*, the mockup decides *how the
shell is built and dressed*. Two real violations of that split, both fixed:

- **`INFRASTRUCTURE`** (Roads/Rivers/Ports/Trade/Logistics) had zero
  grounding in the reference app — replaced with **`EXPLORE`**, the
  reference's real second mode (Tools/Timeline/Info/Journeys/Journey
  Planner).
- **`CARTOGRAPHY:Layers`** and the always-visible `LayersPanel` were two
  surfaces for one thing. The nav subject is gone; `LayersPanel` alone is
  now the single layer surface, freeing CARTOGRAPHY's 5th slot for
  **`Paint`**, the reference's real brush bucket, previously homeless.
  (One drift found from the plan's own assumption: the plan's §3 also
  described consolidating a pre-existing "debug/analysis layer picker (30
  views) + opacity" into `LayersPanel`. Checked, not assumed — no such
  picker has ever existed anywhere in this codebase; `LayersPanel` was
  already the one honest surface, with its own hint explaining why the 30
  debug views aren't separable render layers yet. Nothing to consolidate;
  noted rather than fabricating a picker that would itself have been new,
  unrequested clutter.)

**Real IA implemented** (`main.gd`'s `NAV_GROUPS`):

| Group | Subjects |
|---|---|
| WORLD | Overview (real), Terrain, Water, Climate, Ecology, **Sculpt** (replaces `Resources`, zero reference grounding — Sculpt is the reference's real 4th Generate branch, previously homeless) |
| CIVILIZATION | Settlements (real, redirect), **Factions** (was Population), Economy, **Generation** (was Politics — Step 1 populate → Step 2 roads → Step 3 territories/provinces, the reference's real sequence), **Statistics** (was Culture) |
| CARTOGRAPHY | **Map Style** (was Styling), Labels, **Icons** (was Assets), **Map View** (was Export), **Paint** (new 5th slot) |
| EXPLORE | Tools, Timeline, Info, Journeys, Journey Planner (replaces `INFRASTRUCTURE` wholesale) |

18 of 20 non-Overview subjects now carry a specific, reference-grounded
honest placeholder (`main.gd`'s `NAV_SUBJECT_HINTS`) instead of one generic
"not wired yet" string — naming the real controls that subject corresponds
to in the reference app. `CIVILIZATION:Settlements` is the one exception
with real backing today (settlements render, respond to hover); its hint
redirects to the Inspector rather than claiming a table that doesn't exist,
the same honest-redirect pattern `Layers` established.

**Top bar** (`main.gd`'s `_build_menus`): `ProjectMenu`'s `New world...`/
`Save project` deleted outright (zero reference grounding — reference has
no such File▾ entries; "save" *is* Export .zip). Replaced with a disabled,
honest Import group (Load heightmap / Infer tectonics / Import asset pack)
and Export group (Export .zip / Export GeoJSON). `GenerateMenu`'s fabricated
flat 11-stage pipeline list — the single largest piece of invented
structure in the prior shell — replaced with the reference's real
Civilization Step 1→2→3 sequence plus Generate Provinces. `SimulateMenu`/
`MapMenu`/`ViewMenu` renamed to real reference-grounded items at the same
slot counts (behavior-contract changes only: Map's three items are now a
navigator-jump contract to CARTOGRAPHY subjects, not future modals).
`AssetsMenu` converted from `MenuButton` to a plain `Button` (structural
node-type change) matching the reference's real `#assetsHeaderBtn`
mode-switch mechanism, not a dropdown — stays `disabled`, no Assets-mode
viewport exists yet. A `ThemeToggleButton` was added to the global header
(reference has one, mockup specifies it, it was simply missing) —
`disabled`, since the light-theme milestone itself stays deferred.

**Real bug fixed**: `FooterVBox` (Generate/Load Save/Status) previously had
no visibility logic at all in `_select_nav_subject` — it persisted, visible,
across all 20 nav subjects instead of scoping to `WORLD:Overview` alone.
Fixed with two new `@onready` refs (`footer_vbox`, `footer_separator`) and
one `is_overview` boolean gate.

**Visual consistency** (the plan's §5, the largest real fix): authored
`theme/dark_theme.tres`, a real dark `Theme` resource — surface `#0d0e0f`/
viewport `#101112`, hairline/divider/border alpha-white tokens, text
`#c8cbcd`/emphasis `#e8ebec`/dim `#8d9296`/label `#5f6468`, single accent
`#e0a34a`, matching the exact literal `Color(...)` values already scattered
inline throughout `main.tscn` (verified by comparison, not guessed) —
centralising them so the SpinBox/OptionButton/CheckBox/Button controls that
previously fell back to `app_theme.tres`'s light-parchment chrome now
resolve to one theme with none left. Explicit `disabled` styles defined for
every control type (dark "inert" look, not the light theme's pale-tan
fallback). Assigned as both `Main`'s theme and the project-wide default
(`project.godot`'s `gui/theme/custom`), and directly on `CreditsDialog`
(Window-derived nodes don't inherit the Control-tree theme automatically —
confirmed real: before this, Credits was a fully unstyled default-grey
Godot dialog; after, its text/button chrome resolve through the same dark
tokens). `app_theme.tres` itself is untouched (still a real, working light
theme) but no longer wired into the live path anywhere. `theme_type_
variation = &"SettingsCard"` retired outright: `WorldParamsCard`/
`WorldStructureCard`/`AdvancedCard` (three light-parchment cards sitting on
the dark shell — the single most visible inconsistency in the prior shell)
flattened into plain sectioned `VBoxContainer`s with `HSeparator` dividers,
matching the reference app's own h2-subsection convention. `AdvancedCard`
specifically became a Godot 4.4+ `FoldableContainer` (collapsed by
default, "ADVANCED FEATURES (physical coupling fields)"), matching the
reference's own `<details class="adv">` pattern. `map_overlay.gd`'s
hover-card literals recolored from cream/brown to the same dark surface/
accent/emphasis tokens (was the fourth stray light-styled surface even
though this control's independence from the Theme resource is legitimate —
map content, not chrome).

**One real limitation found, not silently worked around**: `AcceptDialog`'s
own "panel" background isn't reachable the same way its text/button colors
are — assigning the theme to `CreditsDialog` demonstrably re-themed its
`RichTextLabel` text and `Close` button (screenshot-confirmed: white text,
accent-bordered button, versus the fully unstyled grey-on-grey it was
before), but the dialog's background panel stayed Godot's own default
mid-grey rather than the shell's near-black surface token. Left as a
smaller, real, flagged gap rather than chased further this pass — the
"third unstyled look" the plan named is fixed (legible, coordinated colors,
not the old dead default), even if the panel hue isn't a pixel-perfect
match. `LoadSaveDialog` (native `FileDialog`) was deliberately left
untouched per the plan — confirmed by screenshot to already render with
Godot's own neutral dark-grey engine chrome, not the light `app_theme.tres`
skin, so "leave as native OS-chrome" cost nothing.

**Verification, in full**:

1. `cargo build -p cartalith-godot` — clean, 0 new Rust (pure GDScript/
   scene/theme-resource work). `cargo test --workspace` — 0 failures.
2. `godot4 --headless --quit main.tscn` — clean load, re-checked after each
   incremental restructuring step (theme swap → card flattening → menu
   rewrite → nav-group rewrite), not only once at the end.
3. **Real windowed screenshot verification, before/after**: the *before*
   screenshot was captured by genuinely running the old shell (`git stash`
   of every changed file, launch, screenshot, `git stash pop` to restore) —
   not reconstructed from memory — confirming the light-parchment cards on
   the dark shell, the `INFRASTRUCTURE` nav group, and the old menu
   contents as they actually rendered. The *after* sequence covered: the
   default `WORLD:Overview` view (dark chrome throughout, folded Advanced
   section, new nav groups/subjects); `ProjectMenu` and `GenerateMenu`
   popups (content matches the plan's tables exactly); a full Generate run
   (seed 12345, 2048×2048, 800 km, Classic → 40 real settlements rendered,
   status/readout both correct); territory + province-boundary toggles
   (real faction-colour fill and boundary lines drawn); a settlement hover
   (dark-styled on-canvas card *and* the Inspector's causal "WHY HERE?"
   chain — `strong fresh water (0.93) → strong gentle terrain (0.99) →
   weak fertile land (0.34)`, `Despite: weak flood risk (0.06)`,
   `Suitability 0.80` — both still real and correct); the Credits dialog
   (dark, legible, functional); `LoadSaveDialog` (opens, browses the real
   filesystem, Cancel closes it); `CIVILIZATION:Settlements`' honest
   redirect placeholder with `FooterVBox` correctly hidden (the bug fix,
   directly confirmed); `CARTOGRAPHY:Paint`'s new honest placeholder with
   correct accent-highlighted active nav row; the `ViewMenu` popup (content
   matches the plan's table exactly). Every golden-path element the task
   named — seed/resolution/width/sea level/world shape reaching generation,
   all three map-overlay toggles, the causal-chain Inspector, load-save,
   credits — reconfirmed working through the restructured shell, not
   assumed unaffected.

**Files touched**: `cartalith-native/godot-project/main.tscn`, `main.gd`,
`map_overlay.gd`, `project.godot`, new `theme/dark_theme.tres` (light
`theme/app_theme.tres` untouched but retired from the live path), this
file, `docs/CHANGELOG.md`, `docs/STATUS.md`.
