# UNIFIED_TOOL_PLAN.md — the tool system: what a tool is, milestones A–F

**What this is:** the definition of the map-editing tool system — what each
tool does, what it needs from the engine, and the shared draft / commit /
staleness model — with milestones A–F (plus E2) and the findings each build
pass recorded against the reference. **What it is not:** status. Where each
milestone stands is `cartalith-native/docs/STATUS.md`'s "Tool system" group
(UTP-A…UTP-F). The tool-by-tool section is the 2026-08-18 reading the
milestones were sized from; where a build pass refuted it, the correction is
made in place and the reasoning kept in that milestone's "as built" section.

This is the repository-root document, not `docs/UNIFIED_TOOL_PLAN.md` (the
source project's — `CLAUDE.md`'s naming hazard 1). `UI_SHELL_DESIGN.md`:
*"`UNIFIED_TOOL_PLAN.md` decides what a tool *is*; this document decides where
it appears."* It was written as `DCC_SHELL_SCOPE.md`'s milestone 2 — Track 2,
the tool system — investigation and scoping only. Reference line numbers
resolve against `reference/Cartalith Gen1 v2.10.html`.

**Where the tools are armed.** `UI_SHELL_DESIGN.md` as first imported
(`db46908`, 2026-08-17) put fifteen tools on a left rail; the DCC spec's §4.5
Tool palette replaced the rail on 2026-08-19 (`STRANDED_TOOLS.md`), and Ruling
AK (2026-09-23) moved the palette into an always-visible top bar on desktop and
tablet (`DccApp._install_tool_palette_bar`, `DccWidgets.tool_strip`); the phone
keeps the dock's TOOLS block (`DccWidgets.tools_block`). Sculpt is armed by
picking a feature in WORLD ▸ SCULPT (Ruling L). "Milestone F as built" maps
every tool to its binding.

## Headline findings

1. **The reference has a real, shipped Sculpt editor** (`SCULPT_FEATURES`,
   `sculptStamps[]`, `sculptCommit`/`sculptDiscard`; lines 8821–~9470) that
   solves brush falloff, noise-modulated strokes, draft/commit/discard and
   undo granularity for the Terrain group. Prior art to port, not a brush
   model to invent.
2. **Its draft/commit/discard model is the pass buffer's ancestor.**
   `sculptStamps[]` — an uncommitted, session-scoped list of stamp objects — is
   visible immediately, commits by baking the whole stack into `field` in one
   pass followed by **one** `computeFlow(true)` / `refreshClimate()` (never
   per stroke), and discards by dropping the list. The pass buffer is the same
   idea generalised past terrain.
3. **`cartalith-spatial`'s `DirtyTracker` is necessary but not sufficient.**
   Per-tile dirty flag, monotonic version and reason — real bookkeeping — but
   no data: no draft storage, no preview, no discard. The pass buffer is a new
   type beside it, as the reference needed `sculptStamps[]` beside its
   field-level `undoStack`.
4. **Backing by group, as read on 2026-08-18.** Terrain (2) had real prior art
   throughout. Water & ecology (3) splits: River/water rides the Sculpt editor;
   Biome paint is an **override layer** (`paintBiome`), not a mutation of the
   classifier. Civilization (4): Place settlement and Draw route/way have real
   reference precedent as *manual* tools, separate from this port's
   algorithmic placement and roads; Territory paints over this port's own
   `assign_territory`, which the reference never had. Annotation & measure (5)
   is the most mixed: Label and Region select/export have rich precedent, Icon
   stamp partial, Measure **none** — new, and small.
5. **Full-pipeline recompute after every edit is not interactively viable.**
   `CPU_MULTITHREADING_SCOPE.md` measured a **full generation** at 2048²:
   `cartalith-terrain` ~5.1 s parallel, ~7.07 s with the civ per-cell layer —
   before climate, erosion, hydrology and civ's sequential stages. That is not
   what a commit costs (the reference-shaped commit, flow and climate included,
   was later measured at ~123–564 ms CPU, `SCULPT_LIVE_SCOPE.md` §8), but it
   rules out cascading everything per edit. So a commit runs the reference's
   own tail once — flow and climate — and never cascades into settlements,
   roads or territory, which stay stale until asked for.

## The reference's Sculpt editor

`SCULPT_FUNCTION_CHART.md` charts it control by control; this section keeps what
the tool system is built on. It is late (v1.15+), well engineered, not a stub.

**What it does.** Thirteen landform "features" (`SCULPT_FEATURES`, line 8891)
applied along a captured pointer stroke, or a tap for radial features:
Mountains, Hills, Ridge, Plateau, Cliff/Escarpment, Canyon, Valley, River,
Lake, Basin, Coastline, Volcano, and Freehand (eight sub-modes: raise, lower,
smooth, cliff, ridge, canyon, mesa, volcano). Eight one-click presets
(`SCULPT_PRESETS`, line 9005, e.g. "Alps", "Badlands", "Volcanic Isle") seed a
feature's parameters before the stroke.

**The brush model:**

- **Falloff**: `smoothstep(0, 1, (R − dist) / feather)` from the stroke polyline
  (or the radial centre), `feather = max(floor, R · (1 − hardness))`;
  `hardness` (0..1, default 0.5) narrows the band. The mockup's
  `falloff: smooth (gauss)` is this curve. (The port adds three more shapes —
  `sculpt::Falloff`, milestone B.)
- **Intensity** scales coverage into strength (`k = cov · intensity`),
  separately from hardness: shape and strength are independently tunable,
  which is why the mockup has both sliders.
- **Noise-modulated strokes**: three FBM families, `sculptFbm`/`sculptRidged`/
  `sculptBillow`, read `noiseScale`/`octaves`/`persistence`/`lacunarity` from
  one global block. A separate domain-warp **edge noise** (`edgeNoise`,
  per-feature `edgeChar`/`edgeFreqMul`) frays each stamp's *coverage mask* —
  ragged and slow for coastlines and lakes, tight and fast for ridgelines.
- **Radial vs. path**: `feat.radial` (Lake, Volcano) samples distance from the
  stroke centroid; everything else samples signed distance to the polyline
  (`sculptNearestOnStroke`), so strokes can meander (`ctx.meander(amp)`,
  used by River, Canyon and Valley).

**Draft / commit / discard.** Painting never touches `field`. Each finished
stroke becomes a stamp object (`{type, seed, pts, g, f, hidden}`) pushed onto
`sculptStamps[]` — the comment at line 9084: *"nothing here touches `field` or
triggers any recompute"*. `sculptRenderOverlay` redraws each footprint as an
outline and hatch, by its own comment *"a deliberately simpler indicator than a
full live-recolor"* (line 9242); this port replaces it with a real preview
(`SCULPT_LIVE_SCOPE.md`). `sculptCommit()` (line 9317) pushes one undo
snapshot, bakes the whole stack in order into `field` in one pass, then runs
exactly one `enforceRiverChannels()`, one `enforceChannelDescent()` per river
stamp (carving through rises so the river reaches its outlet and **locking** the
carved cells so later erosion cannot refill them — the `carveRiverValleys()`
precedent), one lake→`lakeMask` deposit, one `computeFlow(true)`, one
`refreshClimate()` and one `renderNow()`. `sculptDiscard()` (line 9353) drops
the list after a confirm and touches nothing else.

**Undo is two-tier.** Draft undo/redo (`sculptHistory`/`sculptRedoStack`, JSON
snapshots of the stamp list, `SCULPT_HIST_MAX = 30`) tracks the structural
edits — add, delete, reorder, hide — while drafting; continuous slider
re-tuning does **not** push history (*"a reasonable, common undo granularity"*,
line 9298). The field-level `undoStack` records **one** snapshot, at Commit.

**Water constraints are per feature.** No generic "respect water mask" flag
exists. River and Lake are the two features that *write* water state
(`riverMask`/`riverFloor`/`lakeMask`) on commit; every other feature may raise
or lower over water (Coastline is defined as pulling terrain toward
`seaLevel`). The categorical paint tools (`_paintAt`) are the ones with a hard
water gate, land-only by construction (`wb[i]!==0`).

**Panning while painting** is solved for touch in the reference (a relocated
joystick, `_sculptNavPanLoop`), since a single-finger drag is a stroke. Input
routing, not a tool definition; the port's phone uses a navpad and two-finger
pan (`SCULPT_FUNCTION_CHART.md` §9).

## The shared editing model: pass buffer, commit, discard, staleness

Every tool shares one mechanism. It is specified here as built (milestone A);
the first proposal was a plain struct holding a `HashSet` of touched tiles, and
the trait design below replaced it.

### What `DirtyTracker` gives, and what it does not

`DirtyTracker` (`cartalith-spatial`) is one `TileStatus` per tile —
`{dirty, reason, version}`. Only `mark_dirty` bumps `version`; `clear_dirty`
resets the flag without bumping it, since clearing is not a change to the data.
Its doc refuses Cartalith-specific field names — a generic reason string, not
an enum in a library crate — and that turns out right, because each pipeline
stage owns its own tracker. It holds **no data**: `TiledField<T>` holds one
array, the committed field, so there was nowhere to put a draft and nothing to
discard.

### The draft type

- **`Stamp`** (a trait, `cartalith-spatial/src/pass.rs`) — `bounds()` and
  `apply(&self, dst, width, height)`, with `type Cell` covering `f32` height and
  `u8` categorical layers alike.
- **`PassEntry<S>`** — a stamp plus its `hidden` flag.
- **`PassBuffer<S>`** — the ordered draft, touched-tile tracking, the draft
  undo/redo stack (`HISTORY_MAX = 30`), `move_up`/`move_down`.

Its contract:

- **Preview** — `preview_into(base, scratch)` composites the stack over a read
  of the committed field into a scratch buffer; `base` is a shared reference,
  so the no-mutation guarantee is the borrow checker's. `preview_touched_into`
  does the same inside `touched_bounds()` only and returns the window it
  touched. One apply function, two destinations — the reference's own contract,
  from the comment at lines 9016–9018 above `sculptStampBBox`/`sculptApplyStamp`:
  *"caller-supplied H/W arrays (never `field`/module globals) so both the draft
  preview (a scratch buffer) and commit (field itself) reuse the identical code
  path"*.
- **Commit** — apply every visible stamp, in order, into the real field;
  `mark_dirty` every touched tile **once**, with a reason, however many strokes
  touched it ("one committed pass"); clear the draft. `CommitSummary` returns
  the touched tiles.
- **Discard** — clear the draft. Nothing was ever written, so nothing needs
  undoing.
- **Undo granularity** — draft undo covers the four structural edits; field
  undo is one snapshot per commit.

### Staleness: what re-runs, and what stays deferred

The causal chain, confirmed against the crates: `cartalith-terrain` (height) →
`cartalith-hydrology` (flow, rivers) → `cartalith-climate` → `cartalith-civ`
(biome classification, soil/NPP/carrying capacity, settlement suitability,
roads, territory). The real graph has extra direct edges —
`build_settlement_suitability` takes height and slope directly — so civ depends
on height, hydrology and climate, and climate on height and hydrology
(`pipeline_stage_graph`).

The rule, as built:

- **A commit marks; the graph answers lazily.** A commit marks
  `PipelineStage::Height` changed at the touched tiles in a `StageGraph`
  (`cartalith-spatial/src/staleness.rs`). A stage is stale if an upstream's
  version moved past what it last computed against, **or** an upstream is
  itself stale, evaluated recursively at query time. Nothing is pushed
  downstream at commit time: a flow change's downstream footprint is exactly
  the expensive query to avoid.
