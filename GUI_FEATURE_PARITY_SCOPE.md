# GUI feature parity: real audit and milestone plan

> **This document defines the parity audit's four categories and the milestone
> plan built from them. It does not track them.** Where any of that work stands
> is in **`cartalith-native/docs/STATUS.md`**, the only place progress is
> recorded. Everything below is a **2026-08-18 audit** — read it for what was
> found, classified, researched and decided, never for what is built.
>
> **Read this box before the audit below.** This document was written
> against the *panel-browser* shell (navigator + swapping subject panel).
> That shell no longer exists: the DCC shell replaced it structurally
> (`DCC_SHELL_SCOPE.md`, `UI_SHELL_DESIGN.md` — eight menus, five workspace
> tabs, a left tool rail, and a right dock holding only Layers/Properties/
> Sample). The audit's findings about **what the Rust engine really has**
> were checked against `lib.rs`, not against a layout, so they outlived the
> shell they were written for. Its findings about **where a control should
> live** did not: dock-panel recommendations do not map onto a shell whose
> governing rule is *menu items open dialogs, never persistent side panels*.
> Every placement below was re-stated against `UI_SHELL_DESIGN.md` on
> 2026-08-18, and those DCC homes are the placement decisions this document
> carries.
>
> **One re-classification, recorded because it changes what a row *is*.**
> Category 1 item 5 (faction culture-terrain-fit) was catalogued as
> wiring-only and is not one: `civ_culture_terrain_fit` takes a per-faction
> `terrain_mix` and a `world_mean_terrain` that nothing computed, so it needs
> the `_civFactionAggregates` aggregation first — a `cartalith-civ` milestone
> (Phase 2 milestone 20, `ECONOMY_SCOPE.md`), not a wiring job. **Item 5
> belongs to Category 2**; the full reasoning is under Category 1 below.

Owner directive, verbatim: *"work through all phases until feature parity
with the original project has been established... all options, sliders etc
[should be] available and... the GUI from claude design [should be]
respected... without old design still sticking. also all menu's in the new
design should be functional for what the name implies, if youre not sure
search for relevant information online and other similar applications."*

Three real requirements, not one vague one: (1) every reference control needs
a real Godot equivalent, not a representative subset; (2) the dark shell must
be visually consistent, no leftover light-parchment chrome; (3) every menu
item and navigator subject must genuinely do what its name implies, or be
explicitly, honestly justified as deferred/out-of-scope — researched against
comparable real tools where the reference itself is thin or absent
(`GUI_SHELL_SCOPE.md`'s own milestone-1 note: the shell's ~200 reference leaf
items were populated *representatively*, not exhaustively — this document is
the exhaustive pass that follow-up was always going to need).

This document does not implement anything. It is the itemized audit plus a
milestone plan, in the same format `JOURNEY_PLANNER_SCOPE.md`/
`ASSET_LIBRARY_SCOPE.md` used for their own investigations.

## Method

Read, not assumed: `GUI_SHELL_SCOPE.md` in full (every pass to date, including
the Fable-5 decluttering pass's two disclosed gaps — Credits dialog grey
panel, navigator one-level compression of the reference's real
Generate/Explore two-level structure); `design/cartalith-menu-structure.md`
(the full `#id`/`NEW` inventory); `cartalith-native/godot-project/main.gd`,
`main.tscn`, `map_overlay.gd`, `theme/dark_theme.tres` as they exist today
(post-declutter); `cartalith-native/crates/cartalith-godot/src/lib.rs` in
full (the real `#[func]` surface — every accessor `WorldGen` currently
exposes); `cartalith-native/crates/cartalith-engine/src/lib.rs`'s
`WorldParams`/`PlanetParams` structs directly, not assumed from a doc summary;
`MVP_SCOPE.md`, `PHASE2_SCOPE.md`, `JOURNEY_PLANNER_SCOPE.md`,
`ECONOMY_SCOPE.md`, `TERRAIN_APPEARANCE_SCOPE.md`, `ASSET_LIBRARY_SCOPE.md`,
`VISION.md`. Web research into QGIS, Mapbox Studio, World Machine/Gaea, and
Blender/Photoshop panel conventions for the category-3 items below, cited
where used.

**Ground rule for classification**: "real backing" means a working
`#[func]`/Rust function exists and produces correct, golden-verified (or
real-unit-tested) output *today* — checked in `lib.rs`/the relevant crate,
not inferred from a scope doc's milestone title.

