# Functional contract: the HTML app's behavior versus this port

Owner directive (2026-08-17→18, Fable 5 pass), governed by `DECISIONS.md` §7d
(just committed, `60cfb1b`): **behavior is the contract, not implementation.**
Every capability the HTML reference app describes must exist here and produce
an equivalent-or-better result. How it's built is free to differ — and where
QGIS, Mapbox, or the DCC-tool lineage this port's own shell now follows solve
the same job more effectively than a single-file browser app ever could,
their approach is *leading*.

This document is the master inventory that future parity work checks itself
against. It does not re-derive what's already been investigated carefully —
it cites and cross-references. Where this pass's own reading disagreed with
an existing doc, that's flagged explicitly, not silently overwritten.

## Live-repo version check

`https://github.com/Achos0190/Cartalith_RC`, root contents, 152 versioned
HTML files present, `v0.6` through **`v2.10`**. The frozen reference this
entire port is built and verified against (`reference/Cartalith Gen1
v2.10.html`) is the live repo's own latest version. **No drift, no re-freeze
question to raise.** This should be re-checked periodically as the upstream
project continues (this repo's own `CLAUDE.md` already says so) — it is not
a one-time fact.

## Method

Grounded in three layers, cross-checked against each other: (1) the frozen
reference's own real source (`reference/Cartalith Gen1 v2.10.html`,
`reference/FUNCTION_INDEX.md`) — the authoritative behavior definition; (2)
this repo's own accumulated scope docs, each the product of a real
investigation pass this session (`PHASE2_SCOPE.md`, `JOURNEY_PLANNER_SCOPE.md`,
`ASSET_LIBRARY_SCOPE.md`, `ECONOMY_SCOPE.md`, `UNIFIED_TOOL_PLAN.md`,
`GUI_FEATURE_PARITY_SCOPE.md`, `TERRAIN_APPEARANCE_SCOPE.md`,
`cartalith-native/docs/STATUS.md`) — consulted, not re-derived; (3) real,
current external research (QGIS/Mapbox documentation) for every modernize
recommendation, not assumed superiority.

> **Staleness addendum (2026-08-19, documentation audit).** Several "absent"
> statuses below were true when written and have since been overtaken by
> milestones landing on 2026-08-18: the Journey Planner is engine-complete
> (6/6, `JOURNEY_PLANNER_SCOPE.md` closing status), the Sculpt engine landed
> as tool-plan milestones B–E (`UNIFIED_TOOL_PLAN.md`), and region-tile
> export's compute/format core is done and golden-verified (milestone E2;
> `LOD_TILING_INTEGRATION_SCOPE.md` catalogues it as complete, UI panel
> pending the milestone-F hold). The rows are left as written — this document
> is checked against `cartalith-native/docs/STATUS.md`, which is
> authoritative; treat status cells here as of 2026-08-18 morning.

## Capability-by-capability contract

### 1. World generation

**HTML app**: `buildTectonicSubstrate()` → height formula → normalize →
volcanism/craters → world-structure archetypes (continentality/fragmentation/
tectonic-energy presets with sea-level histogram re-anchoring, v1.25) →
climate (temperature/wind/rainfall, ocean-current coupling v1.77-v1.82/v2.10)
→ erosion (droplet/stream-power/thermal) → hydrology (flow accumulation,
Strahler order, real-km channel width, v2.07). Deterministic from a seed.

**This port**: **done**, MVP criterion 1 — every stage golden-verified
bit-exact/tight-tolerance, including world-structure archetypes and ocean
currents (`MVP_SCOPE.md` had flagged ocean currents as a stretch goal; it
shipped). Sea level (`MVP_SCOPE.md` point 9) done as a real user control.

**§7d tag**: port as-is. The generation math is the contract's core and is
already verified against JS output directly — this is exactly the case §7d
says stays untouched ("nothing here re-litigates a verified stage").

### 2. Terrain editing (Sculpt)

