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

> **Cross-reference addendum (2026-08-23, legacy control/function checklist).**
> `reference/FUNCTION_INDEX.md` has been rewritten from a full end-to-end analyst
> read of the frozen v2.10 file: it now carries a complete user-facing control
> checklist (Part 0 — every button, slider, toggle, dialog, keyboard shortcut and
> canvas interaction, each linked to its backing function) and a one-line purpose
> for all 1094 indexed functions (Part 1). For fine-grained "does the port cover
> control X / function Y" checks, walk that checklist; this document remains the
> capability-level contract and does not duplicate it.

> **Corrected 2026-08-23 (`PARITY_AUDIT.md` C2/§7).** The staleness addendum
> below was written 2026-08-19 and, true to its own name, itself went stale:
> it said "the rows are left as written" and by 2026-08-23 ten of the summary
> table's rows plus capability 4's body no longer matched real code. Every
> row and body paragraph flagged by `PARITY_AUDIT.md` §7's C2 detail table
> has now been corrected in place, each citing the audit's file:line
> evidence, re-verified against the repository rather than taken on the
> audit's word. This addendum is left here as the historical record of when
> and why the drift happened; the capability sections below are current as
> of 2026-08-23.

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

**This port**: **built** (corrected 2026-08-23, `PARITY_AUDIT.md` C2). What
was absent when `MVP_SCOPE.md` scoped it out is now real: `UNIFIED_TOOL_PLAN.md`
milestones B–E landed the brush model, feature registry and commit/discard
semantics as `cartalith-terrain/src/sculpt.rs` plus `sculpt_bridge.rs` in
`cartalith-godot`, with roughly 30 `#[func]` bindings exposing it to Godot
and real draft-scoped undo/redo (`sculpt_undo`/`sculpt_redo`,
`crates/cartalith-godot/src/lib.rs:3403,3411`). Found the reference's own
commit/discard model is the direct ancestor of the DCC shell mockup's "pass
buffer" language, so this wasn't invented UX, it was a real feature ported
with a Rust-native storage layer, as planned below.

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
and culture-terrain-fit are ported and wired. **Corrected 2026-08-23**
(`PARITY_AUDIT.md` C2): the Timeline/collapse-recovery simulation layer
named above is **fully built**, not absent — `cartalith-civ/src/timeline.rs`
plus `timeline_bridge.rs` in `cartalith-godot`, with real
`civ_add_year`/`civ_goto_year`/`civ_year_diff`/`civ_run_collapse_simulation`
bindings (`crates/cartalith-godot/src/lib.rs:240,261` and neighbouring),
per-year territory snapshots and timeline playback. The "zero of it exists"
claim was true when first written and had gone stale by the time this
document's own summary table still carried it.

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

**This port**: **6 of 6 milestones done, engine-complete** (corrected
2026-08-23, `PARITY_AUDIT.md` C2/§3.3 — `JOURNEY_PLANNER_SCOPE.md`'s own
closing status): 65 of the reference's 74 `jp*` functions ported (6 UI-only,
2 JS idioms, 1 formerly blocked and since ported), with a real Godot
takeover view, the Travel Library, party set-ups, the timeline band and
blocked-stage inline resolutions all wired. Remaining gaps are individually
registered rather than milestone-sized:
`jpAutoPickTransport`/`_jpRerouteForMode` (JP-01/JP-03),
`jp_journey_cost` ported but never called (JP-04, `GUI_GAP_REGISTER.md`
calls it "the single cheapest (B) in the register"), the calculation-trace
window (JP-05), journey save/registry (JP-06/JP-08), ⇧-drag spine trim
(JP-07), and the vessel sailing window/resolver (JP-09/IN-06).

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
them as switchable options doesn't yet), and SDF tinting/contour intervals
are **absent**. **Corrected 2026-08-23** (`PARITY_AUDIT.md` C2): the tile
pyramid/LOD/region-export system is **not** entirely absent — deep-zoom LOD
tile synthesis is live and region-tile export is golden-verified and wired
to a real route; only the persistent atlas/tile cache and the
bake/finalize-lock remain unbuilt. See this capability's §7d tag below for
the detail.

**§7d tag**: mixed, itemized:
- Default render, atlas look, splat: port as-is / already modernized (the
  atlas look and hillshade/AO are genuinely *better* than the reference's
  own default, not merely equivalent).