## Ground truth as the audit found it, 2026-08-18 (summary, not restated in full)

*This is the snapshot the categories below were derived from — the engine and
shell as they read on 2026-08-18, kept unedited so the audit's own reasoning
stays checkable. It is not an inventory of the engine now.*

Confirmed real and GUI-wired: generation (seed/resolution/width/sea
level/world-shape archetypes/4 experimental flags/villages toggle),
load-save, credits, settlement+road+sea-route rendering with hover inspector
and causal "why here?" chain, territory fill, province boundaries, the
then-new dark `Theme` resource.

Confirmed real in Rust but **not GUI-wired at all** (the highest-value
findings of this audit — see Category 1): asset-pack loading
(`load_asset_pack`/`has_asset_pack`), province metadata
(`get_provinces`), per-settlement trade balance (`get_trade_balances`),
faction culture-terrain-fit (`civ_culture_terrain_fit`, ported but not even
given a `#[func]` yet), the three `PlanetParams` fields (`g`,
`rotation_hours`, `axial_tilt_deg` — real, live in climate, no setter
exists), the `use_gpu` flag (`WorldParams::use_gpu`, `GPU_LAYER_INTEGRATION_
SCOPE.md` milestone 6, real, never set to `true` by `WorldGen`), and raw
World-Structure knobs (`WorldStructureParams`'s five floats — `WorldGen`
only exposes them through five hardcoded named-archetype presets, never as
live sliders, even though the underlying struct already takes arbitrary
values).