- **The commit then runs the reference's own tail, once.**
  `cartalith_engine::staleness::recompute_stale` re-runs whatever is stale
  among hydrology and climate as **one** `refresh_climate` — discharge,
  temperature, rainfall — the reference's `computeFlow(true); refreshClimate();`.
  `WorldGen`'s sculpt, fjord and paint commit paths and its recompute actions
  call it (`8e666ac`, 2026-08-24). Erosion is not a stage: it is part of Height,
  which internally iterates (owner decision 2026-08-24, the
  `cartalith-engine/src/staleness.rs` module doc).
- **Civ stays stale until asked for.** `compute_civilisation` runs whole-field
  and lives in `cartalith-godot`; the reference's `sculptCommit` never cascades
  into settlements, roads or territory either. The status bar names what is
  stale (milestone F) and Recompute civilisation settles it.
- **Tile-incremental recompute is out of scope.** `cartalith-hydrology`,
  `cartalith-climate` and `cartalith-civ` operate on the whole field, so every
  stage recomputes globally when asked. Making them tile-incremental is a
  separate re-architecture, worth taking on only if lazy whole-field recompute
  proves too slow in practice.

## Tool by tool

*The 2026-08-18 reading of each tool: what it does in the reference, its
parameters, its pass-buffer and staleness behaviour, and what the port needed.
Corrections from the build passes are applied in place and point to the "as
built" section that found them.*

### Group 1 — Navigate & inspect

**Select/inspect (`V`)** and **Pan (`H`)** are viewport navigation: no field
mutation, no pass buffer, no staleness. Select/inspect registers no tool
handler — the shell's selection wiring is its behaviour — and Pan is a camera
modifier that is never armed.

**Point sample (`I`)** is a read, not an edit: the reference's `_civInfoAt`
(line 20436) generalises the settlement Inspector ("WHY HERE?", `VISION.md`)
to any point. No parameters, no pass buffer. It became the right dock's Sample
context rather than a tool (F).

### Group 2 — Terrain

All four share the Sculpt editor's registry and stamp model; they differ in
which `SCULPT_FEATURES` entries they expose. They became one `sculpt` tool id
(F), since splitting them further is UI sequencing, not an engine boundary.