- NPR Painter styles, geology microtexture toggle: port as-is — these are
  presentation choices with no efficiency problem to modernize, just unbuilt.
- **Tile pyramid / LOD / region export: mostly landed, corrected 2026-08-23**
  (`PARITY_AUDIT.md` C2/§3.1). Deep-zoom LOD **tile synthesis is live**
  (`lod_bridge.rs`, `lod_synthesize_tile`/`lod_tile_cells`, driven
  automatically by `viewport_host.gd`), and **region-tile export is
  golden-verified and wired to a real route**
  (`crates/cartalith-godot/src/lib.rs:4632` `region_export_tiles`, Data
  manager's Export ▸ Maps pane, DM-13). Genuinely still absent: the
  *persistent* atlas/tile cache and the bake/finalize-lock. The reference's
  own tiled-LOD system exists because a single `<canvas>` could not hold a
  20,000km world at full resolution — a browser-memory workaround, not a
  design goal. Mapbox GL's real architecture (verified via current
  documentation) is the leading answer for what remains: tiles form a
  quadtree pyramid, `2^zoom × 2^zoom` grid per level, geometry simplified at
  lower zoom so detail cost scales with what's actually visible, not the
  whole world. `cartalith-spatial`'s `QuadTree<T>`/`TiledField<T>` are
  already shaped for exactly this and are now real consumers of it (the
  LOD tiles), not unintegrated.

### 7. Labels, annotation, icons

**HTML app**: region-name labels, manual icon placement (category/density
brush/radius), a paint-brush layer for biome/terrain "painted layers"
(distinct from generation), scale bar, a measurement tool.

**This port**: **rule-driven** icon/label placement is done
(`ASSET_LIBRARY_SCOPE.md` milestone 4/7 — `place_map_icons_ruled`, real
splat/glyph compositing). **Manual, interactive** placement is **built**
(corrected 2026-08-23, `PARITY_AUDIT.md` C2): `label_bridge.rs`,
`icon_bridge.rs`, `paint_bridge.rs` and the `measure_*` functions
(`crates/cartalith-godot/src/lib.rs`, `infra_tools_bridge.rs`) are all
bound and live — `UNIFIED_TOOL_PLAN.md`'s Annotation & Measure group
(Label, Icon stamp, Measure, Region select/export) landed. The
biome/terrain/splat "painted layer" tool (`UNIFIED_TOOL_PLAN.md` milestone
F) is **also built and wired**, not absent as capability 8 below still
claimed until this pass — `paint_bridge.rs`'s `paint_set_layer`/
`paint_set_brush`/`paint_stroke_at`/`paint_commit`/`paint_discard`
(`crates/cartalith-godot/src/lib.rs:4013-4210`) are armed as a real "Biome
paint (B)" tool in `world_workspace.gd`, click/drag/release handlers and
all (this was found independently of the audit's own list — see the final
report). What remains open, per the register: label/icon on-canvas resize
handles are inconsistent (`icon_bridge.rs` has none, `label_bridge.rs` does
— CA-05), and paint's hardness/softness sliders are stored and echoed back
but never consumed — painting is still a hard disc with no soft falloff
(WW-06).

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
**Library-authoring workspace UI is also built** (corrected 2026-08-23,
`PARITY_AUDIT.md` C2/§3.5) — `asset_library_window.gd` (2,709 lines) is a
real sprite-sheet slicer with pointer interaction, drag/drop, batch tag/
collect/rename/duplicate, and pack validation, rebuilt against the design
canvas per `GUI_SHELL_SCOPE.md`. Open items are individually registered
rather than a missing workspace: "Unassigned imports" (AS-12), per-
interior-line dragging and cell-scoped slicing (AS-17), and
`as_set_item_transform` for slot scale/pan editing (AS-07). Note also: the
two Cartography "painted layer" biome/terrain overrides this section
previously said were "honestly left out" are themselves built — see
capability 7 above.

**§7d tag**: port as-is for the data layer (done, correctly) and for the
authoring UI (also done — a canvas-based pixel slicer built fresh in Godot
Control nodes was real UI work either way; nothing suggested a better
external model to copy, since this isn't a cartographic problem QGIS/Mapbox
solve).

### 9. Import / export