Confirmed **not real anywhere** in this engine (no Rust function exists, not
even unwired): year-by-year playback, Warfare, full faction/territory
economic aggregation (`_civFactionAggregates`'s 165-line piece —
`civ_resource_trade_balance`'s settlement-level piece *is* real, the
faction-level one is not), Journey Planner orchestration (2 of 6 milestones
landed, primitives only, `JOURNEY_PLANNER_SCOPE.md`), Asset Library UI
(milestone 7, renderer integration, in progress by a concurrent fork right
now), terrain appearance's editable GUI (4 of ~15 research-doc phases landed,
all CPU-only render changes, zero GUI), tile/LOD pan-zoom, 2D/3D toggle,
heightmap import, GeoJSON export, brush/stamp tools, staleness tracking,
Narrative/Scenario (`VISION.md`: "a different product, not yet scoped
anywhere").

## Category 1 — real backing, needs wiring only

The highest-value, lowest-risk work in this whole document: every row below
already has correct, tested Rust behind it. "Wiring" ranges from a pure
GDScript change (a button, a `FileDialog`, a table) to a one-line `#[func]`
that mirrors an existing pattern (`set_sea_level`, `set_villages_enabled`).

**Where each row belongs in the DCC shell** — the placement decisions this
category carries, re-pointed 2026-08-18 off the panel-browser homes named by
the original analysis, which follows unedited underneath because its reasoning
about the engine is what made each row cheap in the first place:

| # | Control | Its DCC home |
|---|---|---|
| 1 | Import asset pack | `File ▸ Import asset pack…` |
| 2 | Settlements table | `Simulate ▸ Statistics…`, Settlements tab — sortable, filterable, row click pins the causal chain in Properties |
| 3 | Trade balance / Economy | `Simulate ▸ Economy…`, Economy tab |
| 4 | Province list | `Simulate ▸ Statistics…`, Provinces tab (boundaries render from the Layers dock, which is a separate surface) |
| 5 | Faction culture-terrain-fit | Belongs with `_civFactionAggregates` in Category 2, for the reason below — not a placement question until that exists |
| 6 | Planet gravity / rotation / tilt | `Generate ▸ Climate…`, PLANET section |
| 7 | GPU status / toggle | `View ▸ Performance readout…` — one line per stage, GPU or CPU. **This audit split the row: build the readout, defer the switch**, because `DECISIONS.md` §7c makes the GPU path produce a different world for the same seed, so a switch beside the readout offers a choice the parity contract does not allow. Per the honesty rule the deferred half is present and disabled with its reason on it, never omitted |
| 8 | World Structure raw sliders | `Generate ▸ Tectonics…`, WORLD STRUCTURE section (plus `apply_archetype()`, so a preset writes those same five sliders) |
| 9 | Layer granularity | Layers dock — Settlements / Roads & ways / Sea routes as three toggles |
| 10 | Click-to-pin selection | Properties dock |

**Item 5, the one correction to this document's own classification.** `civ_culture_terrain_fit`'s signature is
`(culture_key, terrain_mix, world_mean_terrain)`. When this sweep ran,
neither map existed anywhere in this workspace — computing them *is* the
territory-aggregation piece `ECONOMY_SCOPE.md` listed as unstarted and this
document lists under Category 2 medium (`_civFactionAggregates`). So it was
not a `#[func]` away; adding one would only have produced a function with no
argument to call it with. This row moved to Category 2 and folded into that
milestone — its dependency, and the order the two have to be built in, is the
finding.

`_civFactionAggregates` is Phase 2 milestone 20 (`ECONOMY_SCOPE.md`), and it
is the right owner of the two maps: `FactionAggregate::terrain_mix` and
`FactionAggregates::world_mean_terrain` drop straight into
`civ_culture_terrain_fit`'s existing signature, so once that milestone exists
item 5 is a `#[func]` and a panel and nothing more.

| # | Control | Real backing | What's missing |
|---|---|---|---|
| 1 | **Import asset pack…** (`Project` menu, currently `disabled`) | `WorldGen::load_asset_pack(path) -> bool`, `has_asset_pack() -> bool` — both real, both used today only by a hardcoded `TEMP milestone-7` debug call in `main.gd` line 509 | A `FileDialog` (same pattern as `LoadSaveDialog`) + enabling the menu item + a status readout using `has_asset_pack()`. Near-zero engine risk — the function already runs correctly every app launch via the temp call. |
| 2 | **Settlements table** (`CIVILIZATION:Settlements`) | `get_settlements()` returns full per-settlement data today | Only a hover card + Inspector panel exist; no persistent, sortable/searchable table (the hint text itself says "a dedicated searchable/sortable table here is not yet built") |
| 3 | **Trade balance / Economy panel** (`CIVILIZATION:Economy`) | `get_trade_balances()` — real, per-settlement export/import lists, computed every generate, zero GUI consumer anywhere | An Economy panel listing exports/imports per settlement (or aggregated) |
| 4 | **Province list** (`CIVILIZATION` — no current subject owns this) | `get_provinces()` — real, id/faction/name/capital-settlement-index per province, computed every generate | No UI reads it at all; province *boundaries* render (checkbox), but no province *identity* (name, owning faction) is ever shown |
| 5 | **Faction culture-terrain-fit** | `cartalith_civ::civ_culture_terrain_fit` — ported and real-unit-tested, with no `#[func]` over it | A `#[func]` plus a panel — **but not until its two inputs exist**. `terrain_mix` and `world_mean_terrain` come from `_civFactionAggregates` (Phase 2 m20, `ECONOMY_SCOPE.md`), which is why this row is Category 2 and not Category 1 |
| 6 | **Planet parameters** (gravity, day length, axial tilt — `World` menu's "Planet" section) | `PlanetParams { g, rotation_hours, axial_tilt_deg }` — real fields, `axial_tilt_deg` confirmed live in climate (`compute_temperature`/`simulate_weather` call sites) | No `WorldGen` setter exists at all — `WorldParams::defaults` hardcodes `g:1.0, rotation:24h, tilt:23.4°` for every generate. A `set_planet_params(g, rotation_hours, axial_tilt_deg)` mirroring `set_sea_level`'s own shape is close to the entire fix. |
| 7 | **GPU acceleration status/toggle** (`World` menu's "Source & resolution" section) | `WorldParams::use_gpu` — real, `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6, per-stage CPU fallback already implemented, `WorldState.gpu_stages_used` already records which path ran | `WorldGen` never sets this to `true`, and nothing surfaces `gpu_stages_used` to the GUI. A checkbox + a readout line, no new engine work. |
| 8 | **World Structure raw sliders** (`World` menu's "World structure" section: continentality/fragmentation/tectonic energy/ocean depth/hotspot density) | `WorldStructureParams` already takes five raw `f64`s — `generate_world_structure()`'s `#[func]` hardcodes exactly five named presets and has no path for caller-supplied raw values | A `generate_world_structure_custom(seed, width_km, resolution, continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density)` `#[func]` (or an optional-override variant of the existing one) plus five sliders. The reference's own presets are almost certainly *also* just named points on these same five sliders — worth confirming against `reference/Cartalith Gen1 v2.10.html`'s own World-Structure panel before building, but the Rust-side gap is exactly this narrow. |
| 9 | **Layer granularity** (`LayersPanel`) | `get_settlements()`/`get_roads()`/`get_sea_routes()` are already three separate queries; `map_overlay.gd`'s single `CivLayerCheck` shows/hides all three together via one Control's visibility | Splitting into three checkboxes (Settlements / Roads / Sea routes) needs a `visible` flag per draw-category inside `map_overlay.gd`'s own `_draw()`, not new Rust |
| 10 | **Selection "pin"** (Inspector panel) | `explain_settlement(index)` already returns the full causal chain; today it only fires on hover (`_on_settlement_hovered`), never on click | A click-to-lock state in `map_overlay.gd`/`main.gd` so the Inspector holds a settlement's data after the mouse moves away — pure GDScript, `explain_settlement` needs no change |

**Not a gap, listed for completeness**: resolution presets (512/1K/2K/4K/8K),
sea level, the four experimental flags, villages toggle, world-shape
archetype selection, load/save, credits, territory/province-boundary
rendering — all already correctly wired.

## Category 2 — reference has it, this port doesn't yet

Ordered by real size, using each subsystem's own existing scope document
where one already exists rather than re-deriving size estimates.

**Re-baselined 2026-08-18.** Two classification changes and one placement
pass:

- **Faction culture-terrain-fit joins this category** (Category-1 item 5,
  above) and belongs with `_civFactionAggregates` below, which is the thing
  that has to exist first.
- **CPU/GPU/memory live readout leaves the small list** — its home is
  `View ▸ Performance readout…`, covering all three numbers (GPU from
  `get_gpu_stages_used()`, CPU/memory from Godot's own singletons, exactly as
  this section predicted), so it is a Category-1 row rather than a Category-2
  one.
- **The Journey Planner's entry under *Large* below is scoped as GUI-only**,
  since `JOURNEY_PLANNER_SCOPE.md` owns its engine side in its own right. Its
  DCC home is `Simulate ▸ Logistics`.

DCC homes for the rest of this category, replacing the panel-browser
placements the individual entries name: heightmap import →
`File ▸ Import heightmap…` (present, disabled); GeoJSON export →
`File ▸ Export GeoJSON` (present, disabled); route-corridor/travel-cost
analysis fields → `View ▸ Analysis field overlay`; terrain appearance GUI →
`Render ▸ Terrain appearance…`; faction roster → `Simulate ▸ Statistics…`
as a fourth tab beside Settlements/Provinces/Economy, since that dialog is
now where civilisation-layer rosters live; labels/annotation and manual icon
placement → the `Label`/`Icon stamp` tools on the left rail plus
`Render`/`Assets`, not a menu of their own.

### Small (a few functions to one milestone each)

- **Heightmap import / infer tectonics from heightmap** (`Project > Import`) —
  no `cartalith-io` reader for this exists; scope unconfirmed (not
  investigated by any existing scope doc). Likely small-to-medium: an image
  decode plus seeding the tectonic substrate from it rather than from noise.
  Needs its own short investigation before a milestone estimate is trustworthy
  — flagged rather than guessed.
- **GeoJSON export** (`Project > Export`) — vector output of the already-real
  road/settlement/territory data; no Rust writer exists. Small, bounded by
  what's already computed (nothing new to derive, only to serialize).
- **CPU/GPU/memory live readout** (`Top bar`, `View > Debug & performance`) —
  `GUI_SHELL_SCOPE.md`'s own "ambiguous, verify" list already confirmed no
  `#[func]` exists; Godot's own `OS`/`Performance` singleton supplies CPU/
  memory readouts natively without any Rust work — only GPU status has real
  Rust backing (`WorldState.gpu_stages_used`, once Category 1 item 7 lands).
  Small, GDScript-only for two of the three numbers.
- **Route corridors as a selectable analysis field** — `build_route_corridors`
  already runs inside `compute_civilisation`; exposing it as one of the
  `View > Analysis field` options needs a new texture-builder `#[func]`
  mirroring `build_territory_texture`'s own shape. Same for `travel_cost`
  (already computed, already surfaced per-settlement via `explain_settlement`,
  never as a full-field texture).

### Medium

- **Faction roster / Politics beyond territory** (`CIVILIZATION:Factions`,
  `Generate` menu's Politics stage) — settlements already carry a faction id;
  no faction *entity* (name, roster, add/remove) exists. `_civFactionCulture`/
  `Government`/`Religion`/`Ag-technology` are confirmed **UI-only categorical
  labels in the reference itself, with zero derived computation**
  (`PHASE2_SCOPE.md` milestone 18) — so a faction roster's "flavor" fields are
  cheap to port (arbitrary labels, not simulation), while the roster
  *mechanics* (add/remove, persistent identity across a session) are new
  Rust-side state that doesn't exist yet.
- **`_civFactionAggregates`** — owned by Phase 2 milestone 20
  (`ECONOMY_SCOPE.md`), not by this document: population, tax, the five-axis
  "power" heuristic, sector output and territory-fit. **The open question this
  row raised was decided there, and the decision is worth carrying**: port the
  heuristic "power" composite **verbatim** rather than simplifying it, because
  the reference labels it honestly as derived/heuristic and a simplification
  would be a different heuristic with nothing to check it against. What this
  document keeps is the GUI half — a faction roster, whose *mechanics*
  (add/remove, persistent identity across a session) are new Rust-side state
  in their own right, not a consequence of the aggregation.
- **Terrain appearance GUI** (`Map > Terrain appearance`, all of §5b in
  `design/cartalith-menu-structure.md`) — `TERRAIN_APPEARANCE_SCOPE.md`
  milestones 1-4 built real, tested, CPU-only rendering improvements (relief
  lighting, hydrology tint, the atlas look) with **zero GUI** on top of any
  of it. Every slider in §5b (ramp editor, colour/material/relief/detail/
  atmosphere sections) is real render-time data (`TerrainAppearance` struct
  fields already exist for most of it) with no control surface. This is a
  genuinely medium-to-large GUI-only milestone sitting on a foundation
  somebody else owns — lower engine risk than most Category 2 items, because
  the rendering it dresses belongs to `TERRAIN_APPEARANCE_SCOPE.md`'s
  milestones and the engine risk goes with it. What is left here is a control
  surface.
- **Labels & annotation** (`Map`) — region-name labels: no placement/
  rendering exists anywhere in this port. Reference feature, unclear exact
  size without reading the reference's label-placement code directly (not
  done in this pass — flagged, not estimated).
- **Manual icon placement / Paint brush** (`Cartography:Icons`, `:Paint`) —
  the underlying rule-driven scatter *engine* is real
  (`ASSET_LIBRARY_SCOPE.md` milestones 3-4, `place_map_icons_ruled` etc.,
  fully ported and golden-tested in `cartalith-assets`), but the *manual*
  brush interaction (click-to-place, density brush, splat painting) is a
  distinct UI+state-tracking feature never scoped — needs its own
  investigation, sequenced after Asset Library milestone 7.

### Large (own scope documents already exist; restated, not re-derived)

- **Journey Planner** (`Simulate > Logistics`, Inspector's Route context) —
  `JOURNEY_PLANNER_SCOPE.md`: ~70 real functions across 6 milestones, which
  that document defines and this one does not re-derive. Milestone 5
  (route/stage derivation) is flagged there as "almost certainly the largest
  single milestone in this whole plan." Explicitly **not** for wiring into
  automatic per-settlement computation — it's a real, interactive,
  user-driven tool, so its GUI is itself a genuine milestone once the engine
  side is done.
- **Asset Library** (`Assets` mode, `Cartography:Icons`/`:Paint`'s pack
  gallery) — `ASSET_LIBRARY_SCOPE.md`: ~2,250 reference lines across 7
  milestones, six of them pure logic (manifest, zip, scatter rules, placement,
  library model, image handling) and milestone 7 the renderer + Godot
  integration. The boundary that matters to this document: the Asset Library
  *page UI itself* (browser, inspector, sprite-sheet slicer) is explicitly out
  of scope for milestone 7 and is a further GUI milestone of its own.
- **Simulate: year-by-year playback** (bottom-bar transport controls, all of
  `Simulate > Time`) — not a GUI gap at all. The engine is a one-shot static
  generator by explicit, repeated owner decision (`HARDWARE_ACCELERATION.md`'s
  static-generation correction; "no need to continuously calculate"). Any
  GUI work here would be building controls for a system that doesn't exist
  and isn't planned to — see "Out of scope" below, not a milestone.
- **Tile/LOD pan-zoom viewport** (`View > LOD`, corner tile/LOD readout) —
  `LOD_TILING_BASE_SCOPE.md`: `cartalith-spatial` exists as a real,
  standalone, unintegrated crate. Integrating it into the viewport is real,
  substantial work with its own already-written scope document this
  document doesn't re-derive.

## Category 3 — mockup inventions with no reference equivalent (`NEW`-tagged)

Per-item recommendation, research-informed where the reference is silent.

**Re-baselined 2026-08-18.** These recommendations were written against the
old shell's `LayersPanel` and navigator; the reasoning survives, the homes
change. Per-layer opacity now belongs to the DCC **Layers dock** (which
already holds the five real toggles) rather than a panel; the measurement
tool is already a real rail slot (`Measure`, `M`) waiting for a
pass-buffer, so it depends on `UNIFIED_TOOL_PLAN.md`'s tool system, not on a
place to put it; quality tiers belong under `Render ▸ Render quality`;
coordinate system / projection has no DCC home at all and its "defer"
recommendation is unchanged. **Stale-field tracking stops being this
document's recommendation and becomes another document's milestone** —
`UNIFIED_TOOL_PLAN.md` milestone A defines the `PassBuffer`/`StageGraph`
staleness core, so what is left here is surfacing it. The interim affordance
is a decision, not a placeholder: a regenerate-to-apply footer plus a
status-bar note on the Generate stage dialogs, rather than a per-stage stale
pip that would advertise the incremental pipeline a one-shot generator does
not have.

**Worth building — natural fit for a cartographic/terrain tool, cite real precedent:**

- **Per-layer opacity + blend mode + reorder** (`LayersPanel`) — QGIS's own
  Layer Styling panel puts opacity and 13 named blend modes (Normal, Multiply,
  Screen, Overlay, Darken, Lighten, etc.) directly in the layer stack, plus a
  separate "control rendering order" panel independent of the layers list.
  Mapbox Studio/GL style spec treats opacity as a first-class paint property
  on every layer type. This is exactly the vocabulary a cartographic tool's
  users already expect; the risk is that Cartalith's actual layers
  (territory/province-boundary/settlements+roads) are baked per-pixel in
  `render.rs`'s single pass rather than independently compositable Godot
  nodes today — so "reorder"/"blend mode" would need those three to become
  independently rendered layers first (a real, moderate architecture change,
  not just a GUI addition). **Recommendation: build opacity now (cheap, the
  three overlay textures already support alpha), defer reorder/blend-mode
  until/unless the render architecture actually separates the layers.**
- **Measurement tool** (`Map > Labels & annotation`) — a standard feature in
  every comparable tool (QGIS's measure-line/-area, World Machine's ruler).
  Cheap to build once any coordinate/scale readout exists (the scale bar
  already does, in km). **Recommendation: build**, small effort, real value
  for a cartographic tool.
- **Coordinate system / projection** (`World > Scale & calibration`) — every
  cartographic tool (QGIS foremost) treats this as core, not optional.
  **But**: Cartalith's world is a flat, non-georeferenced procedural grid
  with no real-world CRS to project from or to — there is nothing to
  *convert between*. **Recommendation: defer.** This is a mockup import from
  a genuinely different kind of tool (real-world GIS) grafted onto a
  procedural-world generator; building it would be decorative, not
  functional, until/unless Cartalith gains multiple coordinate
  representations of the same world (e.g. a lat/long overlay is closer to
  what's actually meaningful here, and that's a labels feature, not a
  projection one).
- **Quality tiers (Performance/Balanced/Quality/Ultra)** — `TERRAIN_
  APPEARANCE_SCOPE.md`'s own research doc already names this (§29) as real
  future scope once a GPU rendering path and multiple detail levels exist.
  World Machine and Gaea both gate expensive erosion/simulation nodes behind
  similar tiers. **Recommendation: build once the terrain-appearance GUI
  (Category 2) exists at all — premature before that.**
- **Stale-field tracking** (mentioned explicitly by
  `design/cartalith-menu-structure.md` itself as "the one genuinely new
  system worth building early") — every node-graph terrain tool (World
  Machine, Gaea) recomputes everything downstream of an edited node
  automatically; the equivalent here (mark hydrology→climate→biomes→
  settlements→infrastructure stale when an upstream stage changes) is the
  same idea applied to a linear pipeline instead of a graph. Real, valuable,
  and — per the menu-structure doc's own reasoning — worth prioritizing
  *before* exposing individual pipeline-stage editing (Category 2's
  Generate-stage sliders), since editing a stage without it "silently
  produc[es] an inconsistent world." **Recommendation: build as the first
  real prerequisite for any Category-2 Generate-stage slider work**, not a
  standalone nice-to-have.

**Worth deferring — real but premature, not because it's a bad idea:**

- **Preview: Compare current/previous/split/before-after** (`Terrain
  appearance`) — genuinely useful (Blender/Photoshop-style A/B preview is
  standard), but has no meaning until the terrain-appearance GUI itself
  exists. Defer with it, not separately.
- **Panel collapse / rails, responsive breakpoints** — already explicitly
  deferred by `GUI_SHELL_SCOPE.md` itself, restated here for completeness,
  not re-litigated.
- **Light theme toggle** — same; the toggle button exists and is `disabled`
  by design.
- **Project settings…, Recent worlds, Save project (as a File-menu concept
  distinct from Export .zip)** — the decluttering pass already found these
  have zero reference grounding (world creation is the onboarding gate, not
  a menu action; "save" *is* Export .zip) and deleted the fabricated
  originals. No new reasoning changes that; still correctly absent.

**Should stay deliberately deferred/removed — doesn't fit this project's scope:**

- **Warfare** (`Simulate` layer toggle, `VISION.md`'s render) — mentioned
  nowhere in the reference or this port's own history except as a mockup
  label. `VISION.md` already flags this as requiring an explicit product
  decision. No comparable "terrain generator" tool (World Machine, Gaea,
  QGIS) has a warfare system — it belongs to a different product category
  (grand-strategy/4X games), and building UI for it here would be exactly
  the "invented structure" the decluttering pass already spent real effort
  removing elsewhere. **Recommendation: leave disabled, don't scope further
  without an explicit owner decision.**
- **Narrative/Scenario domain** (Events, Characters, Conflicts, Objectives,
  multi-track timeline) — `VISION.md`'s own conclusion stands unchanged by
  this audit: "a different product, not yet scoped anywhere... closer to a
  scenario editor than a generator." Not touched here.
- **Year-by-year historical playback** as a *simulation* (not the UI shell
  for it, which already exists and is honestly inert) — same static-vs-
  temporal product question `VISION.md` raises. Not a GUI gap to close.

## Category 4 — visual consistency gaps beyond the Fable-5 pass

The Fable-5 verifier's own disclosed issues (Credits dialog panel using
Godot's default grey; the navigator's one-level compression of the
reference's real Generate/Explore two-level structure) are carried forward
unchanged — this audit found no new information about either and did not
re-investigate them. Found this pass, by reading `theme/dark_theme.tres` directly against every control
type Godot actually themes, not by screenshot alone (screenshots don't show
what a popup *would* look like if never opened during verification):

- **`PopupMenu` has no theme entry at all.** Every one of the six real
  top-bar `MenuButton` dropdowns (Project/World/Generate/Simulate/Map/View)
  renders its popup with Godot's engine-default styling, not this shell's
  dark tokens — confirmed by reading `dark_theme.tres`'s full `[resource]`
  section, which defines `Button`/`CheckBox`/`OptionButton`/`LineEdit`/
  `SpinBox`/`HSlider`/`FoldableContainer`/etc. but no `PopupMenu` type at
  all. This is not a rare path — every menu the shell has is a `PopupMenu`
  under the hood, so this fires on the single most common interaction the
  top bar offers, not an edge case.
- **`TooltipPanel`/`TooltipLabel` are unstyled.** Three real controls
  (`AssetsMenu`, `ThemeToggleButton`, `CreditsButton`) carry `tooltip_text`;
  none of their tooltips resolve through the dark theme, so hovering any of
  them shows Godot's default tooltip chrome instead.
- **`ScrollBar`/`VScrollBar`/`HScrollBar` are unstyled** — `NavigatorScroll`,
  `OverviewContent`, `CreditsScroll` are all `ScrollContainer`s that can show
  a scrollbar (Overview's Advanced-features fold pushes content past the
  panel height at smaller resolutions); no theme entry exists for the
  scrollbar grabber, so it falls back to default engine styling when it
  appears.
- These three are all genuinely easy fixes (a `PopupMenu` StyleBoxFlat plus
  `TooltipPanel`/`TooltipLabel`/`ScrollBar` type entries in the same
  `dark_theme.tres`, same pattern as everything already there) — grouped
  with the Credits dialog panel fix as one small visual-consistency
  milestone, not spread across unrelated future work.

## Milestone breakdown

Ordered by value/risk: Category 1 first (near-zero new engine work, real
function today), then Category 2 by size (small → medium → large, citing
existing scope docs rather than re-scoping them), then Category 3's
build-recommended items, then Category 4 (small, standalone, can run
anytime — not blocking, not blocked by, anything else here).

1. **Category 1 sweep** — the ten rows of the outcome table above. The
   prediction this ordering rests on: it needs zero new Rust. One row (5)
   turned out to be Category 2 and moved there, which is the one way this
   step can surprise whoever runs it.
2. **Category 4 visual-consistency sweep** — one small milestone,
   independent of everything else, can run in parallel with 1.
3. **Stale-field tracking** (Category 3) — build this before any Category-2
   Generate-stage slider work, per the menu-structure doc's own reasoning.
4. **Category 2 small items** — heightmap import (needs its own short
   investigation first), GeoJSON export, route-corridor/travel-cost analysis
   fields. CPU/memory readout is **not** here: it is a Category-1 row, per
   the re-baseline note above.
5. **Terrain appearance GUI** (Category 2, medium) — the best risk/reward
   ratio in Category 2: the hard rendering work is four already-golden-tested
   milestones in `TERRAIN_APPEARANCE_SCOPE.md`, so this is GUI-only on a
   foundation somebody else proved.
6. **Faction roster** (Category 2, medium) — sequenced after
   `_civFactionAggregates` (Phase 2 m20, `ECONOMY_SCOPE.md`), which is its
   prerequisite rather than part of it. Roster "flavor" fields are cheap
   (confirmed UI-only in the reference itself); roster mechanics are the real
   Rust-side work in this step.
7. **Category 3 build-recommended remainder** — layer opacity, measurement
   tool, quality tiers (which 5 has to exist to gate).
8. **Large Category 2 items** — Journey Planner at `Simulate ▸ Logistics`,
   scoped as a GUI milestone over an engine `JOURNEY_PLANNER_SCOPE.md` owns;
   the Asset Library page UI, likewise over `ASSET_LIBRARY_SCOPE.md`;
   tile/LOD viewport (`LOD_TILING_BASE_SCOPE.md`). Simulate playback UI is
   **not** in this list (see Out of scope).

## Out of scope, and why

- **Warfare, Narrative/Scenario, year-by-year historical simulation** —
  `VISION.md` already names all three as requiring an explicit product
  decision the owner hasn't made; building UI for any of them now would be
  inventing structure for a system that may never exist, the exact failure
  mode the Fable-5 decluttering pass already spent real effort correcting
  elsewhere in this same shell. Not touched by this document's milestone
  plan; revisit only after that decision is made.
- **Coordinate system / projection** — no real-world CRS exists for this
  engine's flat procedural grid to convert between; building the control
  would be decorative. See Category 3 above for the fuller reasoning.
- **Wiring Journey Planner or Asset Library into automatic per-world
  computation** — both are real, interactive, user-driven tools in the
  reference, not auto-computed per settlement/world. Their own scope
  documents already say so; restated, not re-litigated.
- **Responsive breakpoints, panel collapse/rails, light theme** — already
  explicitly deferred by `GUI_SHELL_SCOPE.md`; nothing in this audit changes
  that sequencing decision.

## Honest size statement

This is a large, multi-milestone effort — comparable in total scope to the
Journey Planner and Asset Library combined, not a cleanup pass. Roughly 10
Category-1 quick wins (one milestone), ~3 small Category-2 items (one
milestone), 2 medium Category-2 subsystems (terrain appearance GUI, faction/
economy aggregation — one milestone each), 3 large Category-2 subsystems each
already carrying their own multi-milestone scope document (Journey Planner,
Asset Library, LOD/tiling), roughly 10 Category-3 items split across
build-now/build-later/defer/reject, and a small but real Category-4 visual
pass. The single largest remaining piece of "every option and slider
available" is the Generate pipeline's own ~60-80 individual stage sliders
(tectonics/volcanism/erosion/glacial/hydrology/climate/weather/ecology) named
in `design/cartalith-menu-structure.md` §3 — none of which are individually
scoped anywhere yet, each stage's own engine-side parameters mostly already
exist as `WorldParams` sub-struct fields (per this document's own Category-1
findings on `PlanetParams`/`WorldStructureParams`) but have never been
audited stage-by-stage the way this document audited the shell's top-level
structure. That audit is real, additional future work this document
deliberately does not attempt — a `WorldParams`-field-by-field pass against
every stage-01-through-11 slider in the menu-structure doc, which would be
its own investigation comparable in size to this one.