**Raise/lower (`B`)** — `freehand` with `subMode` `raise`/`lower`, a per-pixel
add of `±amount`, coverage-weighted. Parameters, from the mockup's Properties
panel (`design/Cartalith DCC Shell.dc.html`) against the reference: `hardness`;
`intensity` (the mockup's "+120 m" is a presentation conversion via map scale
and height range, not an engine unit); `falloff` (the smoothstep above);
`noise scale` (the mockup's "0.8 km", the same point); `octaves`/`persistence`;
a raise/lower/smooth selector; and a `respect water mask` toggle. That toggle
has **no reference equivalent** — Freehand has no water gate — so it would be a
new gate, not a port. The mockup's `affect layer: bedrock + sediment` has **no
engine backing**: the height field is one `f32` array with no persisted
bedrock/sediment split; treat it as aspirational. Staleness: height edited →
everything downstream stale.

**Smooth (`S`)** — Freehand's `smooth` sub-mode, the one feature that bypasses
the per-pixel `apply()` and blurs a **stable pre-loop snapshot** of the field
(line 9039: *"the generic per-pixel-independent apply() path can't read stable
neighbour state"*). A port that reads and writes the live buffer mid-scan would
smooth direction-dependently. Parameters: `amount` (0.02–0.3, default 0.12),
`hardness`.

**Flatten/terrace (`F`)** — `SCULPT_FEATURES.plateau`: *"Terraced FBM mesa.
Sets a flat top; terraces quantize the surface. Never lowers existing
terrain."* (line 8917). Parameters: rise (0.03–0.45, default 0.26), terraces
(1–8, default 4), detail frequency (0.4–3, default 1.1). "Never lowers" comes
from `mode = 'set'; val = max(h0, level)` — a set-to-max, not an add, and the
trait that distinguishes this tool from Raise/lower.

**Stamp (landform library)** — the rest of `SCULPT_FEATURES` (Mountains, Hills,
Ridge, Cliff, Canyon, Valley, Coastline, Volcano, Basin) plus the eight
presets: a picker (landform × preset) feeding the same pipeline. Every
`apply()` body is a small, pure function with no DOM dependency — a large but
mechanical port.

### Group 3 — Water & ecology

**River/water (`R`)** — `SCULPT_FEATURES.river` and `.lake`, whose commit is
special-cased in `sculptCommit()`: River stamps run `enforceChannelDescent`
(carve-through and lock) and write `riverMask`/`riverFloor`; Lake stamps run a
**`waterOnly` dry run** *after* the main bake, so the final height is what is
tested against the lake surface (line 9074: a second normal pass would
double-carve the bowl), and deposit into `lakeMask`, which
`buildWaterBodies`' `forceLake` reads. Parameters: River — width (2–26,
default 7), depth (0.02–0.22, default 0.09), meander (0–0.6, default 0.28),
branch noise (0–1, default 0.5); Lake — depth (0.03–0.3, default 0.13), shore
(0.05–0.6, default 0.25). The structures already existed for the algorithmic
river network; what was new is the manual-stamp path into them.

**Biome paint (`P`)** — a different data shape. `classify_biome` is a pure
`(temperature, moisture) → biome` function with nothing to hold a manual edit.
The reference's answer is a **separate override array**: `paintBiome` (line
4764, a lazily allocated `Uint8Array(GW*GH)`), written by `_paintAt` (line 4783,
a hard-edged disc — *"categorical data has no half-painted state"*), `0` =
unpainted. **How the override is consumed** was corrected by milestone C: the
per-cell replace (`mb[i] = paintBiome[i]`) happens only in the Cartalith editor
export (line 12435), the renderer alpha-blends the painted colour over the
shaded colour at 0.60, and no analysis consumer merges at all. There are
**three** such layers — biome, terrain, splat. Painted overrides are cleared on
a full regenerate (line 3353: *"hand-painted Cartography overrides don't survive
a terrain rebuild"*).

Parameters: brush radius (`_paintRadius`, default 6 cells), *"a value picker
populated from `CART_BIOMES` (13 land biomes, water excluded — 'the brush never
touches water')"*, and an erase toggle. The reference's land-only gate
(`wb[i]!==0`, ocean **and** lake) is hard; the port keeps it as the default and
makes it switchable (`Land only`, backed by `PaintStamp::ungated` — milestone
C's one new affordance). The port also adds a brush falloff, as a
probability-threshold edge that never blends two palette indices
(`DECISIONS.md` §7k). Pass buffer: disc-paint deltas to the override array,
which `Stamp` is generic enough to hold. Staleness: painting biome does *not*
mark height/hydrology/climate dirty — it is downstream of them — it marks civ.

### Group 4 — Civilization

**Place settlement** — `_civDropPlace` (line 16051): a click near an existing
place selects it (prominence-weighted, `_civPlacePickWeight`); otherwise, on
land, it pushes `{x, y, name:'', kind:'town', faction, pop:1000, traits:[]}`
onto `state.places` and opens an editor. This port's
`SettlementPlacement`/`NamedSettlement` already carry that shape; what was
needed is a *manual insertion path* appending into the list `place_settlements`
produces, so naming, roads and territory treat a hand-placed settlement exactly
like a generated one. Parameters: kind, faction (shared with Territory's
active-faction select), initial population. Pass buffer: unnecessary — one
atomic append. Staleness: suitability-adjacent, roads, territory.

**Draw route/way** — manual, since this port's roads are algorithmic. The
match is `_civTool === 'draw_way'` (lines 26071–26107): the user clicks
waypoints, snapped to settlements, POIs and other ways (`_civSnapPoint`/
`_civFindSnapTarget`), and on commit (`_civCommitWay`, Escape) consecutive
waypoints are joined by terrain-cost pathfinding, producing a real path pushed
to `civWays[]` tagged `manual: true`. It is **not** `_civOpenRouteEditor` (line
20406), the Journey Planner's editor over an existing journey. The pathfinder
is **`_civDijkstraPath`** (line 25957), not the `roadDijkstra` kernel this port
already had — milestone D's headline correction: the wrapper (three cost grids,
the existing-way discount, settlement gravity, reconstruction, mode-matched
repair, the `reachable` flag) is most of the tool. Parameters: way type (road,
sea lane, …), which drives land-vs-water mode. Pass buffer: the in-progress
waypoint chain is itself the unit. Staleness: route corridors
(`build_route_corridors`), not terrain, hydrology or climate.

**Territory/faction** — `_civPaintTerritoryAt` (line 15964): a disc writing the
active faction into `civTerritory` within `_civTerRadius`, no falloff and no
land/water gate. It is the reference's **only** way territory is ever set
(`getCivTerritory()` only lazily zero-allocates; `PHASE2_SCOPE.md` milestone
9). This port has an algorithmic base the reference never had —
`assign_territory` (`DECISIONS.md` §7b), the same per-cell shape — so the design
is **paint as an override on top of `assign_territory`**, the Biome-paint
pattern, an addition under `DECISIONS.md` §7d. Parameters: active faction,
brush radius. Staleness: the narrowest in the group. *Added 2026-09-23 on owner
request:* a **Territory lasso** (`territory_lasso`) — a clicked polygon
rasterised by `civ_tools_bridge::polygon_cell_mask` and staged by
`CivTools::paint_polygon` into the same territory draft, faction-level only
(province-level assignment: not now, Ruling AP).

### Group 5 — Annotation & measure

**Label (`T`)** — `_civSelectLabel`/`_civConfirmLabel`/`_civCancelLabel`
(lines 15356–15367) with `drawArcLabel`/`_civSectorLabel` (15244/16509):
placed **arc** labels with editable `name`, `angle`, `arc`, `size`, `font`,
`color`, `sizeMode`. Selecting snapshots the editable fields once per session
so Cancel reverts cleanly, while dragging commits position immediately — two
deliberately different commit semantics, worth preserving. The port had no
label model; the reference is a complete spec. Pass buffer: unnecessary — a
discrete action with its own confirm/cancel.

**Icon stamp** — the **manual** half of a system whose rule-driven half the
port already had (`place_map_icons_ruled`, the scatter rules,
`composite_map_icons`). The reference has **three** placement paths, not two
(milestone E's correction): rule-driven autoplacement; click-to-place one icon
(the `_iconPlaceMode` click branch, lines 9774–9784); and a blue-noise scatter
brush (`_carIconBrushStamp`, line 15051). Parameters: icon family and slot from
the asset-pack taxonomy. Which art variant draws is chosen at composite time,
for manual and generated icons alike — `ManualIcon` carries no variant, though
the first reading guessed `pick_icon_variant` would serve one. Pass buffer:
discrete action, staging optional.

**Measure (`M`)** — **no reference precedent**: grepped (`ruler`,
`measureDist`, `distanceTool`), and only `updateScaleBar` (line 14024) turns up,
a passive scale bar. A distance readout over the same grid-to-km scale every
other distance in the port uses. An optional terrain-cost "travel distance"
variant would reuse the routing pathfinder. No pass buffer, no staleness — a
query.

**Region select/export** — `regionSel`/`regionDrag` (line 9583): a drag
rectangle (`normRegion`) feeding **tiled export** (`exportRegionTiles`, line
11891 — gzip-optional, 16-bit-packed height, a schema-2 manifest, headless-
testable but for the PNG step), **region amplification** (`amplifyRegion`,
line 13212 — upsample a sub-region into a standalone map, scaling
`mapWidthKm`), `exportGeoJSON` (line 12576) and the tile-grid overlay
(`drawExportTileGrid`, line 9602). None of the export functions existed in the
port: a real porting project, split into E and E2 when built. Parameters: tile
grid (cols/rows), tile size, gzip, and target resolution for amplify. Pass
buffer: the selection is itself the draft, adjustable before Export or Amplify.

## Milestone breakdown

`DCC_SHELL_SCOPE.md` expected scope *"potentially comparable to Journey Planner
or the Asset Library"*, and the investigation bears that out. Sequencing
follows dependency, not tool order: the shared mechanism first, then the Sculpt
port, which is the largest chunk and the most completely specified.

**Milestone A — `PassBuffer`/staleness core (`cartalith-spatial` +
`cartalith-engine`).** The draft type from "The shared editing model": stamp
storage, touched-tile tracking, preview via scratch composite, commit via real
write, discard. Per-stage staleness along the real dependency chain, lazy, no
eager cascade. No UI. Verifiable headlessly: commit/discard round trips and
staleness propagation over a small synthetic field. Every other milestone
depends on it.

**Milestone B — Terrain group, the Sculpt-editor port.** All thirteen
`SCULPT_FEATURES` (`apply()` bodies are small and individually portable), the
three noise families, the stamp bbox / coverage / domain-warp pipeline
(`sculptStampBBox`/`sculptApplyStamp`), the eight presets, wired to A.
Raise/lower, Smooth, Flatten/terrace and Stamp ship together because they share
one registry. Verify per-feature maths against the reference (as built, a stamp
turned out to be a reproducible golden fixture — see B).

**Milestone C — Water & ecology group.** River/water's special commit path
(`enforceChannelDescent` reuse, the lake `waterOnly` ordering) on B's pipeline,
kept separate from B because it is more delicate. Biome paint's override layer,
with an audit of the `classify_biome` consumers before wiring — don't assume
render-time only.

**Milestone D — Civilization group.** Place settlement's manual insertion into
the `SettlementPlacement`/`NamedSettlement` pipeline; Draw route/way's waypoint
collection, snapping, pathfinding and unreachable-leg behaviour, with a
`ManualWay` type; Territory's override on `assign_territory`. More independent
of each other than the terrain tools, so splittable.

**Milestone E — Annotation & measure group.** Label's model and arc-text layout
(the one genuinely new *rendering* problem in the plan); Icon stamp's manual
placement beside the rule-driven system; Measure; and Region select/export's
`exportRegionTiles`/`exportGeoJSON`/`amplifyRegion`/`packHeight16` port.
*Split in the building: E is the compute and encoding core, and its
PNG/gzip/`.zip`/GeoJSON half is **milestone E2** — boundary and reasoning under
"Milestone E as built".*

**Milestone F — Shell wiring.** Connect every tool built in B–E to the shell
(Track 1's inert controls, `DCC_SHELL_SCOPE.md` milestone 1), replacing "inert"
with real behaviour tool by tool, and give the status bar a staleness readout
from A's per-stage trackers. Where Track 1 and Track 2 merge.

**Deliberately in no milestone:** tile-incremental recompute of hydrology,
climate and civ (see Staleness).

## Milestone A as built (2026-08-18)

Shipped tested and unwired — the "primitive ahead of orchestration" precedent
Phase 2 and the Journey Planner used.

**Where it landed, and why.**

- `cartalith-spatial/src/pass.rs` — `Stamp`, `PassEntry<S>`, `PassBuffer<S>`,
  `CommitSummary`.
- `cartalith-spatial/src/staleness.rs` — `StageGraph`, `StageId`, `Staleness`.
- `cartalith-engine/src/staleness.rs` — `PipelineStage` and
  `pipeline_stage_graph()`: Cartalith's own stage names and edges.

The split follows `cartalith-spatial`'s precedent: its `DirtyTracker` refuses
to bake Cartalith field names into a library crate, and `QuadTree` takes
caller-defined flags for the same reason. Stage names and edges are pipeline
knowledge, so they live with the orchestrator. The tool system, not LOD
rendering, turned out to be what first put `cartalith-spatial` into the
pipeline.

**A stamp is a recipe.** The reference's stamp object is `{type, seed, pts,
g:{…}, f:{…}, hidden, _cx, _cy}` — feature key, seed, the stroke polyline in
grid coordinates, the eight globals and the per-feature values, a hide flag
and a cached centroid. It stores **no pixel data**; it is re-evaluated over its
own padded box whenever it is drawn or baked, which is what lets the draft be
plain state, snapshotted, reordered and discarded for free. Hence `Stamp` is a
**trait**, not a struct: the recipe is terrain-specific and belongs with B's
registry, the stack semantics are generic, and a biome disc, a territory disc
and a 13-feature landform stamp all implement it without the library crate
learning what a biome is. `hidden` moved onto `PassEntry`, because hiding is a
stack edit. A test asserts preview and commit produce identical results — the
test that would catch the two destinations drifting apart.

**Corrections this pass made to the plan.**

1. **`DirtyTracker` needed no extension.** `mark_dirty` already *is* "my data
   changed here, for this reason — bump the version", the one primitive editing
   and recomputation both need. `PassBuffer` uses it at commit; each
   `StageGraph` stage owns one.
2. **Staleness needs two rules, not one.** Comparing a stage's last-computed
   version against its upstream's is not transitive: a height edit bumps only
   height, so climate — comparing against hydrology, whose version did not
   move — would report itself current. Rule 2: a stage is also stale if an
   upstream is itself stale, evaluated recursively at query time. That keeps
   deferral while making civ stale after a terrain edit. A dirty-flag-only
   design fails the other way, and a test pins it: recomputing civ over a
   still-stale hydrology does **not** settle civ.
3. **Deferral is structural.** `StageGraph` has no recompute hook — no closure,
   callback or trait object — and every query takes `&self`. Work happens only
   when a caller runs a stage and says so via `mark_recomputed_tiles`; that
   caller is `recompute_stale` (see Staleness).
4. **The real chain has more edges than the spine.** `build_settlement_suitability`
   takes `field` and `slope_n` directly, so civ depends on height and hydrology
   directly, and `pipeline_stage_graph` encodes it. It does not change *whether*
   civ is stale after a height edit, but it is what a tile-incremental
   recompute would have to honour. **Erosion** was left out as a genuine
   erosion↔climate cycle a DAG cannot express; the owner resolved it on
   2026-08-24 — erosion is part of the Height stage, which internally iterates,
   so the graph stays four acyclic stages
   (`the_owners_erosion_decision_keeps_the_graph_at_four_acyclic_stages`;
   `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 item 4).
5. **The dirty flag and staleness are separate.** A stage's dirty flag means
   "changed, and the presentation layer has not re-read it" — a re-upload
   marker cleared by `acknowledge`. Staleness is computed from version counters
   only. Acknowledging never changes staleness, and a test pins that.

**Also built, from the reference rather than the plan:** the draft undo/redo
stack (`SCULPT_HIST_MAX = 30` → `HISTORY_MAX`) covering all four structural
edits and clearing the redo branch on any new edit. **Stack order is
load-bearing** — stamps read the destination they write, so they compose —
which is why `move_up`/`move_down` exist and why a test uses a set-to-constant
stamp to prove reordering changes the result (an add-only stamp would hide it).

**Verified** by behaviour: a stroke previews without mutating the field;
preview and commit agree exactly; commit applies the whole stack in order and
empties the draft; discard leaves the field **bit-identical** (compared as raw
bit patterns, so `-0.0`/`0.0` would fail); one commit bumps each touched tile's
version exactly once however many strokes touched it; discarded passes never
bump a version; staleness marks exactly the right stages at exactly the right
tiles and never recomputes.

**Left to later, deliberately:** the tools (B–E), the shell (F), and the
field-level undo snapshot — the port had no undo stack to snapshot into, and
inventing one before B had a real commit would have guessed its granularity.
`PassBuffer::commit` returns the touched-tile list a tile-diff undo would need.
The field undo that shipped is `WorldGen`'s whole-height `undo::HeightUndo`,
pushed once by `sculpt_commit`.

## Milestone B as built (2026-08-18)

The Terrain group's engine half: the whole thirteen-feature registry, ported and
golden-verified, wired to A and nothing else.

**Where it landed, and why.** `cartalith-terrain/src/sculpt.rs` (registry,
noise, geometry, stamp, the `Stamp` impl) and
`cartalith-terrain/tests/golden_parity_sculpt.rs`. Not a new crate and not
`cartalith-engine`: A's split leaves a third category, **subsystem-domain
maths**, and all thirteen features are height formulas. `ARCHITECTURE.md`
names `cartalith-terrain` as the crate that owns the height formula, and the
reference keeps `SCULPT_FEATURES` in script block 1 beside tectonics. A
`cartalith-sculpt` crate would have bought a `Cargo.toml` and nothing else;
`cartalith-engine` orchestrates and does not compute.

**The registry, as it turned out.** Thirteen entries in `Object.keys` order
(mountains, hills, ridge, plateau, cliff, canyon, valley, river, lake, basin,
coastline, volcano, freehand), eight presets, eight sub-modes, eight globals.
Three things reading it added:

1. **Registry order is load-bearing.** A stamp's effective noise seed is
   `(stamp.seed ^ ((index + 1) * 1013)) >>> 0`, `index` being the feature's
   position. Reordering silently re-randomises every stamp; `FEATURE_KEYS`
   carries the warning and a test pins the order.
2. **`edgeChar`/`edgeFreqMul` are per-feature data, not derived** — thirteen
   hand-tuned pairs (Coastline 1.5/0.55, ragged and slow; Mountains 1.4/1.5,
   tight and fast; River 0.4/0.8, nearly clean because meander supplies its
   shape). Ported as data.
3. **Volcano is the one feature that ignores `brushSize`.** `sculptStampRadius`
   gives it its own `volcRadius`, because its cone profile is defined by that
   radius. Lake, the other radial feature, uses the brush.

**How the brush actually works:**

- **Coverage** is `smoothstep(0, 1, (R − dist) / feather)`, `feather =
  max(floor, R · (1 − hardness))` — the reference registry's one falloff shape.
  (The port since added `sculpt::Falloff`: Smooth, the reference's, plus
  Linear, Sharp and Constant, with a measurement in its doc showing `hardness`
  cannot reach them — `SCULPT_FUNCTION_CHART.md` §4.)
- **`hardness` shapes, `intensity` scales** — coverage decides *where*,
  `k = cov · intensity` *how much*.
- **Two noise passes, not one.** The domain warp displaces the *sample
  position* before coverage is measured, so the silhouette moves; a second,
  3.4× higher-frequency term roughens `cov` only where `cov < 1`, so the
  interior stays solid while the rim breaks up. Both use `seed + 2100`; the
  feature bodies' `fbm`/`ridged`/`billow` use `seed`/`seed + 700`/`seed + 1400`.
- **`mode` is `add` or `set`, and which is a feature's defining trait.**
  `add` → `h0 + k·val`; `set` → `h0 + k·(val − h0)`, a coverage-weighted lerp.
  Plateau's set-to-`max(h0, level)` is why it never lowers terrain.

**Corrections and additions this pass made to the plan.**

1. **The plan's verification note was too pessimistic — the headline result.**
   It said interactive behaviour has no golden trace and to unit-test the
   `apply()` algebra. That conflates a *stroke sequence*, not a reproducible
   fixture, with a *stamp*, which is: the reference stores one as plain data,
   and `sculptApplyStamp` runs under Node with no pointer events, DOM or
   `generate()`, because the reference marks the block *"pure, DOM-free core"*.
   So B got real golden parity, every case bit-exact.
2. **Three FBM families plus a fourth noise consumer.** The edge warp and the
   rim-detail term are separate uses of `sculptFbm` at their own seed and
   frequency, living in `sculptApplyStamp` rather than any `apply()` body;
   porting them as "part of the features" would have missed them.
3. **Smooth also ignores `waterOnly`**: the smooth branch returns before the
   water-only check. Unreachable (only Lake stamps are passed `waterOnly`),
   ported as is and documented at the site.
4. **`sculptStampBBox` and `sculptApplyStamp` disagree about `feather`,
   deliberately kept.** The bbox uses `max(2, rad·(1−hardness))` for every
   feature, `apply` `max(1.5, R·(1−hardness))` for non-radial ones. The bbox's
   floor is larger, so the box always covers what `apply` writes; "fixing" it
   would change which tiles a stamp reports as touched.
5. **Sea level lives on the stamp.** The reference reads `state.seaLevel` live
   at apply time, so moving the sea slider re-renders existing Plateau and
   Coastline drafts (the only two features that read it). `Stamp::apply` takes
   only a destination, so `sea_level` is a stamp field with `with_sea_level()`
   as the explicit re-stamp. The owner has since ruled the other way —
   **Ruling 17**: re-read live, matching the reference, accepting that one
   slider retroactively changes every committed stamp. Investigated 2026-09-24:
   the precondition does not exist in this port — the world's sea level changes
   only on generate and load, which both replace the draft — so
   `with_sea_level` has no caller, and the ruling lands with
   `OUTSTANDING_WORK.md` §2.5's "Sea level is not a live control".
6. **`Math.hypot` is not `sqrt(x²+y²)`** — V8 divides by the larger magnitude
   and compensates. Ported as V8 computes it (now `cartalith_jsmath::js_hypot`).
   B's own fixtures could not tell the two apart, because the `f32` store
   absorbs the difference; the real risk is named at the site
   (`nearest_on_stroke` picks a segment by `dist < best`, so an ULP can flip the
   sign of `sd`, which Cliff and Canyon read). Milestone D found the fixture
   that does distinguish them.
7. **`Math.pow`/`Math.exp` needed no tolerance.** Every value is rounded to
   `f32` exactly where the JS `Float32Array` assignment rounds it. The
   razor-thin part: the *fixture's own* base field must be built in `f64` and
   rounded once at the store — building it in `f32` shifts it by an ULP and
   fails every case.
8. **A limitation carried over, not introduced.**
   `docs/SCULPT_EDITOR_INTEGRATION_PLAN.md` §6 left open whether stroke
   distance handles the world-mode antimeridian. The shipped
   `sculptNearestOnStroke` answers it: **no** wrap handling. This port matches;
   worth revisiting when world-mode sculpting becomes real, but inventing wrap
   behaviour would break parity for the common case.

**Verified.** Golden parity under a Node `vm.runInContext` harness over four
contiguous slices of v2.10 — 2292–2293, 7568–7569, 8304, 8821–9081 — each with
a block-comment balance assertion and a top-level-boundary check. It earns its
keep: 8821–9081 opens and closes inside long `/* … */` blocks, so an off-by-one
would have silently swallowed code rather than thrown. Cases: the twelve
non-Freehand features, the eight sub-modes, the "Alps" preset, Lake's
`waterOnly` dry run, and a cross-check that no two features produce the same
field at one seed (which a harness with a copy-paste error would not catch).

**Left out, deliberately:** `sculptCommit`'s water hooks (milestone C) —
though `apply_into`'s `water`/`water_only` parameters, one branch inside this
function, are here; the "respect water mask" gate for Raise/lower (new, not a
port); stroke capture and simplification (`rdpSimplify`/`catmullRomSample`,
input routing); the `SCULPT_COLORS` overlay palette; and shell wiring (F).

## Milestone C as built (2026-08-18)

The Water & ecology group's engine half — River/Lake's commit hooks and the
Cartography paint brush — golden-verified and wired to A.

**Where it landed, and why.**

- `cartalith-spatial/src/paint.rs` — `PaintStamp`, `PaintLayer`; golden
  `cartalith-spatial/tests/golden_parity_paint.rs`.
- `cartalith-hydrology/src/lib.rs` — `enforce_river_channels`.
- `cartalith-engine/src/sculpt_commit.rs` — `WaterState`, `commit_sculpt_pass`,
  `SculptCommitSummary`; golden `cartalith-engine/tests/golden_parity_sculpt_water.rs`.
- `cartalith-civ/src/lib.rs` — `apply_force_lake`, closing a gap this milestone
  opened (below).

1. **The paint brush is generic machinery → `cartalith-spatial`.** A hard-edged
   categorical disc over a `u8` grid, gated by a caller-supplied mask, knows only
   that `0` means unpainted. A's `pass.rs` anticipated exactly this type, and it
   means D's Territory paint needs no new stamp type.
2. **The water commit path is orchestration → `cartalith-engine`.** It composes
   spatial's `PassBuffer`, terrain's `SculptStamp` and hydrology's
   `enforce_channel_descent`, computes nothing new, and `cartalith-engine` is
   the only crate depending on all three.
3. **`enforce_river_channels` is hydrology-domain → `cartalith-hydrology`**,
   beside `enforce_channel_descent`, as in the reference.

### What River/water's "special commit path" actually is

Read directly (reference 9318–9346), a fixed five-step sequence whose
**ordering is load-bearing**:

1. **Bake the whole stack** — every feature. `PassBuffer::commit`, unchanged.
2. **`enforceRiverChannels()`** — re-clamp cells locked by an *earlier* commit
   (or by generation's `carve_river_valleys`) back to their floor, **after the
   bake and before this batch's carving**: a non-river stamp *"can raise
   terrain over an already-locked river channel … re-clamp locked cells back to
   their floor before this batch's own river hook carves+locks any NEW
   cells."* Run it before the bake and a Mountains stamp across an old river
   buries it. The step a naive port drops, so it has two tests: that the
   re-clamp holds, and that the same stamp *does* raise those cells with no
   lock recorded — otherwise the first would pass against a no-op.
3. **Per river stamp, in stack order**: `enforce_channel_descent` over the
   stamp's own stroke, then lock every carved cell into
   `river_mask`/`river_floor`.
4. **Lake, last, as a `water_only` dry run** against the final height,
   depositing into `lake_mask`.
5. **One `computeFlow(true)`, one `refreshClimate()`.** Not part of
   `commit_sculpt_pass`: steps 1–4 write the state a commit *produces* and are
   local to the stamps' footprints; step 5 is a whole-field recompute of
   downstream stages, run by the caller — `WorldGen::sculpt_commit` marks Height
   changed and calls `recompute_stale` (see Staleness).

**Corrections and additions.**

1. **`half_w` is the brush, not the discharge.** `carveRiverValleys` derives its
   half-width from Strahler order and a real-km scale; `sculptCommit` uses
   `max(1, brushSize·0.13)`. Right, not inconsistent — a hand-painted river has
   no drainage area — but porting the generated formula would have changed
   every hand-painted channel.
2. **`enforceChannelDescent` walks the stroke's own points and never
   resamples.** A two-point stroke carves at two sites and locks **3 cells**; the
   same stroke as 23 points two cells apart locks **46**. The reference gets
   away with it because a captured pointer polyline is dense. **A constraint on
   the shell: stroke capture must not decimate hard**, or a river carves
   visibly, locks almost nothing, and is refilled by later erosion. Both
   fixtures ship.
3. **A draft with no water stamps is bit-identical to a plain commit** — tested
   on raw `f32` bit patterns — so callers never choose between two commit
   functions.

### How Biome paint's override layer works

The plan's core was right — a separate override array, `0` = unpainted, a hard
land-only gate in the reference. Three things around it were not:

1. **Three paint layers, not one.** `paintBiome`, `paintSplat` (asset-pack
   ground textures) and `paintTerrain` (`CART_TERRAINS`, the "surface
   underfoot" palette) are peer `Uint8Array(GW*GH)` arrays driven by one brush
   and switched by `_paintLayer`. One `PaintStamp` serves all three;
   `PaintLayer` is instantiated per layer. A biome-shaped type would have
   locked the other two out.
2. **The merge is two operations at two altitudes.** The plan asked for a merge
   at *"every render/query site that currently calls `classify_biome`"*. The
   audit's answer: the **per-cell replace** happens in exactly one place, the
   Cartalith editor export (line 12435) — `PaintLayer::merge_over`. **The
   renderer replaces nothing**: `landColorCore` (7898–7900) alpha-blends the
   painted index's colour over the *fully shaded* colour at **0.60**,
   deliberately *"not a rewrite of the `materialWeights` mix … so
   hillshade/AO/crest/splat/haze still show through and painted cells don't
   read as flat pasted stickers."* **No analysis consumer merges at all** —
   `buildEcoregions` and the Journey Planner's `currentCartBiome()`/
   `currentCartTerrain()` read the unpainted output. Painted overrides are
   presentation and export, never simulation input; merging at every classifier
   call site would have invented behaviour. The rasters underneath are
   `build_cart_biome`/`build_cart_terrain` (1-based `Vec<u8>`), `PaintLayer`'s
   shape.
3. **The gate is `wb[i] !== 0`, not `=== 1`** — the comment insists it
   *"excludes BOTH ocean(1) and lake(2), never a bare `field[i] < sea` check,
   which misses above-sea-level lakes."* A port gating on ocean alone passes
   every ocean test and paints over lakes, so the fixture's water band is
   classified **2**.

**Also from the reference:** `_paintSampleAt`'s **nearest-neighbour** sampling
(bilinear *"would blend two unrelated palette entries into a meaningless third
index"*), `getPaintLayer`'s lazy allocation with its resolution-change length
guard, and `state.cartoPaint`'s sparse `[index, value, …]` persistence with
its drop-out-of-range rule.

**One deliberate new affordance.** `PaintStamp::mask` is `Option`; `None` means
no gate. It exists for the mockup's *"respect water mask"* switch, which the
reference lacks. `PaintStamp::new` requires a mask and the ungated form is
separately named, so parity is the default and the addition opt-in
(`DECISIONS.md` §7d). The shell's `Land only` toggle is it.

**One question, since ruled.** The reference clears painted overrides on
terrain rebuild, and only ever had one `generate()`. This port has incremental
edits, and whether a Sculpt commit should clear the paint under it had no
reference answer. The owner ruled it on 2026-09-24 (Ruling AS,
`LARGE_ITEM_RULINGS.md`): **a sculpt commit clears the painted override cells
it covers** -- the cells each committed stamp gives a non-zero weight
(`SculptStamp::footprint`), on all three layers, drafts untouched
(`PaintEditor::clear_cells_under`, called from `WorldGen::sculpt_commit`).
Overrides are still dropped whole on regenerate and load
(`WorldGen::release_world`, `load_save`). The global undo stack holds height
only, so undoing the sculpt does not bring the cleared paint back; the clear
records its own non-reversible history row saying so.

### A gap this milestone opened and closed

`build_water_bodies` had deliberately omitted the reference's `forceLake` —
*"no painting UI exists in this port, so it would be an always-false input with
no caller ever setting it."* The Lake hook is its producer; without `forceLake`,
`lake_mask` would have been dead output. It ships as `apply_force_lake`, a
post-pass, and that is bit-equivalent, not an approximation: in the reference
`force` is the **last** mutation of `out`. The post-pass form also left
`build_water_bodies`' signature and callers alone.

### Verified

Golden parity, bit-exact on the first run, for the water commit path and the
paint brush, under the Node harness over six contiguous slices of v2.10
(2292–2293, 7568–7569, 8304, **8725–8745**, 8821–9081, **4758–4795**), each with
a balance assertion and start- *and* end-of-slice boundary checks — both new
slices sit hard against comment boundaries. The checks caught two things in the
two ways they can:

- The end-of-slice check threw on `hash`/`vnoise`, one-line functions whose
  closing brace is not at column 0 — a false positive, fixed properly (strip
  trailing comments, require a real terminator) rather than by deleting the
  check.
- A failure no balance check catches, producing *silently empty* output:
  `paintBiome`/`_paintLayer`/`_paintValue`/`_paintRadius` are `let`
  declarations, which in a `vm` script are lexical bindings, **not** context
  properties. Setting `ctx._paintRadius` from the host created a shadow the
  reference never read, so `_paintAt` ran against defaults. Everything now
  drives `_paintAt` from inside the context.

`sculptCommit`'s water-hook body is **transcribed, not sliced** (lines
9320–9346, with its DOM and whole-pipeline calls dropped) — disclosed, because a
transcription is weaker evidence than a slice. The base field is built in `f64`
and rounded once; heights compare as raw `f32` bit patterns folded FNV-1a-64,
with **no tolerance anywhere**. `hidden_river_is_skipped` reproduces B's own
`mountains` golden hash **exactly** — independent evidence that the water hooks
are inert rather than usually harmless.

**Left out, deliberately:** the interaction halves — stroke/tap capture, the
pickers, the layer switch — are F's; automatic river tooling is untouched (this
adds the *manual* path into the same structures). The pack-image decode for
painted biomes and terrains and the 0.60 blend in `land_color` were render-side
changes scoped out here and built later.

## Milestone D as built (2026-08-18)

The Civilization group's engine half — Place settlement's insertion path, Draw
route/way's pathfinder and snapping, Territory's override — golden-verified
against the reference, wired to nothing.

**Where it landed:** `cartalith-civ/src/tools.rs` (the whole milestone) and
`cartalith-civ/tests/golden_parity_civ_tools.rs`, plus `cartalith-civ/src/lib.rs`
(a widened `TerrainValid`, the V8 `hypot`, and a bug fix in `civ_smooth_path`).
One crate, deliberately: **each of the three is a manual entry point into a
pipeline this crate owns.** Manual insertion appends to the same
`Vec<NamedSettlement>` `place_settlements`/`name_and_populate_settlements`
produce; manual ways reuse `road_dijkstra`, `civ_routing_grid`,
`civ_apply_settlement_gravity` and `civ_smooth_path`, which are **private to
the crate**; territory paint merges over `assign_territory`'s output.

**C's prediction held** — Territory needs no new stamp type:
`PaintStamp::ungated` **is** `_civPaintTerritoryAt`, cell for cell (`_paintAt`'s
comment calls itself *"a direct lift of `_civPaintTerritoryAt`'s geometry"*), so
the new surface is a five-line `merge_territory_paint`, golden-verified by
hashing the whole raster against `civTerritory`. Draw route/way, by contrast,
was the largest item by a wide margin.

### The headline correction: `_civDijkstraPath` is not `road_dijkstra`

The plan said the pathing primitive was `road_dijkstra`, already ported, and
that only waypoint collection, a `ManualWay` type and an unreachable-leg
fallback were new. **Wrong, and the gap is most of the tool.** `road_dijkstra`
is the reference's `roadDijkstra` (line 3275), the bare single-source
relaxation over a caller-supplied cost array. `_civDijkstraPath` (line 25957) is
one of its *callers* and calls it once; everything that makes a route a route
lives in the wrapper:

1. **Three cost grids.** `_civLandCostGrid` (21035: slope cost with *all* water
   impassable, above-sea lakes included via the water-body overlay, because a
   bare `field < sea` check misses them), `_civWaterCostGrid` (21051: the mirror
   — any water flat-cost 1, land impassable, lakes included unlike
   `_civMstRoutes`' ocean-only grid), and `_civMixedCostGrid` (21090: land and
   water, slope × biome friction × the navigable-river discount, with
   `_CIV_SEA_COST = 0.6` *below* the flat-land baseline, v0.94's deliberate
   correction).
2. **The existing-way discount** (`_civMarkWaysOnGrid`/`_civMarkWayNeighborhood`/
   `_civWalkWayCells`, 21757/21752/21766, `_CIV_EXISTING_WAY_DISCOUNT = 0.25`).
   `_civWalkWayCells` *rasterises the segments between* the sparse smoothed
   points, because `pts` alone is gappy on long straights and routers used to
   ignore half a road.
3. **Settlement gravity** — already ported (`civ_apply_settlement_gravity`),
   never before called from a manual tool.
4. **Reconstruction into world coordinates**, `((rx+0.5)/sc, (ry+0.5)/sc)` per
   routing cell, with the caller's own endpoints restored at full precision.
5. **Wrap-aware smoothing with a mode-matched validity repair**, needing two
   `_civTerrainValidTest` modes this crate lacked.
6. **The `reachable` flag** (v1.47) — the only way a caller tells a real path
   from the synthesized fallback.

The port is `civ_dijkstra_path`. The reference signals the distinction itself:
`roadDijkstra` sits in script block 1 beside `buildTravelCost`, ~22 500 lines
before `_civDijkstraPath`, whose header says it *"mirrors buildRoadsOp"* — a
caller, not the kernel. It is also the pathfinder `_jpRerouteForMode` needed,
since that function *"never silently accepts `_civDijkstraPath`'s straight-line
fallback as if it were a real path"* (`JOURNEY_PLANNER_SCOPE.md`, "The function
census (closeout, 2026-08-18)").

### What the reference corrected, tool by tool

**Place settlement.**

1. **Gate order is load-bearing:** bounds, then **select-near-existing**, *then*
   the water refusal. A settlement whose terrain later changed under it stays
   selectable (the v1.86 comment worries about exactly that); checking water
   first would make it unclickable.
2. **`_civPlacePickWeight` is prominence, not nearest-pixel** (v1.88): the
   winner minimises `d² / weight²` with `weight = 4 + rank`, mirroring
   `drawCivLayer`'s pin size, so a big city beats a slightly closer hamlet. The
   port's `SettlementKind` has the six tiers the pipeline produces (hamlet 0 …
   metropolis 5, ranks matching the reference); the reference's four special
   kinds (monastery, fortress, university, industrial) and the POI's flat
   weight of 5 are absent, not approximated.
3. **Three fields the reference's place object lacks** — `suit`, `capital`,
   `coastal`, which the reference recomputes on demand. `capital` follows from
   `kind`; `coastal` uses the same `civ_is_coastal(.., ocean_only = true)` and
   `max(6, gw/60)` radius as `place_settlements`; `suit` is the caller's.
4. **Name and population stay the reference's placeholder**, `""` and `1000`,
   not `civ_base_pop_for_kind`. Naming inside `civ_drop_place` would consume
   draws from `civ_name_rng`'s stream out of band and silently rename every
   later generated settlement. It has to be the caller's explicit step.

**Draw route/way.**

1. **A second, closer trap.** `_civCommitRoute` sits eighteen lines above
   `_civCommitWay`, looks nearly identical, and is a different tool: it routes
   `'mixed'` and pushes to `civJourneys`; `_civCommitWay` routes `'water'` for
   sea lanes, `'land'` otherwise, and pushes to `civWays`. Porting the wrong one
   would let a hand-drawn road cut across a bay *and* file it with the journeys.
   (The Route tool became its own `route` tool id in F.)
2. **The unreachable fallback is not a straight line from start to end.** The
   `{pts: fp}` straight-line branch runs only when `_civSmoothPath` returns
   `null`, which it does not for a distant unreachable target: reconstruction
   yields `[start, targetCell, end]`, `_civSmoothPath` splits runs at any
   `|Δx| > GW/2` jump — **unconditionally, world mode or not** — and the run
   holding the start has length 1 and is dropped. The drawn stub sits at the
   **target** end and the start is absent. Golden-verified and pinned by a test
   asserting the start really is missing, so nobody "fixes" the port into
   disagreeing with the reference; the shell's warning must not promise a line
   between the two waypoints.
3. **`_civTerrainValidTest` needed all four modes.** `civ_dijkstra_path` needs
   `'water'` (ocean *or* lake), `'land'` **with the v1.99 sea-lane ferry
   exception**, and `undefined` (mixed, nothing to repair). The ferry exception
   is the only place a land-mode `Infinity` cell becomes finite, so without it
   the repair pass drags a legitimate ferry leg back onto dry land.
   Golden-tested, with its negative half (the same pair is unreachable with no
   lane).
4. **`_civWalkWayCells` is used twice with a deliberate asymmetry**:
   `_civMarkWaysOnGrid` skips `w.hidden`; the sea-lane collection in
   `_civTerrainValidTest` does not. Ported as written, commented at the site.
5. **`state.roads.edges` has no equivalent** — `buildRoadsOp`'s legacy
   Edit-tab output, which this port never had. A caller's generated `Way`s go
   through the same `ways` slice instead. Named rather than silently dropped.
6. **The cost grid is rebuilt per leg**, as the reference does: hoisting would
   be a real optimisation *and* a divergence, since the way discount and
   settlement gravity mutate the grid in place.

**Territory/faction — an addition, not parity.** The reference has a territory
*brush* and nothing else (`PHASE2_SCOPE.md` milestone 9). The brush is a
faithful port; what it composites onto — `assign_territory`, `DECISIONS.md`
§7b — is new, a superset under §7d.

- **`ungated`, not `new`.** `_civPaintTerritoryAt` has **no land/water gate**,
  unlike `_paintAt` — a faction can own coastal water and lake surface. C's
  `PaintStamp::ungated` turns out to be the reference-faithful choice here.
- **Faction ids widen at the merge.** `civTerritory` is a `Uint8Array`;
  `assign_territory` returns `Vec<i32>`. The `u8` layer covers every faction the
  reference could express, and the widening happens in `merge_territory_paint`
  and nowhere else.

### Two bugs found in already-shipped, golden-verified code

Latent because no fixture in this crate had a **wrapped** route; both fixed
with every pre-existing golden still passing.

1. **`civ_smooth_path` summed `km` across run boundaries.** The reference
   guards the accumulation with `if(k > 0)`, `k` the index *within the current
   run*, so the seam a `brks` entry marks is excluded; the port used "if
   anything has been pushed". Case 1, the first wrapped fixture, reported
   876.8 km for a route the reference measures at 136.6 km — one map width added
   per seam crossing. Every consumer of a wrapped way's length
   (`civ_consolidate_and_smooth_ways`, `civ_sea_routes`, manual ways) was
   affected.
2. **`Math.hypot` became test-enforced.** `_civSmoothPath` accumulates `km` in
   `f64` across dozens of segments with no rounding, so one ULP survives: case
   1's unreachable land route is `610.6390435628962` with Rust's `f64::hypot`
   and `610.6390435628963` — the reference's value — with V8's. The compensated
   form was applied across the route-geometry chain (`civ_rdp_simplify`,
   `civ_catmull_rom_sample`, `civ_smooth_path`, `civ_dijkstra_path`'s
   fallback); it now lives in `cartalith_jsmath::js_hypot`, shared with
   `cartalith-terrain`. Other `.hypot()` sites covered by their own passing
   goldens were deliberately not changed on a hunch.

### No `PassBuffer` in this milestone, deliberately

Place settlement is one atomic append. Draw route/way's in-progress waypoint
chain is the natural unit, and `civ_commit_way` takes it as a plain slice (the
reference's `_civWayWaypoints`). Territory paint stages through C's
`PaintLayer`; a `PassBuffer<PaintStamp>` on top would be a second staging
mechanism over the same data.

### Verified

Golden parity, every case bit-exact — `km` compared as raw `f64` bit patterns,
the territory raster as FNV-1a-64 over its bytes, **no tolerance anywhere**.

- **Whole `<script>` blocks, not line slices:** blocks #1 (2084–14556) and #2
  (14563–26720), the boundaries `golden_parity_hierarchical_network.rs`
  documents, with the harness asserting the line before each *is* `<script>`
  and the line after *is* `</script>` — stronger than B/C's slices.
- **The balance checks fired twice and were wrong both times**, which is how
  such a check proves it is looking: nested template literals desynchronised a
  crude string skipper (fixed with a real template-literal stack), then regex
  literals containing a bare `"` read as string openers (fixed with a
  regex-literal skipper).
- **Emptiness assertions before any golden was written:** every "should route"
  path has ≥ 2 points and `km > 0`; every "should not route" reports
  `reachable === false`; the territory brush painted cells; the drop tool
  appended exactly one place; the unreachable commit produced a warning.
  Re-asserted on the Rust side.
- **The world under the tools is bit-identical, checked first:** the harness's
  `field`, water-body classification, biome raster and Strahler order were
  FNV-1a-64'd and matched this port's `generate_terrain` +
  `build_water_bodies` + `build_biome_raster` + `fresh_river_order` exactly.
  Case 0 (`gw=24 gh=18 seed=24601`, not wrapped): a western landmass, an ocean
  and an eastern strip, so a western land route is real, an eastward one
  unreachable, and `mixed` crosses water. Case 1 (`gw=20 gh=16 seed=314159`,
  wrapped): an ocean connected *only through the seam*, so both routes carry a
  `brks` entry — the path that found bug 1. Both have real ocean, land and at
  least one lake (case 1: 42 lake cells).
- Six presentation-only functions (`_civRenderPlaceEditor`,
  `_civRenderWayList`, `_civRenderJourneyList`, `_civUpdatePlannerPanel`,
  `drawCivLayerAuto`, `renderNow`) are neutralised **inside** the context; no
  tool body is transcribed or edited. Everything is driven from inside the
  context (`civWays`, `_civActiveFaction`, `_civTerRadius`, `_civWayWaypoints`
  and `civTerritory` are all `let` bindings).

**Left out, deliberately:** the interaction halves — waypoint capture and
Escape-to-commit, the shared active-faction select, radius and way-type
pickers, the snap switch (`state.viz.snapWays`) — are F's (`civ_zoom_pick_r` is
exposed so the shell supplies its own zoom). Outside this milestone: `_civDropPOI`
(this port has no POI concept), `_civConnectPlaceToNetwork` (the "connect a
hand-placed settlement to the network" spur, not one of the tools), and
`_civGenerateProvinces` over a *painted* territory raster (provinces are
generated from `assign_territory`'s output).

## Milestone E as built (2026-08-18)

The Annotation & measure group's engine half — Label, Icon stamp, Measure, and
the compute/encoding core of Region select/export — golden-verified, wired to
nothing.

**Where it landed.**

| Piece | Code | Golden |
|---|---|---|
| Label | `cartalith-civ/src/labels.rs` | `cartalith-civ/tests/golden_parity_labels.rs` |
| Icon stamp | `cartalith-assets/src/manual.rs` | `cartalith-assets/tests/golden_parity_manual_icons.rs` |
| Measure | `cartalith-spatial/src/measure.rs` | none possible — see below |
| Region rectangle | `cartalith-spatial/src/region.rs` (`norm_region`, `tile_dims`, `FloatRegion`) | `cartalith-spatial/tests/golden_parity_region.rs` |
| Amplify | `cartalith-terrain/src/amplify.rs` (`amplify_region`, `refine_tile`) | `cartalith-terrain/tests/golden_parity_amplify.rs` |
| Encodings | `cartalith-io/src/tiles.rs` (`pack_height16`/`unpack_height16`, `TileManifest`, `manifest_json`) | `cartalith-io/tests/golden_parity_tiles.rs` |
| Composition | `cartalith-engine/src/region_export.rs` (`export_region_tiles`) | `cartalith-engine/tests/golden_parity_region_export.rs` |

Each placement follows A–D's rule (generic machinery → `cartalith-spatial`,
pipeline knowledge → `cartalith-engine`, domain maths → the owning crate).
Label is the reference's own `_civ` family, drawn beside places, ways and
territory and sized from this map's zoom-relative icon scale. Icon stamp is the
manual half of a rule-driven system `cartalith-assets` owns (`icon_brush_rule`
reads the same `ScatterRule` table as `place_map_icons_ruled`). Measure and the
region rectangle are generic grid machinery; `FloatRegion` is load-bearing,
since `refine_tile`'s sub-bounds are `region.w / cols`, generally not an
integer, and rounding them would break the seam agreement tiling rests on.
Amplify is a height formula; the encodings are what a Cartalith file looks like
on disk; the composition orchestrates.

### Region select/export did need a split — and here is the boundary

`exportRegionTiles`' body is four calls and a loop; everything hard in it is
either pure geometry, which ships in E, or a browser API, which cannot. So E is
the **compute and encoding core** and **E2** is *format and pixels*:

| Shipped in E | Deferred to E2 |
|---|---|
| `normRegion`, `tileDims` | per-tile PNG (`tilePngBytes`, an `OffscreenCanvas` hypsometric-tint + hillshade pass) |
| `amplifyRegion`, `refineTile` | `gzipBytes` (`CompressionStream`) |
| `packHeight16`, `unpackHeight16` | the `.zip` assembly (`zipStore`) |
| `buildTileManifest` + byte-exact JSON | `exportGeoJSON` (12576) and its `_geoXY`/`_geoTerritoryFeature`/`_geoProvinceFeature` raster-to-vector tracer |
| `exportRegionTiles`' own assembly, minus the two browser steps | `regionNewWorldBtn`'s replace-the-world path (orchestration over a live world) |

A *smaller* E2 than the plan feared: the geometry is done and bit-exact, and
what is left is a PNG encoder, a gzip crate and a GeoJSON serialiser, none
needing the reference re-read for its maths. `burnChannels` (line 10317) is in
neither half — it belongs to the LOD viewer. The selection interaction
(`regionSel`/`regionDrag`, the dashed overlay, `drawExportTileGrid`) is F's.

### What the reference corrected, tool by tool

**Icon stamp — the plan described the wrong function.** There are **three**
placement paths:

1. Rule-driven autoplacement (`placeMapIconsRuled`) — already ported.
2. **Click-to-place one icon** — the `_iconPlaceMode` click branch (9774–9784),
   four lines: `place_manual_icon`.
3. **A dart-throwing scatter brush** — `_carIconBrushStamp` paints a blue-noise
   *stand* of icons under a radius as the pointer drags, with a rejection radius
   tested against the icons already on the map and the ones this stamp is
   placing. By far the larger manual path, and the plan did not describe it.

It is **deliberately non-deterministic**: *"Unlike the procedural scatterer
this uses `Math.random`, not `hash()`: a brush stroke is an authoring ACTION
whose result is persisted in `state.mapIcons` — re-painting the same spot should
add new icons, not deterministically reproduce the previous ones."* So
`icon_brush_stamp` takes its randomness as a parameter
(`&mut dyn FnMut() -> f64`), the harness overrides `Math.random` inside the vm
context with a seeded LCG, and the port drives the identical stream. The RNG is
consumed three times per accepted dart and twice per rejected one, so matching
every placed icon pins the exact accept/reject sequence, not just the outcome.
Two smaller corrections: the click path has **no sea-level gate** (only the
brush does), ported as written; and `icon.fam`'s `'feature'` maps to the *pack*
family `icons`, so `ManualIconFamily` is its own type with a `pack_family()`
bridge.

**Label — the arc layout is real, and splits cleanly at text measurement.**
`drawArcLabel` (15244) is a Canvas function, but only two inputs come from the
canvas — `measureText(text).width` and the per-char advances — both properties
of the font. So `arc_label_layout` takes them as parameters and returns one
`{dx, dy, rot}` per glyph in the label's frame; the renderer applies them, and
`cartalith-civ` stays free of Godot. Four things the plan lacked:

1. **`total_w` and the per-char widths are separately load-bearing** — the
   total for centring, the per-char widths in the loop — and in a real font
   they disagree because of kerning. The harness's stub metrics make them
   *deliberately* unequal.
2. **`sizePx` is truncated for the font string but not the geometry** —
   `${sizePx|0}px` sets the measuring font; the untruncated value feeds the
   arc-radius floor `max(sizePx·1.2, …)` and the halo `max(1, sizePx·0.16)`.
3. **The `|arc| < 0.01` straight branch is not an optimisation** — below it the
   radius `total_w / (2.2·|a|)` diverges.
4. **The two commit semantics are exact and asymmetric.** `_civSelectLabel`:
   the snapshot is taken *"once per edit session (re-clicking/dragging an
   ALREADY-selected label does not retake the snapshot)"*, and *"x,y are
   deliberately excluded — dragging to reposition commits immediately"*.
   `LabelEditSession` implements both; the golden pins that cancel reverts the
   name and **not** the position.

The three on-canvas handle formulas (resize, rotate, arc) are **transcribed, not
sliced** — they are inline in a `pointermove` listener. The shell's drawn glyph
layout and this box model were later found to use different font-size models;
Ruling AG (2026-09-23) rules that they unify on `map_overlay.gd`'s.

**Measure — an addition, flagged as one.** Re-grepping finds only
`updateScaleBar` (14024), so there is **no golden test for
`cartalith-spatial::measure` and there cannot be one**; it is unit-tested
against its own contract and recorded as new under `DECISIONS.md` §7d. What it
is faithful to is the km scale: `hypot(dx, dy) · map_width_km / gw`, the
expression `civ_smooth_path`'s golden `km`, `civ_catchment_radius_cells`'
`cell_km` and `_geoCellKm` all use, compared by a test as raw `f64` bit
patterns.

**Region export — a real division by zero, ported rather than fixed.**
`amplifyRegion` maps `cy = rh > 1 ? ry + (oy/(outH−1))·(rh−1) : ry`; with
`outH == 1` **and** `rh > 1` that is `0/0`, and the whole output is `NaN` —
verified against the reference and pinned by a golden, with a companion fixture
where the region also collapses and the result is finite (the pair
distinguishes `rh > 1` from `rh >= 1`). No shipped caller reaches it (`tileDims`
floors both edges at 2 px). It forced `js_min`/`js_max`: `Math.min(1, NaN)` is
`NaN`, while Rust's `f64::min` returns the other operand and would have turned
an all-NaN tile into a plausible one.

**The manifest JSON is written by hand, on purpose.** `serde_json` renders
`16.0` as `16.0`, `JSON.stringify` as `16`, and a schema-2 manifest is read by
other tools. `manifest_json` formats numbers as `Number.prototype.toString`
does; one golden uses `cols = 7` over `bounds.w = 30` so every
`coarse.x`/`coarse.w` is a long fraction.

### Verified

Golden parity under the whole-block harness (#1 2084–14556, #2 14563–26720,
`<script>`/`</script>` asserted at both ends), driven from inside the context,
with `renderNow`, `drawCivLayerAuto`, `_civRenderLabelEditor`,
`_civRenderLabelList`, `_carRenderIconList` and `_carSelectIcon` neutralised.
Two environment modifications, disclosed: a Canvas-2D stub for
`drawArcLabel`/`_civLabelBox` that records transforms and answers `measureText`
from a fixed formula (no function body transcribed or edited), and the seeded
`Math.random`.

- **The balance check fired twice and was wrong both times:** a `}` closing an
  object or arrow body inside a `${ }` substitution ended the substitution
  early (fixed with a brace-depth-anchored substitution stack), and the
  regex-literal skipper's "does a value precede this `/`?" test matched a
  *single* identifier character, so `c0.waveStr/Math.max(…)` read as a regex
  (fixed). The documented apostrophe-in-prose blind spot appeared as a
  *symptom* of the first.
- **Emptiness and shape assertions first:** all 13 rectangles non-empty; every
  non-degenerate amplification non-constant and inside `[0, 1]`; the collapsed
  run constant *and finite*; the `outW == 1` run entirely NaN; the four
  `refineTile` tiles agreeing on their shared edge exactly; straight- and
  arc-branch label layouts; hit-testing with hits, misses and topmost-wins;
  cancel reverting the name but not the position; brushed icons with **two runs
  legitimately empty** (a real negative control), every icon in bounds and on
  land; accepted and rejected click placements.
- **The fixture is synthetic, and both sides hash it first:** a height field
  from pure arithmetic (no `sin`/`cos`/`exp`, so V8's libm and Rust's cannot
  disagree about the input), a quantised `% 11` term, and land and water in
  quantity so the brush's sea-level gate is exercised.
- **One non-bit-exact result, measured.** Case 9 of the arc layout (36 glyphs)
  matches 106 of 108 values; two are **one ULP** off, both `dx`, both from
  `r · sin(theta)` — `dy` and `rot` agree at those glyphs, so `theta` is
  identical and the divergence is V8's `Math.sin` against Rust's. Safe here,
  since nothing branches on a glyph position. The test pins *exactly two* values
  within 1 ULP at exactly those indices, so it cannot quietly grow. Everything
  else compares **exactly**.

### Mutation testing

**89 mutations across the six modules — 86 killed, 3 survivors, all three shown
equivalent.** Every mutation touched its file and the runner asserted cargo
actually recompiled (a missing `Compiling` line counts as BROKEN, not a
survivor). This run applied mutations with comment-skipping `sed` addresses;
`MISTAKES.md` has since required an exact-literal Python replace with a
`finally` restore, and a new harness should follow that.

**The first pass found five fixture-shape gaps**, fixed with differently
*shaped* fixtures rather than weaker mutations: a larger fractional drag for
`norm_region`'s `ceil`; explicit minimums of 0 and 1 for its JS-falsy
`minW || 8`; an aspect-1 case below the 2 px floor for `tile_dims`'
`aspect >= 1`; a region and output collapsing together for the `rh > 1`/`rw > 1`
guards; two probes straddling a box edge for the label hit test's `side / 2.0`.

**Five brush constants no golden could reach** (`ICON_BRUSH_MIN_DENSITY`,
`ICON_BRUSH_MIN_SPACING`, `ICON_BRUSH_MAX_DARTS`, the `3.0` spacing constant,
the `× 2` oversample), for two structural reasons: a dart lands on an
**integer** cell, so dart-to-dart distances are integers and none lies between
2.9 and 3.0 — only an existing icon at a **fractional** position can see the
spacing constant; and `max(1.2, 3/sqrt(d))` reaches its floor only above
`d = 6.25`, beyond the 0..1 density slider. Killed by scripted-RNG unit tests
observing each constant directly, plus two goldens (a large zero-density brush,
an unsaturated five-tap drag).

**The three survivors, each shown equivalent:** `amplify_region`'s
`base < sea` → `<=` (at equality both branches give 0); `norm_region`'s
`x + w > gw` → `>=` (at equality the clamp is a no-op); and the `js_round`
half-up rule in `region.rs`, whose only caller passes strictly positive values,
where half-up and half-away-from-zero agree (the other `js_round`, in
`manual.rs`, sees negatives and is killed by a dart at exactly −0.5).

**Left out, deliberately:** the interaction halves (label drag/rotate/arc and
the new-name prompt, icon arm/disarm and brush toggle, the measure clicks, the
region drag and overlay, the `_civLabelPointerHandled` guard, the `!_lodOn`
gate) — F's; E2 in full; the reference's `_carDrawMapIcon`/`drawArcLabel`
*rendering*; `_carIconTypeList`'s glyph fallbacks; the list panels; and
persistence of labels and icons, which waited on a save writer. That writer now
exists (`DECISIONS.md` §7h; `annotations/{labels,icons,regions}.json` in
`project_bridge.rs`).

## Milestone E2 as built (2026-08-18)

The deferred half of Region select/export — format and pixels, as E scoped it —
plus `exportGeoJSON` and the non-UI core of `regionNewWorldBtn`,
golden-verified and wired to nothing. E's *"smaller E2"* assessment held: only
the verification grew.

| Piece | Home | Verified by |
|---|---|---|
| `hypso`, `SEA`/`LAND`, `edgeL/R/U/D`, `renderHeightTileRGBA`, `ToUint8Clamp` | `cartalith-terrain/src/tile_render.rs` (`u8_clamped` since moved to `cartalith-jsmath`, re-exported) | golden |
| `_geoXY`, `_geoTraceMaskRings`, `_geoRingArea`, `_geoPointInRing`, `_geoMaskOutlineCoords`, `toFixed` | `cartalith-spatial/src/geo.rs` (`js_to_fixed` since moved to `cartalith-jsmath`, re-exported) | golden |
| `gzipBytes`/`gunzipBytes` | `cartalith-io/src/gzip.rs` | unit, round trip |
| `zipStore` (generalised) | `cartalith-assets/src/archive.rs` (`zip_store`, `zip_store_bytes`) | golden |
| `exportGeoJSON`, `_geoTerritoryFeature`, `_geoProvinceFeature`, the `JSON.stringify` writer | `cartalith-engine/src/geojson.rs` (`export_geojson`) | golden |
| `tilePngBytes`, the gzip/PNG loop, `refineBtn`'s `.zip`, `regionNewWorldBtn`'s core | `cartalith-engine/src/region_export.rs` (`zip_region_export`, `extract_region_as_world`) | golden |

The tile visual is a pure function of a height tile and three scalars — a
height ramp and `shadeFactor`'s normal-from-height formula — so domain maths,
beside `amplify_region`. The raster→vector tracer operates on *a binary mask
plus a km scale* and knows nothing of what the mask means — the reference calls
one `_geoMaskOutlineCoords` from both the territory and the province exporter —
so it is generic. gzip sits beside `pack_height16`, whose bytes it compresses.
The compositions orchestrate.

### The zip and PNG conventions: reused, and one corrected

**The region export shares `cartalith-assets`' archive conventions because they
are literally one function.** The reference has one zip writer, `zipStore`
(12009), with three callers (asset pack, project `.zip`, region export), so
`cartalith-assets::archive` grew a neutral `zip_store`/`zip_store_bytes`,
`write_pack_entries` became an alias, and `cartalith-engine` took a
`cartalith-assets` dependency (it needed `raster::encode_png` anyway). **`.png`
entries are STORED** and **timestamps are frozen at 1980-01-01**.

**One convention Phase 4 milestone 2 had skipped turned out reachable.**
`zipStore` falls back to STORE whenever DEFLATE does not make an entry
*smaller*; milestone 2 read that as a browser concern. On a four-entry archive
shaped like a region export, **three of four entries come back STORED** — the
`.png`, a 7-byte `params.json` whose deflate header costs more than it saves,
and an incompressible blob. So `deflate_helps` measures first and chooses
second.

**How close the zip bytes get.** For a STORE-only archive the two writers
produce the **same 172 bytes** apart from two fields no reader interprets:
version-needed/made-by (`zip` writes `1.0` for a stored entry, the reference
`2.0`) and external attributes (`zip` stamps unix `0644`, the reference `0`).
The golden normalises exactly those and demands every other byte match —
stronger than a structural walk, and loud if a third difference appears.

**What cannot match, stated once.** Deflated entries, gzip streams and PNG
payloads come from `miniz_oxide` here and the browser's encoder there; two
conforming encoders need not agree on a bit stream. So the *decisions* are
golden-verified (method per entry, names, manifest fields), the *pixels* byte
for byte before encoding, and the containers by round trip both ways.
Reproducibility survives: gzip MTIME is pinned to `0` and zip timestamps to
1980.

### What the reference corrected, function by function

- **`tilePngBytes` has two renderers.** It picks `renderBiomeTileRGBA` over
  `renderHeightTileRGBA` when `state.mode === 'biome'`, and the biome renderer
  samples the whole climate stack. E2 shipped the height renderer (the
  reference's default and fallback) and disclosed the biome branch; the biome
  tile renderer came later with the LOD tile work
  (`render::render_biome_tile_rgba`, `golden_parity_tile_biome.rs`).
- **`Uint8ClampedArray` is not a cast.** `out[p] = c[0]*s` stores through ECMA's
  `ToUint8Clamp` — NaN → 0, clamp, **round ties to even**. `c[0]*s` is
  fractional almost everywhere, so `as u8` would be wrong in about half of all
  pixels.
- **`hypso` extrapolates past its palette into negative channels.** The depth
  ramp `d = (sea − v)/sea` is unclamped: at `sea = 0.3`, `v = −0.1` returns
  `[−0.67, −10.67, −16.67]`. Verified and pinned; harmless only because the
  clamped store catches it — a second reason `u8_clamped` cannot be shortcut.
- **`toFixed` does not round like Rust.** ECMA picks *"the larger n"* on a
  decimal tie; `format!("{:.3}")` picks the even one. Reachable: an 800 km map
  on a 12 800-cell grid has `cellKm = 0.0625`, so the first easting ties at
  three decimals — JS `0.063`, Rust `0.062`. `js_to_fixed` implements the spec
  rule over the exact decimal expansion.
- **The tracer's JS `Map` semantics are observable.** Ring discovery follows
  *insertion* order, and the checkerboard pinch the reference *"doesn't
  disambiguate"* works by one cell's edge **overwriting** another's at the same
  key — from outside, an **unclosed ring**, whose closing segment
  `_geoRingArea`'s `i < len−1` then omits. Reproduced exactly, with its own
  test.
- **`exportGeoJSON` needed its own JSON writer**, for `manifest_json`'s reason,
  since the document is compared to the reference as a whole string.
  `cartalith-io`'s `js_num`/`json_string` are `pub` and reused (`json_string`
  gained the two short control escapes `QuoteJSONString` uses, since a place
  name is arbitrary text).
- **`regionNewWorldBtn` is a UI action with a computational core:**
  `tileDims(sel, 1, 1, ts)` for the new grid, `max(1, mapWidthKm · sel.w / GW)`
  against the **old** `GW` for the new scale, and `amplifyRegion` for the field
  — `extract_region_as_world`. The rest (allocate, refresh climate, clear the
  civ layer, the confirm and the calibrate hand-off) is orchestration over a
  live world, listed in the function's doc. Two reference decisions kept: it
  does **not** normalise (the data is already elevation in the parent's
  `[0, 1]`), and clearing the civ layer is the honest answer rather than a
  subtly wrong remap.

### The harness bug that looked exactly like a reference bug

E never invoked `exportRegionTiles` itself. E2 could — Node has
`CompressionStream`, and `tilePngBytes` finds no `OffscreenCanvas` and returns
`null`, the headless behaviour the reference documents — and the real function
then disagreed with E on the **fourth tile only**. It was the harness: with the
DOM stubbed, block #1's boot code schedules a deferred first `generate()`, and
the reference's `microtask()` is `setTimeout(r, 0)`, which `exportRegionTiles`
awaits between tiles, so the boot overwrote `field` mid-loop. Fixed by making
`requestAnimationFrame` inert and draining pending macrotasks before installing
any fixture; all four tiles then matched E's hashes exactly, which **discharged
E's disclosure** that it had verified the primitives but not the assembly.
"The reference is non-deterministic" is a conclusion worth being slow to reach.

### Verified

Everything bit-exact with no tolerance: `hypso` as raw `f64` bit patterns, six
rasters as FNV-1a-64 over every byte plus their first and last twelve, both
GeoJSON documents as whole strings (2 136 and 924 characters), a STORE-only zip
as bytes. `renderHeightTileRGBA` calls `Math.sin`/`Math.cos` on the sun
azimuth, and the byte-exact match holds across four azimuths (0, 45, 200, 315)
— not one lucky argument, which matters given that `f64::exp` diverges from
V8's on 20 721 of 240 000 arguments (Phase 5 milestone 5).

### Mutation testing: 58 mutations, 54 killed, 4 survivors

**The first sweep was the useful one**: it started at 47 killed / 10 survivors,
and six of those ten were real fixture gaps:

1. **`_geoXY`'s three decimals** — every coordinate in the 12×9 fixture was a
   whole or `.5` km, so `toFixed(3)` and `toFixed(2)` agreed. Closed with a
   second fixture at `cellKm = 0.390625`.
2. **The tracer's `ring.length >= 4` filter, both directions.** Reachability
   settled by brute force: all 65 536 masks on a 4×4 grid through the
   reference's own tracer — length-4 rings occur for **1 695**, length-3 (which
   the filter drops) for **8 760**. Both fixtured from the reference's own
   examples.
3. **The shell/hole split's `area > 0`** — the sweep found rings of area
   **exactly zero**, which the reference files as holes. Closed with the mask
   that makes one.
4. **`v < sea` in the shading branch** — no pixel sat exactly at sea level. Two
   more rasters that do.
5. **`strahlerOrder`'s spelling** — the GeoJSON world traced no river. A second
   real `exportGeoJSON` run on a 24×18 bowl with two order-2 channels.

*A fixture sampled on round numbers cannot see a rounding rule, and one built
from tidy rectangles cannot see a degenerate-geometry branch.* Where
reachability was in question, brute force answered in seconds and beat any
amount of reasoning.

The four equivalent survivors: **`sarea < best_area` → `<=`** (boundary-traced
shells do not cross, so two containing the same point nest, and the inner has
strictly smaller area — equal areas force the same ring); **`d < 0.5` → `<=`**
(at 0.5 both sea-ramp branches give `SEA[1]`); **compensated `Math.hypot` →
naive** (≤ 2 ULP in `il`, quantised through `u8_clamped`, can move a byte only
within ~1e-15 of a `.5` tie; kept because it is what the reference computes);
**`tile_dims(sel, 1, 1, ts)` → `(sel, 2, 2, ts)`** (`aspect = (w/cols)/(h/rows)`
cancels when `cols == rows`; the asymmetric control `(2, 1)` is killed). Every
survivor was re-run in isolation, because a stale binary reports a healthy
`N passed`.

**Left out, deliberately:** the selection interaction and every UI surface
(`regionNewWorldBtn`, `refineBtn`, the download, the progress label, the
confirm) — F's; `burnChannels`; and `params.json`'s contents —
`zip_region_export` takes the bytes as a parameter because `serializeState()`
was a save writer this port did not yet have.

## Milestone F as built (2026-09-01)

The shell wiring, recorded after the fact against the working tree. It shipped
between 2026-08-18 (`7f5e54c`, the sculpt bindings; `611c5fa`, the six
remaining tool engines) and 2026-08-25, including the dispatch substrate, the
six-mode Measurement toolbar and the staleness readout. Two later owner rulings
moved where tools are armed — Ruling L (2026-09-13: picking a WORLD ▸ SCULPT
feature arms Sculpt) and Ruling AK (2026-09-23: the palette is a top bar) —
without changing any binding. `STRANDED_TOOLS.md`'s 2026-08-19 note that Sculpt
had become *"the template for the rest"* was condensed out of that file; it is
at `git show f90fe5a:STRANDED_TOOLS.md`.

**The claim in one line:** all sixteen tools `STRANDED_TOOLS.md` catalogued are
closed — thirteen with a real `cartalith-godot` binding behind a real shell
control, three that correctly need neither — with one tool the design added
that this port declines to bind (POI, by a milestone D decision) and one
honestly drawn loose end (Region's corner handles).

### The dispatch substrate

One mechanism arms and routes every tool, in `shell/app.gd` (`DccApp`): a single
`ButtonGroup` (`tool_group`) shared by every palette button, an `armed_tool`
string defaulting to `"inspect"`, and five `id → Callable` dictionaries —
`_click_handlers`, `_drag_handlers`, `_release_handlers`, `_escape_handlers`,
`_backspace_handlers` — that each workspace fills through
`register_tool_click_handler` and its four siblings. `_on_map_clicked`/
`_on_map_dragged`/`_on_map_released` call whichever entry matches
`armed_tool`, or do nothing for an id nobody registered — which is
Select/inspect's whole mechanism (`_wire_selection()` already is its
behaviour); Pan is a camera modifier, never armed (`global_tools.gd`'s header).
`arm_tool()` is the one place a tool becomes active and is workspace-agnostic:
no domain file sees another's tool, and there is no shared switch statement.

The palette is declared per domain through `DccWidgets.tools_block()`; off the
phone that hands the entries to `DccApp.set_domain_tools()`, and
`DccWidgets.tool_strip()` draws the four global cells plus the active domain's
own in the top bar (Ruling AK). `DccApp._strip_entries()` filters by mode,
which is how WORLD's Biome paint appears only in SCULPT mode (Ruling L).

**Twelve tool ids are registered** (`register_tool_*_handler` calls,
2026-09-24): `icon`, `label`, `measure`, `paint`, `region`, `route`, `sculpt`,
`settlement`, `territory`, `way` — this plan's tools — plus `territory_lasso`
(owner request, 2026-09-23) and `conflict` (`STORY_PLANNING_SCOPE.md`'s
conflict overlay). Sixteen catalogued tools become ten ids because five Sculpt
rows (Raise/lower, Smooth, Flatten/terrace, Stamp, River/water) share one
registry-backed id, as milestone B predicted, and Draw route/way was always two
reference tools (`draw_way`/`route`).

### Tool by tool, against `STRANDED_TOOLS.md`'s sixteen rows

| # | Tool | Tool id | Armed from | Draft held in (`WorldGen` field) | Drawn by |
|---|---|---|---|---|---|
| 1 | Select / inspect (`V`) | *(default; no handler)* | `armed_tool`'s default | n/a | the shell's own selection |
| 2 | Pan (`H`) | *(none; camera modifier)* | always active | n/a | `viewport_host.gd`'s camera |
| 3 | Point sample (`I`) | *(none; a readout)* | n/a | n/a | the right dock's Sample context |
| 4–8 | Raise/lower, Smooth, Flatten/terrace, Stamp, River/water | `sculpt` | picking a feature in WORLD ▸ SCULPT (`world_workspace.gd`, `arm_tool("sculpt")`) | `sculpt: Option<SculptEditor>` | `tool_overlay.gd` path preview + brush ring; `build_sculpt_preview_texture` composited live |
| 9 | Biome paint | `paint` | top bar in SCULPT mode (`world_workspace.gd`'s `tools_block` entry) | `paint: Option<PaintEditor>` | brush ring; `build_paint_preview_patch` |
| 10 | Place settlement | `settlement` | top bar, CIVIL (`civilization_workspace.gd::_build_tools`) | `civ_tools: Option<CivTools>` | Settlement inspector + map pin |
| 11 | Draw route / way | `way` + `route` | top bar, CIVIL (handlers in `infrastructure_workspace.gd`) | `infra: Option<InfraTools>` | `tool_overlay.gd` path preview while drafting; the right dock's Route/Way inspector once committed (`right_dock.gd::show_route`) |
| 12 | Territory / faction paint | `territory` (+ `territory_lasso`) | top bar, CIVIL | `civ_tools` (shared with row 10) | `territory_texture()` wash + brush ring |
| 13 | Label | `label` | top bar, CARTO (`cartography_workspace.gd`) | `labels: Option<LabelBridge>` | `map_overlay.gd::_draw_labels` once placed; `tool_overlay.gd` resize/rotate/arc handles while selected |
| 14 | Icon stamp | `icon` | top bar, CARTO | `icons: Option<IconEditor>` | `map_overlay.gd`'s icon draw; `tool_overlay.gd` resize handle |
| 15 | Measure | `measure` | top bar, global (`global_tools.gd`) | `infra` (shared with row 11) | `tool_overlay.gd` ruler / ring / A-B labels; the right dock, one context per mode |
| 16 | Region select / export | `region` | top bar, global | `infra` (shared with row 11) | `tool_overlay.gd` dashed marquee + corner handles; the right dock's Region summary (`right_dock.gd::_build_region`) |

Rows 1–3 never needed a binding and have none. Rows 4–16 have both halves: an
engine call and a shell control that reaches it.

### Rows 15 and 16 grew past what was proposed

- **Measure is six modes**: Distance, Bearing, Area, Radius, Cross-section and
  Δ vertical (`global_tools.gd`'s mode table), each reading one of the bound
  `measure_*` `#[func]`s (`measure_begin`/`add_point`/`result`/`clear`/
  `section`/`area`/`radius`/`vertical`) and drawn by shared `tool_overlay.gd`
  primitives rather than one routine per mode.
- **Region select is a closed loop.** `global_tools.gd` drags a rect into
  `region_set`; `right_dock.gd::_build_region` reads it back through
  `region_get` and shows extent, cell count and a per-LOD tile estimate (§4.5.1:
  *"Extent in both units, cell count, tile estimate per LOD, and Send to Data >
  Export"*); its **Send to Data ▸ Export** action opens the Data manager's
  export pane, which calls `region_export_tiles`. Marquee, readout and export
  are one path.

### The one tool the design added that this port declines to bind: POI

The design revision added a seventeenth control: POI (§4.5.3, `_civDropPOI`).
It has no tool id, by a decision that predates F.
`cartalith_civ::tools::civ_place_pick_weight`'s doc (milestone D): the
reference's *"POI branch (a flat weight of 5) is likewise absent because this
port has no POI concept"*; `civ_tools_bridge.rs`'s module doc: *"POI is not a
ported concept… This module therefore binds Settlement and Territory only"*;
and `civilization_workspace.gd::_build_tools` draws no POI button, because
arming a button with no engine behind it would be the fake control this port's
discipline exists to avoid. The standing rule working, not a backlog item.

### The staleness readout — SG-01, SG-02, SG-03

F's other half: *"Status-bar staleness readout… reading Milestone A's per-stage
`DirtyTracker`s."*

- `app.gd::_setup_staleness()` starts a one-second `Timer` onto
  `refresh_staleness()` rather than wiring a signal into every `#[func]` that
  can dirty something — a readout that is a plain query does not need six
  couplings.
- `refresh_staleness()` reads `stale_stages()` (over `WorldGen`'s
  `StageGraph`, A's type, unmodified), names the stale stages and the
  most-upstream reason over every stale tile, and writes the status bar's
  `stale` slot — **SG-01**.
- **SG-02** is the civ half: a sculpt or paint commit settles hydrology and
  climate but leaves civ stale until `recompute_civilisation`, which the graph
  alone cannot represent, so it is reported through a `civ_dirty` flag.
- **SG-03** is the per-*parameter* half: moving a generation dial marks the
  right stage stale — `set_params` marks the graph for the keys that have a
  live-apply path, and `params::invalidates` names the stage each invalidates.
  Marking only; a slider writes on every drag tick.
- The **Recompute** action beside the `stale` slot calls
  `recompute_stale_stages()`, shown only when the bound binary has that method —
  the degrade-rather-than-crash discipline every `engine_bridge.gd` wrapper
  uses.

### One honest residual

Region select's corner handles are drawn (`tool_overlay.gd`: *"drawn even though
resize-by-drag isn't wired yet, so the affordance reads correctly once it
is"*) and grabbing one does nothing: there is no `region_resize` `#[func]` — the
region verbs are `region_set`/`region_get`/`region_clear`/`region_export_tiles`.
Neither E nor E2 scoped a resize verb, only the drag-to-draw marquee; Label and
Icon got resize verbs (`label_resize_size`, `icon_resize`) because their
reference precedent had them. Redrawing a region means dragging a fresh
marquee, which reaches every other step correctly.

### Carried forward

Everything D, E and E2 left to F — waypoint capture, Escape-to-commit, the drag
rectangle and its overlay, the measure clicks, label drag/rotate/arc, icon
arm/disarm/resize — is built. Still outside every milestone here: `params.json`
in the region export (`region_export_tiles` passes `None`, so the exported
`.zip` has none; the no-save-writer reason no longer holds since `DECISIONS.md`
§7h), and `_civGenerateProvinces` over a painted territory raster (D's item).