**HTML app**: load heightmap, infer tectonics from heightmap, import asset
pack, export image/tiles (`bakeAllTiles`, `exportRegionTiles`), export
GeoJSON (`exportGeoJSON`, line 12576), export `.zip` (`exportZip`, line
12418), export region.

**This port**: reading a real HTML-app `.zip` export is **done** (MVP
criterion 7, verified against a real reference-produced export, not a
synthetic one). **Corrected 2026-08-23** (`PARITY_AUDIT.md` C2): several of
these are no longer absent. **Heightmap import and tectonics inference are
done and bit-exact** (`cartalith-terrain/src/infer.rs` +
`cartalith-engine/src/import.rs`, live at `Data ▸ Import ▸ Heightmaps
(PNG)`). **Tile/image export is live end to end** (`region_export_tiles`,
wired to the Data manager's Export ▸ Maps route, DM-13) — what remains of
that row is the slippy-map-addressing half (XYZ/TMS/WMTS, a zoom ladder,
retina variants), not tile export itself. **GeoJSON export** is a genuinely
mixed case worth stating precisely: `cartalith-engine/src/geojson.rs` is
done and golden-verified (nine reference functions ported), but it has
**no `#[func]` binding**, so it does not cross the Rust↔Godot boundary yet
— engine done, boundary not (DM-03, "(B) wrapper"). Still genuinely absent:
writing `.zip`, and GeoJSON **import**. All of this was explicitly out of
MVP scope by design at the time (`MVP_SCOPE.md`: "Writing saves is out...
Point 12 grants reading one specific thing; it is not a general save/load
licence").

**§7d tag**: port behavior as-is for GeoJSON/heightmap-import (standard
formats, no modernization angle) — heightmap import is now done under that
tag. **Tile/image export is the same §7d case as capability 6's tile
pyramid**, now mostly landed — the reference bakes a fixed-size PNG because
that's what a browser can produce; a real tile-server-style export (the
same quadtree structure Mapbox's own MTS pipeline uses for tileset
generation) remains the leading approach for the slippy-map-addressing
remainder, rather than porting the reference's own flat-bake function
verbatim.

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
itself yet). **Corrected 2026-08-23** (`PARITY_AUDIT.md` C2): LOD is **no
longer absent** — deep-zoom tile synthesis is live and automatic
(`lod_bridge.rs`, `viewport_host.gd`'s `_lod_backlog`), `cartalith-spatial`
has real consumers now (`PassBuffer`/`StageGraph`, then the LOD tiles); only
the persistent on-disk atlas/cache remains unbuilt. **Analysis-field
switching is resolved**, not ambiguous: `sample_bridge.rs` exposes 18 live
debug views plus 11 more with stated reasons for their absence — the
"ambiguous, verify before building" flag from `GUI_FEATURE_PARITY_SCOPE.md`
has been answered.

**§7d tag**: 2D — port as-is, already done well. 3D — this is `DECISIONS.md`'s
own deferred scope, not a gap this document is re-opening; when it lands,
evaluating `Terrain3D`/`godot_heightmap_plugin` (`ROADMAP.md`'s own note) is
already the planned modernize-over-port path. LOD — **modernize by
construction, and now real**: `cartalith-spatial` was never going to be a
port of the reference's own tile-baking code, it's a from-scratch packed
quadtree designed for this engine, and it is wired in. Analysis fields —
port as-is; the real `#debugSeg`-equivalent surface and its field list are
confirmed and live.

### 12. Session features (undo, theme, credits)

**HTML app**: `#undoBtn`/`#undoMem` (real undo with a memory budget), theme
toggle (dark/light), credits modal.

**This port**: theme **done**, dark-first (`GUI_SHELL_SCOPE.md`'s
decluttering pass — a real `Theme` resource, not a CSS variable swap; light
theme itself has since also shipped, PR-13/PR-14 done 2026-08-19). Credits
**done**, real and reachable, carrying forward the reference's own
attribution plus this port's own license-audit findings. **Corrected
2026-08-23** (`PARITY_AUDIT.md` C2): undo is **not absent entirely** —
draft-scoped undo/redo is real (`sculpt_undo`/`sculpt_redo`,
`cartalith-spatial/src/pass.rs`'s `PassBuffer`), wired in `right_dock.gd`.
Only **global**, generation-parameter-level undo is absent, and the DCC
shell's own editing model (`UI_SHELL_DESIGN.md`: "Undo granularity is one
committed pass, not one stroke") still makes that a load-bearing part of the
tool-system work (`UNIFIED_TOOL_PLAN.md` milestone A) rather than a separate
feature.

**§7d tag**: theme/credits — already done, arguably already better (a real
Theme resource composits more consistently than the reference's own CSS
custom-property approach, though this is a minor case, not a headline
recommendation). Undo — port behavior for the remaining global case; draft
undo's implementation was necessarily new regardless of §7d, since it had
to integrate with a tool system the reference's single-canvas architecture
never needed to reconcile with a native undo stack.

### 13. Urban morphology (town/city internal layout)

**Added 2026-08-23** (`PARITY_AUDIT.md` C3): this capability had no row in
this document at all — a real gap, not a stale one, for what `README.md`/
`STATUS.md` themselves already call "the largest single unported subsystem."

**HTML app**: town/city internal layout generation — radial (Venus) streets,
plaza and waterway placement, water infrastructure, fortification, graph
cleanup, block/parcel subdivision, district and building placement,
amenities, hinterland/decay/detail/metrics, a `generate()` orchestration
plus `hashModel`, and a 28-function civ-layer adapter (Part 1 block 4 of
`FUNCTION_INDEX.md`, ~92 functions, plus the block-2 `_um*` adapter). On the
user-facing side: town layouts drawn on the map at deep zoom
(`civUrbanLayoutsChk`), a City Viewer modal with its own zoom/pan
(`cityViewerModal`), and a layout thumbnail in the place-edit popup
(`peCityPreview`/`peCityOpen`).

**This port**: **milestones 1-7 of ~17 done**
(`URBAN_MORPHOLOGY_SCOPE.md`) — RNG substreams, a geometry kernel using
`js_hypot`, the planar street graph, A\* over the cost raster, generation
rules + culture profiles, the site model, anchors and primary routes, and
organic growth, each with its own golden-test suite (4,516 lines of
`cartalith-urban` source). **It has zero consumers**, verified directly for
this correction (`grep -rn 'cartalith-urban' crates/*/Cargo.toml` returns
only its own manifest; `cartalith-godot/Cargo.toml` does not depend on it;
the only mention anywhere under `godot-project/` is one disclosure comment
in `civilization_workspace.gd:490-491` naming it as unported). Milestones
8-17 (radial streets/plaza/waterway, water infrastructure, fortification,
graph cleanup, blocks/parcels, districts/buildings, amenities, hinterland/
decay/details/metrics, `generate()`/`hashModel`, and the 28-function civ
adapter) remain entirely unbuilt, and every user-facing surface named above
(`civUrbanLayoutsChk`, `cityViewerModal`, `peCityPreview`/`peCityOpen`) has
no disclosure anywhere in this document or `GUI_GAP_REGISTER.md` prior to
this correction pass.

**Update, 2026-08-23 (same day, later pass): milestones 1-7 are wired.** The
"zero consumers" finding above is closed; the milestone count is not.
`cartalith-civ::urban_adapter` ports the subset of the block-2 `_um*` adapter
whose *outputs* milestones 1-7 can consume (13 of the 28: `_umSiteBoxKm`,
`_umWaterNearKm`, `_umWaterReachKm`, `_umSiteKindFromTerrain`, `_umInferAge`,
`_umRayBoxExit`, `_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`,
`_umTerrainOrient`, `_umWaterCtx`, `_umTerrainCtx`, `_umPlaceContext`) plus
the prefix of `generate()` those seven milestones supply;
`cartalith-godot::urban_bridge` exposes it as one batched `#[func]`; and two
of the three user-facing surfaces are live — the deep-zoom map layer
(`civUrbanLayoutsChk`) and the City Viewer (`cityViewerModal`), the latter
with its own canvas, wheel-zoom, drag-pan, legend and info panel. The
place-edit popup's thumbnail (`peCityPreview`/`peCityOpen`) is not, though
`app.open_city_viewer(index)` now exists for a popup to call.

What that produces is a **street skeleton on a real site**: the map's own
river/coast and relief fed into `buildSite`, the market anchor, the arterial
primaries (grown around the port's real inter-settlement roads when any
reach the settlement), and the organic street growth off them. Blocks,
parcels, buildings, districts, amenities and the wall circuit are milestones
8-17 and are drawn nowhere, stubbed nowhere, and emitted as no dictionary key
at all. Six `_um*` functions are deliberately unported because their only
consumers are milestone 8+ (`_umWallSpec`, `_umInferWalls`, `_umHarbourScale`,
`_umSiteProfile`, `_umOreBearing`, and `_umCacheKey`'s content fingerprint);
`_umPt` has no Rust equivalent to need; the LRU/queue and the two draw
functions are out of scope for every milestone by the scope document's own
statement. **The adapter is not golden-verified** — the capture harness this
repository's goldens come from slices block 4, and there is no block-2
fixture; the engine beneath it is golden-verified milestone by milestone, the
adapter is ported by reading and covered by ordinary unit tests.

**§7d tag**: port as-is. This is deep procedural-generation domain logic
with real reference precedent line-for-line (`URBAN_MORPHOLOGY_SCOPE.md`
cites exact reference line ranges per milestone); nothing here suggests a
modernize-over-port angle the way tile pyramids or layer compositing do.

## Summary coverage table

| Capability | Status | §7d tag |
|---|---|---|
| World generation | Done | Port as-is |
| Terrain editing (Sculpt) | **Built** (corrected 2026-08-23) | Port behavior, modernize storage |
| Hydrology/climate/ecology live tuning | Absent | Open design question |
| Civilisation | Done, incl. Timeline/collapse (corrected 2026-08-23) | Port as-is (territory: modernized by necessity) |
| Journey Planner | **6/6 milestones, engine-complete** (corrected 2026-08-23) | Port as-is |
| Map rendering (default + atlas) | Done, improved | Already modernized |
| Map rendering (NPR/geology/SDF toggles) | Absent | Port as-is |
| Tile pyramid / LOD / region export | **LOD tiling live; region export wired** (corrected 2026-08-23) | Modernize (Mapbox-style quadtree); atlas/cache still open |
| Labels/annotation (rule-driven) | Done | Port as-is |
| Labels/annotation (manual tools) | **Built**, incl. biome/terrain paint (corrected 2026-08-23) | Port as-is |
| Asset library (data layer) | Done, Phase 4 complete | Port as-is |
| Asset library (authoring UI) | **Built** (corrected 2026-08-23) | Port as-is |
| Import (GeoJSON/heightmap) | **Heightmap done; GeoJSON import still absent** (corrected 2026-08-23) | Port as-is |
| Export (tiles/image) | **Tile export live; GeoJSON export ported, unbound** (corrected 2026-08-23) | Modernize (tile-server-style) for slippy-map addressing remainder |
| Save (read) | Done | Port as-is |
| Save (write) | Absent | Port as-is |
| 2D rendering | Done | Port as-is |
| 3D view | Absent, deferred by decision | Modernize when built (`Terrain3D`) |
| Analysis-field switching | **Resolved** — 18 live views + 11 with stated reasons (corrected 2026-08-23) | Port as-is |
| Theme | Done, incl. light + follow-system | Already modernized (minor) |
| Credits | Done | Port as-is |
| Undo | **Draft-scoped real; global still absent** (corrected 2026-08-23) | New implementation, necessarily, for the global case |
| Urban morphology | **Milestones 1-7 of ~17 done, and now wired end to end** — adapter, bridge, deep-zoom map layer and City Viewer; what draws is a street skeleton, not a city (added and updated 2026-08-23) | Port as-is |

## Honest absent-entirely list, with real size

**Rewritten 2026-08-23** (`PARITY_AUDIT.md` C2) — most of the previous
version of this list named things that had since been built. What is
genuinely still absent, as of this correction:

- **Urban morphology, milestones 8-17**: radial streets/plaza/waterway,
  water infrastructure, fortification, graph cleanup, blocks/parcels,
  districts/buildings, amenities, hinterland/decay/details/metrics,
  `generate()`/`hashModel` — ~45 of block 4's 92 functions. Milestones 1-7
  are done **and wired** as of the later pass the same day (adapter, bridge,
  map layer, City Viewer); 13 of the 28 adapter functions are ported, and the
  other 15 are either milestone-8+-only, not applicable to typed Rust, or
  explicitly out of scope for every milestone. By far the largest item on
  this list — see capability 13 above.
- **Hydrology/climate/ecology live re-tuning without a full regenerate**:
  open design question, not yet even scoped as a milestone target.
- **Save writing**: `.zip` save is entirely unbuilt (read-only so far).
- **GeoJSON import**: absent (GeoJSON *export* is ported and golden-verified
  but not yet bound to Godot — see capability 9).
- **NPR Painter styles, geology microtexture toggle, SDF tinting, contour
  intervals**: presentation-only, no engine dependency, unscoped.
- **Global (generation-parameter) undo**: absent, tied to the tool-system
  work. Draft-scoped undo/redo for sculpt is real.
- **Journey Planner's individually-registered gaps**: `jpAutoPickTransport`/
  `_jpRerouteForMode`, the calculation-trace window, journey save/registry,
  ⇧-drag spine trim, the vessel sailing window/resolver — none
  milestone-sized any more; the Journey Planner itself is engine-complete.
- **Persistent LOD tile atlas/cache, and the bake/finalize-lock**: tile
  synthesis itself is live; nothing persists it to disk yet.
- **Settlement/place editing and deletion, and the map's right-click context
  menu**: `civ_drop_settlement` creates a settlement; nothing edits, moves
  or deletes one, and there is no `_civCtxShow`-equivalent context menu on
  the map (`PARITY_AUDIT.md` §3.2, §5 items 2-3; see `GUI_GAP_REGISTER.md`
  ED-03 for the corrected framing).

## Top three modernize-over-port recommendations

**1. Tile pyramid / LOD / deep-zoom rendering.** Leading example: **Mapbox
GL's tile pyramid** — tiles as a quadtree, `2^zoom × 2^zoom` per level,
geometry/detail simplified at lower zoom so rendering cost scales with what's
visible rather than total world size. The reference's own tiled-LOD system
exists because a browser `<canvas>` had no other way to hold a 20,000km
world; that constraint doesn't apply here. **Landed 2026-08-23**:
`cartalith-spatial`'s `QuadTree<T>` and `TiledField<T>` are no longer
unintegrated — deep-zoom tile synthesis (`lod_bridge.rs`,
`viewport_host.gd`) is live and automatic. What remains of this
recommendation is the persistent on-disk tile cache/atlas, not the
quadtree integration itself.

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
a tiled field, not whole-canvas snapshots. **Landed 2026-08-23**:
`cartalith-spatial`'s `DirtyTracker` and the `PassBuffer<Stamp>` type
`UNIFIED_TOOL_PLAN.md` specified are both real and wired
(`sculpt_bridge.rs`, `sculpt_undo`/`sculpt_redo`), modeled directly on the
reference's own `sculptStamps[]`/commit/discard semantics — the behavior
ported, the storage modernized, as recommended. Only global
(generation-parameter-level) undo remains open.

## Disagreements with prior docs

**Corrected 2026-08-23** (`PARITY_AUDIT.md` C2/§7). This line previously
read "None found" while this document's own summary table disagreed with
`cartalith-native/docs/STATUS.md` — the document this file says it is
checked against — on ten rows plus capability 4's body: Sculpt, Journey
Planner, tile pyramid/LOD/region export, manual annotation, asset-library
authoring UI, heightmap import, tile export, undo, analysis-field
switching, and the Timeline/collapse layer. Each has now been corrected in
place above rather than left as an open disagreement — see every "corrected
2026-08-23" marker in this document, all citing `PARITY_AUDIT.md`'s
file:line evidence, independently re-verified rather than taken on the
audit's word. One additional disagreement this correction pass found on its
own, not in the audit's original list: capability 8's claim that the
Cartography biome/terrain "painted layer" tool was never ported disagreed
with `paint_bridge.rs`'s real, wired `paint_set_layer`/`paint_stroke_at`/
`paint_commit` surface and `world_workspace.gd`'s live "Biome paint (B)"
tool — corrected in capabilities 7 and 8 above.

One gap remains genuinely open, not previously flagged anywhere: interactive
re-tuning of generation parameters without a full regenerate (capability 3)
still has no scope document and no recorded design decision — the one-shot
generation model is itself a deliberate, documented choice
(`HARDWARE_ACCELERATION.md`) that this capability would need to be
reconciled with, not silently assumed away.