**HTML app**: a real, complete, late-stage (v1.15+) feature
(`reference/Cartalith Gen1 v2.10.html` lines ~8837-9470, per
`UNIFIED_TOOL_PLAN.md`'s own fresh investigation) — 13 landform features
(Mountains, Hills, Ridge, Plateau, Cliff, Canyon, Valley, River, Lake, Basin,
Coastline, Volcano, Freehand with 8 sub-modes) plus 8 presets, each stamped
along a captured pointer stroke with a gauss-falloff brush (hardness/
intensity/noise-fBm-modulated, per-feature domain-warped edge noise). Real
draft/commit/discard model already: `sculptStamps[]` accumulates touching
nothing real, `sculptCommit()` bakes the stack plus exactly one
`computeFlow`/`refreshClimate` call, `sculptDiscard()` drops it.

**This port**: **absent**, explicitly (`MVP_SCOPE.md`'s own "Out of scope"
table: "Sculpt editor | block 1"). `UNIFIED_TOOL_PLAN.md` (just landed) is
the first real investigation and the scoping document for building it —
found the reference's own commit/discard model is the direct ancestor of the
DCC shell mockup's "pass buffer" language, so this isn't invented UX, it's a
real feature waiting to be ported with a Rust-native storage layer.

**§7d tag**: **port behavior, modernize implementation.** The brush
model/feature registry/commit-discard semantics are real and worth porting
faithfully. The storage substrate should not be the reference's direct
field-array mutation — `cartalith-spatial`'s `TiledField`/`QuadTree`/
`DirtyTracker` (built this session, unintegrated, exactly for this trigger)
are the modern answer: tile-granular dirty tracking instead of whole-field
copies for undo, a real pass-buffer type layered on top (per
`UNIFIED_TOOL_PLAN.md`'s own design). This is the single largest unbuilt
capability in the whole contract — see the milestone plan already written.

### 3. Hydrology / climate / ecology display and control

**HTML app**: real-time parameter sliders for erosion/climate/hydrology
tuning, river density and minimum stream order controls, biome ecotone
detail toggle, lakes-as-water/rivers-as-ways toggles.

**This port**: generation-side **done** (part of criterion 1's parity).
Interactive re-tuning without a full regenerate is **absent** — every
parameter change today means a fresh `generate()` call, not a live
recompute. `UNIFIED_TOOL_PLAN.md`'s staleness section addresses this
directly for the *editing* case (terrain sculpt marking hydrology stale) but
pure parameter re-tuning without a brush stroke isn't scoped anywhere yet.

**§7d tag**: port behavior (users must be able to change a generation
parameter and see the effect), implementation open — this port's one-shot
generate model is itself already `DECISIONS.md`-grounded
(`HARDWARE_ACCELERATION.md`'s static-generation correction); whether
per-parameter re-generation becomes interactive is a real design question
for whoever picks this up, not decided here.

### 4. Civilisation (settlements, factions, roads, territory, provinces, villages, economy, statistics)

**HTML app**: `_civIterativeAutoWorld` orchestration — settlement placement/
faction assignment, population/naming, road network (`_civHierarchicalNetwork`
+ MST + sea-lane routes), territory (both an interactive paint tool,
`_civPaintTerritoryAt`, **and** an algorithmic auto-generator,
`_civAutoPolity` — corrected 2026-08-19, see `DECISIONS.md` §7b's own
correction notice; an earlier audit of this section's claim missed it),
province sub-partitioning, village seeding, faction economy aggregates
and trade, statistics reporting. Also present and untracked by this port
anywhere until now: a **Timeline/collapse-recovery simulation** layer
(`civAddYear`/`civGotoYear`, `_civSimulateTimeline`/`_civCollapseStep`/
`_civRecoveryGrowthStep`/`_civRunCollapseSimulation`, per-year territory
snapshots, timeline playback — reference lines ~20597-26478) with its own
design grounding in RC's vendored `docs/research/collapse-timeline-
dynamics.md` and `settlement-emergence.md` §5-6.

**This port**: **done**, 19 Phase 2 milestones (`PHASE2_SCOPE.md`), each
golden-verified against the real reference where an execution path exists.
One disclosed divergence, corrected 2026-08-19: territory assignment is
this port's own cost-distance Voronoi design (`DECISIONS.md` §7b) —
**not** a from-scratch invention as previously claimed here, since the
reference does have `_civAutoPolity`, but a deliberately different
algorithm (capital-seeded + population-weighted, vs. the reference's
all-settlement-seeded + unweighted + reach-capped), never reconciled
against it. Economy (`civ_resource_trade_balance`/`get_trade_balances()`)
and culture-terrain-fit are ported and wired. **Absent**: the Timeline/
collapse-recovery simulation layer named above — zero of it exists in this
port (no year snapshots, no collapse/recovery step functions, no
timeline UI) — untracked anywhere until this correction pass.

**§7d tag**: port as-is for everything with real reference precedent
(settlements/roads/population/naming/provinces/villages) — already the
contract's most thoroughly verified layer. Territory now has real
reference precedent too (`_civAutoPolity`) that it was never checked
against — **resolved 2026-08-19 (owner decision)**: the current design
stays as the only mode, un-reconciled, closed (`DECISIONS.md` §7b).
Timeline/collapse is **approved for build, 2026-08-19 (owner decision)**
— see `TIMELINE_SCOPE.md` once scoped.

### 5. Journey Planner

**HTML app**: ~70 `jp*`/`_jp*` functions (`ECONOMY_SCOPE.md`'s own count) —
transport-mode selection, physical travel cost (train pace, sail polar,
weather-weighted cost), consumption/resupply, seasonal/mountain-pass
closures, route/stage derivation, verdict/reporting. A real, interactive,
per-journey planning tool the reference's own UI exposes as a form.

**This port**: **3 of 6 milestones done** (`JOURNEY_PLANNER_SCOPE.md`) —
physical-modeling primitives, seasonal closures, transport-mode selection (6
of 10, 4 correctly deferred on real unbuilt dependencies), physical travel
cost (7 of 11, 2 correctly deferred). Route/stage derivation (milestone 5,
flagged as likely the largest single piece) and verdict/reporting (milestone
6) are **absent**. Nothing wired to any UI — deliberately, per that doc's
own "out of scope" boundary: this is real interactive per-journey tooling,
not something auto-computed for every settlement pair.

**§7d tag**: port as-is. This is deep, historically-grounded domain logic
(real v1.27/v1.50/v1.52/v1.63/v1.97/v1.98 fixes cited throughout) with no
obvious modernization target — it's a cost-model calculator, not a rendering
or data-management problem QGIS/Mapbox solve differently.

### 6. Map rendering

**HTML app**: `materialWeights` biome/hillshade default render, six
climate-selected colour ramps, `state.viz.*`-gated stretch layer: splat
texturing, geology microtexture, NPR "Painter" styles (contour veins, ink
linework, hachure, watercolor, cel/toon, engraving, stipple, sepia/antique,
risograph, pointillism — `design/cartalith-menu-structure.md`'s own §5
inventory of these, cross-checked against the design import), AO/SVF/shadows,
multi-sun, SDF coast/river/biome tinting, contour intervals, a real tiled-LOD
deep-zoom system (`bakeAllTiles`, line 10809) and region-tile export
(`exportRegionTiles`, line 11891).

**This port**: default biome/hillshade render **done** (MVP criterion 2,
golden-verified). Phase 3 (`TERRAIN_APPEARANCE_SCOPE.md`) has landed 4
milestones past the default: multidirectional hillshade + AO, hydrology
tint, the atlas look (paper ground, forest stippling, plate border) — all
gated behind a `js_reference()` no-op so the original JS-parity test stays
exact, all real deliberate improvements past the reference under §7a/§7d's
own carve-out. **Splat texturing is real** — Asset Library milestone 7 wired
ground-texture splat into the already-golden `materialWeights` blend. NPR
Painter styles, geology microtexture, AO/SVF/shadow toggles as *user
controls* (the render effects exist internally in the reference; exposing
them as switchable options doesn't yet), SDF tinting, contour intervals,
and the entire tile pyramid/LOD/region-export system are **absent**.

**§7d tag**: mixed, itemized:
- Default render, atlas look, splat: port as-is / already modernized (the
  atlas look and hillshade/AO are genuinely *better* than the reference's
  own default, not merely equivalent).
- NPR Painter styles, geology microtexture toggle: port as-is — these are
  presentation choices with no efficiency problem to modernize, just unbuilt.
- **Tile pyramid / LOD / region export: port behavior, modernize
  implementation.** This is the contract's clearest §7d case. The
  reference's own tiled-LOD system exists because a single `<canvas>` could
  not hold a 20,000km world at full resolution — a browser-memory
  workaround, not a design goal. Mapbox GL's real architecture (verified via
  current documentation) is the leading answer: tiles form a quadtree
  pyramid, `2^zoom × 2^zoom` grid per level, geometry simplified at lower
  zoom so detail cost scales with what's actually visible, not the whole
  world. `cartalith-spatial`'s `QuadTree<T>`/`TiledField<T>` (built this
  session, unintegrated) are already shaped for exactly this — this is a
  second real trigger for integrating it, alongside the Sculpt editor.

### 7. Labels, annotation, icons

**HTML app**: region-name labels, manual icon placement (category/density
brush/radius), a paint-brush layer for biome/terrain "painted layers"
(distinct from generation), scale bar, a measurement tool.

**This port**: **rule-driven** icon/label placement is done
(`ASSET_LIBRARY_SCOPE.md` milestone 4/7 — `place_map_icons_ruled`, real
splat/glyph compositing). **Manual, interactive** placement (a user clicking
to drop a label or icon, or painting biome/terrain overrides by hand) is
**absent** — `ASSET_LIBRARY_SCOPE.md` milestone 7 named this explicitly as a
follow-up rather than a silent gap, and `UNIFIED_TOOL_PLAN.md`'s Annotation &
Measure group (Label, Icon stamp, Measure, Region select/export) covers
exactly this: Label and Region export have rich reference precedent and
zero Rust implementation; Measure has no reference precedent at all and
needs almost none to build fresh.

**§7d tag**: port as-is for the interaction model (click-to-place is
correct here, not a case for modernization); the biome/terrain "painted
layer" override mechanism is real unbuilt behavior with a clear reference
precedent (`paintBiome` — an override layer that wins over `classify_biome`'s
output per-cell, not a mutation of the classifier itself, per
`UNIFIED_TOOL_PLAN.md`'s own finding).

### 8. Asset library

**HTML app**: a full asset-pack authoring workspace (library browser,
sprite-sheet slicer with canvas/pointer interaction, tag/collect/rename/
duplicate, pack validation with a real hardening history: v1.27 NaN/aliasing
fixes).

**This port**: **Phase 4 complete**, all 7 milestones (`ASSET_LIBRARY_SCOPE.md`)
— manifest model, ZIP read/write (round-tripped through the reference's own
export/import code, not a synthetic fixture), scatter rules (hardening
re-derived correctly for Rust's own failure modes, not transcribed), placement
(diffs exactly to 1e-9), the Library model (`AssetDB`/`AssetValidator`, two
undocumented hardening behaviors found and ported), image handling, and
renderer integration (sprite compositing + splat, real pixel-verified). The
one honest carve-out: the **Library-authoring workspace UI** — the sprite-
sheet slicer, browse/tag/collect UI — is real editor-application UI with no
engine logic behind it, correctly deferred to `GUI_SHELL_SCOPE.md`'s own
Assets menu work rather than attempted as data-layer porting.

**§7d tag**: port as-is for the data layer (done, correctly). The
authoring-UI question is itself a §7d case worth deciding deliberately when
it's picked up: a canvas-based pixel slicer built fresh in Godot Control
nodes is real UI work either way; nothing here suggests a better external
model to copy (this isn't a cartographic problem QGIS/Mapbox solve).

### 9. Import / export

**HTML app**: load heightmap, infer tectonics from heightmap, import asset
pack, export image/tiles (`bakeAllTiles`, `exportRegionTiles`), export
GeoJSON (`exportGeoJSON`, line 12576), export `.zip` (`exportZip`, line
12418), export region.

**This port**: reading a real HTML-app `.zip` export is **done** (MVP
criterion 7, verified against a real reference-produced export, not a
synthetic one). Everything else — writing `.zip`, GeoJSON export, tile/image
export, heightmap import, tectonics inference — is **absent**, and was
explicitly out of MVP scope by design (`MVP_SCOPE.md`: "Writing saves is
out... Point 12 grants reading one specific thing; it is not a general
save/load licence"). `GUI_FEATURE_PARITY_SCOPE.md`'s own audit already found
these sitting as `disabled` menu items in the current shell — present,
honest, unwired.

**§7d tag**: port behavior as-is for GeoJSON/heightmap-import (standard
formats, no modernization angle). **Tile/image export is the same §7d case
as capability 6's tile pyramid** — the reference bakes a fixed-size PNG
because that's what a browser can produce; a real tile-server-style export
(the same quadtree structure Mapbox's own MTS pipeline uses for tileset
generation) is the leading approach once the underlying LOD/tiling work
lands, rather than porting the reference's own flat-bake function verbatim.

### 10. Save / load

**HTML app**: `.zip` save/load with a documented format (`SAVEFILE_COMPAT.md`
already reverse-engineers this against live reference code, not guessed).

**This port**: reading **done** (criterion 7). Writing **absent**,
deliberately deferred (`SAVEFILE_COMPAT.md`, `MVP_SCOPE.md`).

**§7d tag**: port as-is. This is a well-specified binary format with a
correctness bar (byte-for-byte compatibility with reference-produced files,
already proven on the read side); there's no "QGIS does this better"
question — bring the same rigor to the write side when it's picked up.

### 11. View modes (2D/3D, LOD, analysis fields)

**HTML app**: 2D map (default), 3D terrain view (`#viewDimSeg`), tiled LOD
view with auto-detail-on-zoom, a real analysis-field switcher (elevation,
slope, aspect, curvature, flow accumulation, drainage, temperature, rainfall,
wind, ocean currents, soil, lithology, biome — `#debugSeg`, no direct
function-index hits found this pass, confirm the exact control id before
building against it).

**This port**: 2D **done**. 3D **absent, deliberately** (`DECISIONS.md` §4,
"cutting it keeps the first milestone achievable... `ROADMAP.md` Phase 3
brings it back" — Phase 3 has landed 2D fidelity work but not the 3D drape
itself yet). LOD **absent**, foundation built and deliberately unintegrated
(`cartalith-spatial`, `LOD_TILING_BASE_SCOPE.md`). Analysis-field switching:
`GUI_FEATURE_PARITY_SCOPE.md`'s own audit already flagged this as
"ambiguous, verify before building" — `render.rs` computes these fields
internally per-pixel but may not expose them as independently-selectable
output channels; not resolved by this pass either, still a real open
question for whoever wires it.

**§7d tag**: 2D — port as-is, already done well. 3D — this is `DECISIONS.md`'s
own deferred scope, not a gap this document is re-opening; when it lands,
evaluating `Terrain3D`/`godot_heightmap_plugin` (`ROADMAP.md`'s own note) is
already the planned modernize-over-port path. LOD — **modernize by
construction**: `cartalith-spatial` was never going to be a port of the
reference's own tile-baking code, it's a from-scratch packed quadtree
designed for this engine. Analysis fields — port as-is once the real
`#debugSeg` control and its field list are confirmed against the reference.

### 12. Session features (undo, theme, credits)

**HTML app**: `#undoBtn`/`#undoMem` (real undo with a memory budget), theme
toggle (dark/light), credits modal.

**This port**: theme **done**, dark-first (`GUI_SHELL_SCOPE.md`'s
decluttering pass — a real `Theme` resource, not a CSS variable swap; light
theme itself still deferred as its own milestone). Credits **done**, real
and reachable, carrying forward the reference's own attribution plus this
port's own license-audit findings. Undo is **absent entirely** — no
generation-parameter undo exists, and the DCC shell's own editing model
(`UI_SHELL_DESIGN.md`: "Undo granularity is one committed pass, not one
stroke") makes real undo a load-bearing part of the tool-system work
(`UNIFIED_TOOL_PLAN.md` milestone A, pass-buffer/staleness core) rather than
a separate feature.

**§7d tag**: theme/credits — already done, arguably already better (a real
Theme resource composits more consistently than the reference's own CSS
custom-property approach, though this is a minor case, not a headline
recommendation). Undo — port behavior, and its implementation is
necessarily new regardless of §7d, since it has to integrate with a tool
system the reference's single-canvas architecture never needed to reconcile
with a native undo stack.

## Summary coverage table

| Capability | Status | §7d tag |
|---|---|---|
| World generation | Done | Port as-is |
| Terrain editing (Sculpt) | Absent, scoped | Port behavior, modernize storage |
| Hydrology/climate/ecology live tuning | Absent | Open design question |
| Civilisation | Done | Port as-is (territory: modernized by necessity) |
| Journey Planner | 3/6 milestones | Port as-is |
| Map rendering (default + atlas) | Done, improved | Already modernized |
| Map rendering (NPR/geology/SDF toggles) | Absent | Port as-is |
| Tile pyramid / LOD / region export | Absent, foundation built | Modernize (Mapbox-style quadtree) |
| Labels/annotation (rule-driven) | Done | Port as-is |
| Labels/annotation (manual tools) | Absent, scoped | Port as-is |
| Asset library (data layer) | Done, Phase 4 complete | Port as-is |
| Asset library (authoring UI) | Absent | Deliberate future UI work |
| Import (GeoJSON/heightmap) | Absent | Port as-is |
| Export (tiles/image) | Absent | Modernize (tile-server-style) |
| Save (read) | Done | Port as-is |
| Save (write) | Absent | Port as-is |
| 2D rendering | Done | Port as-is |
| 3D view | Absent, deferred by decision | Modernize when built (`Terrain3D`) |
| Analysis-field switching | Ambiguous, unresolved | Port as-is once confirmed |
| Theme | Done | Already modernized (minor) |
| Credits | Done | Port as-is |
| Undo | Absent | New implementation, necessarily |

## Honest absent-entirely list, with real size

- **Terrain Sculpt editor**: real, scoped (`UNIFIED_TOOL_PLAN.md`), the
  single largest item — comparable to Journey Planner/Asset Library.
- **Journey Planner route derivation + verdict**: 2 of 6 milestones,
  milestone 5 flagged as likely the largest single piece in that plan.
- **Manual annotation/labeling/measurement tools**: scoped
  (`UNIFIED_TOOL_PLAN.md` group 5), small-to-medium once the pass-buffer
  core exists.
- **Tile pyramid / LOD / deep-zoom / region export**: foundation exists
  (`cartalith-spatial`) and unintegrated by deliberate choice
  (`LOD_TILING_BASE_SCOPE.md`) pending a concrete trigger — this document
  and capability 6 above are that trigger, now recorded.
- **Save writing, GeoJSON/tile/image export, heightmap import**: real but
  each individually small-to-medium; none scoped yet.
- **NPR Painter styles, geology microtexture toggle, SDF tinting, contour
  intervals**: presentation-only, no engine dependency, unscoped.
- **Undo**: absent, tied to the tool-system work.
- **Asset library authoring UI**: real UI-only work, `GUI_SHELL_SCOPE.md`'s
  job when picked up.
- **Live parameter re-tuning without full regenerate**: open design
  question, not yet even scoped as a milestone target.

## Top three modernize-over-port recommendations

**1. Tile pyramid / LOD / deep-zoom rendering.** Leading example: **Mapbox
GL's tile pyramid** — tiles as a quadtree, `2^zoom × 2^zoom` per level,
geometry/detail simplified at lower zoom so rendering cost scales with what's
visible rather than total world size. The reference's own tiled-LOD system
exists because a browser `<canvas>` had no other way to hold a 20,000km
world; that constraint doesn't apply here. `cartalith-spatial`'s
`QuadTree<T>` and `TiledField<T>` (real, tested, built this session,
deliberately unintegrated pending "a concrete need") already have the right
shape for this. This document is that concrete need, recorded.

**2. Layer compositing (opacity/blend-mode/reorder for territory,
provinces, settlements, overlays).** Leading example: **QGIS's raster
symbology model** — 13 real blend modes (Normal, Multiply, Screen, Overlay,
Darken, Lighten, etc.), per-layer opacity independent of blend, group
opacity applied after child flattening rather than per-layer. Cartalith's
current renderer bakes every layer into one `render.rs` pass per pixel —
`GUI_FEATURE_PARITY_SCOPE.md`'s own research already reached the right
conclusion here: build opacity now (textures already carry alpha, cheap),
defer blend-mode/reorder until layers are actually separable compositing
targets rather than baked contributions to one pass. Confirmed, not
contradicted, by this pass's own research.

**3. Terrain-editing storage (Sculpt/pass-buffer/undo).** Leading example:
the DCC-tool lineage this port's own shell now follows (Photoshop/Blender-
style non-destructive editing) — a committed history of discrete edits over
a tiled field, not whole-canvas snapshots. `cartalith-spatial`'s
`DirtyTracker` (per-tile dirty flag + monotonic version counter) is the
right primitive; `UNIFIED_TOOL_PLAN.md` already specifies the
`PassBuffer<Stamp>` type layered on top, modeled directly on the reference's
own `sculptStamps[]`/commit/discard semantics — the behavior ports, the
storage modernizes.

## Disagreements with prior docs

None found. This pass's own reading of `reference/Cartalith Gen1 v2.10.html`
(Sculpt editor lines, export function line numbers) confirmed rather than
contradicted `UNIFIED_TOOL_PLAN.md`'s and `MVP_SCOPE.md`'s existing claims.
One gap not previously flagged anywhere: interactive re-tuning of generation
parameters without a full regenerate (capability 3) has no scope document
and no recorded design decision — noted here as newly surfaced, not urgent,
since the one-shot generation model is itself a deliberate, documented
choice (`HARDWARE_ACCELERATION.md`) that this capability would need to be
reconciled with, not silently assumed away.
