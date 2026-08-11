# Azgaar's Fantasy Map Generator vs Cartalith Gen1 — comparative analysis

*Written 2026-07-18 (session research). Sources: FMG GitHub README + wiki (fetched), Azgaar's dev blog
(fetched search), and training knowledge of FMG's long-stable behaviors (marked ⁽ᵏ⁾ where not
re-verified this session). FMG state at time of writing: v1.1xx line, "Economy" update mid-2026,
active development (~2k commits, 5.8k stars), gradual JS→TypeScript migration, SVG + WebGL rendering.*

## 1. The two tools in one paragraph each

**Azgaar's Fantasy Map Generator (FMG)** is a free, browser-based fantasy-map *authoring environment*.
It generates a complete fictional world — terrain, climate, rivers, biomes, cultures, religions,
states, provinces, settlements ("burgs"), routes, emblems, and (2026) an economy layer — on an
**unstructured Voronoi cell graph** (~10k cells default, up to ~100k), renders it as **layered SVG**
with a WebGL 3D view, and wraps every layer in a dedicated interactive editor. Its centre of gravity
is *editorial breadth*: everything the generator produces can be hand-edited, restyled, renamed and
exported (SVG/PNG/tiles-zip/JSON/.map).

**Cartalith Gen1** is a single-file, zero-dependency worldbuilding tool whose centre of gravity is
*physical simulation depth on a raster*: a plate-tectonics substrate (stress, flexure, orogeny),
climate↔erosion coupling, mass-conserving sediment routing, Strahler hydrology and priority-flood
lakes on dense **Float32Array fields** (512–8K grid), rendered as a shaded raster with an
IndexedDB-backed LOD tile pyramid, plus a civ/politics layer, an asset library, and — unique among
the tools in this space — an **embedded procedural urban-morphology engine** that renders each
settlement's street-level town *in place on the region map*, seamless with the region's real roads
and water.

## 2. The core architectural divergence

Everything else follows from one choice:

| | FMG | Cartalith Gen1 |
|---|---|---|
| World model | Unstructured Voronoi cell graph (jittered grid → Delaunay/Voronoi, "repacked" to the coastline); per-cell attributes (height 0–100, temp, precip, biome, culture, state, religion, pop…) | Dense raster fields (`GW×GH` Float32Arrays: height [0,1], temp °C, rain, flow, stress, flexure…), plus typed per-cell masks |
| Cell count / resolution | ~10k default, ~100k practical ceiling (performance-bound) | 0.26M cells at 512px → 33M+ at 8K (memory-bound) |
| Rendering | SVG polygons/paths per layer + WebGL 3D; style system per layer | Per-pixel raster colorization (materials, hillshade, NPR styles), canvas overlays for vectors, LOD tile pyramid |
| Natural strengths | Cheap topology (neighbors, region growing, pathfinding on cells); crisp vector export; per-entity editing | Physical PDE-ish operators (diffusion, erosion, flow accumulation); continuous detail under zoom; photographic shading |
| Natural weaknesses | Low spatial resolution: one cell ≈ many km²; terrain is schematic; zooming reveals polygon soup | Editing individual features is harder; vector export is secondary; grid resolution caps crispness (see #96 lake fix, v1.05) |

FMG's graph is a *cartographer's* data structure — regions, borders, labels and routes fall out
almost for free. Cartalith's raster is a *geologist's* — erosion, climate coupling and continuous
relief fall out almost for free. Each tool pays steadily for the other's free lunch.

## 3. Pipeline-by-pipeline comparison

### 3.1 Terrain
- **FMG**: heightmap templates ("blob"-composition algorithms — one big island blob + ~10 small
  modifier blobs, plus named templates like Archipelago/Continents), value range 0–100 with sea at 20;
  manual heightmap brush editor; image-to-heightmap import. **No tectonic model** — mountains are
  painted or templated, not caused.
- **Cartalith**: full causal chain — plate seeding → boundary classification → stress/flexure →
  orogeny → volcanism/craters → erosion ops (droplet, stream-power, glacial, velocity) in Web Workers
  → mass-conserving sediment routing; import path *infers* tectonics from a loaded heightmap so the
  rest of the pipeline has a substrate.
- **Verdict**: not comparable in kind. FMG terrain is a fast sketch to carry the *human* layers;
  Cartalith terrain is the product itself. FMG's named **templates** are still worth borrowing as
  setup-gate archetype presets (one-click "Archipelago / Pangaea / Inland sea" parameter bundles).

### 3.2 Climate & biomes
- **FMG**: latitude temperature bands adjusted by altitude; precipitation from a simplified
  prevailing-wind pass; biome from a temperature×moisture matrix.⁽ᵏ⁾ Cheap, plausible, editable.
- **Cartalith**: temperature with CPU/GPU-lockstep lapse, seasonal fields, rain with orographic
  effects, `evolveCoupled` climate↔erosion loop, Köppen classes, lithology→soil→water-access
  affordance fields feeding carrying capacity.
- **Verdict**: Cartalith is a class deeper; FMG's matrix approach is however *transparent to users*
  (editable biome matrix). A user-facing "why is this cell this biome" inspector is an FMG-ish
  affordance Cartalith's Info tool partially covers.

### 3.3 Hydrology
- **FMG**: downhill flux accumulation on the cell graph, depression resolution, river polylines with
  width classes and a river editor.⁽ᵏ⁾
- **Cartalith**: rain-seeded discharge accumulation, Strahler orders, river-network tracing feeding
  both rendering (order-deepening blue, v0.96) and the settlement layer (real centerlines into town
  layouts, v0.98); priority-flood lakes whose **pool fill level** now drives sub-cell shorelines
  (v1.05).
- **Verdict**: comparable concepts, different substrate; Cartalith's hydrology is load-bearing for
  more downstream systems (towns, journeys, suitability).

### 3.4 The human world (civ layer)
This is FMG's home turf and its breadth is unmatched:
- **FMG**: cultures (spread by cost expansion, each with a **namesbase** driving all naming), states
  *and provinces* (two administrative tiers), **religions** (folk/organized, spread models),
  diplomacy matrices, military regiments, **emblems/heraldry** (the Armoria COA generator), zones,
  markers, rural/urban population per cell, and the 2026 **economy update**. Every layer has an
  editor.
- **Cartalith**: factions with territory paint, capacity-grounded settlement populations
  (catchment × carrying capacity × trade centrality), network metrics (betweenness/closeness) feeding
  economy/trade volume, timeline + collapse simulation (mortality/migration), journey planner.
- **Verdict**: FMG wins on *breadth of named institutions* (provinces, religions, heraldry,
  name culture); Cartalith wins on *grounding* (population from carrying capacity and network
  position rather than dice) and on temporal simulation (collapse, timeline). The clearest
  borrow-list from FMG: **culture-flavored name generation** (namesbases), a **province/region tier**
  between faction and settlement, and optionally a religions layer.

### 3.5 Settlements & urban layouts — the sharpest contrast
- **FMG**: burgs are *points* with attributes (population, port/capital flags, features). For an
  actual town plan, FMG **links out** to Watabou's Medieval Fantasy City Generator / Village
  Generator, passing the burg's size/flags as URL parameters — the city opens as a separate
  document in a new tab, visually unrelated to the surrounding map.⁽ᵏ⁾
- **Cartalith** (v0.95–v1.05): the town layout is generated by an embedded engine and drawn **on the
  region map itself** under deep zoom — crossfading from the pin, built *around the real approach
  roads* (the through-road is the high street, gates where it crosses the wall), with the town's
  river/coast being the map's actual water, plus a fit-to-town card in the settlement popup.
- **Verdict**: this is Cartalith's genuinely novel capability. Nothing in FMG (or its ecosystem)
  renders a street-level town *in situ*, continuous with regional roads and hydrology. FMG's
  delegation is pragmatic and its Watabou plans are prettier per-building today — Cartalith's
  layouts close that gap version by version, and the *seamlessness* cannot be retrofitted onto a
  link-out architecture.

### 3.6 Routes
- **FMG**: cost pathfinding over the cell graph for roads/trails/sea routes; editable.⁽ᵏ⁾
- **Cartalith**: two-pass hierarchical network (MST → min-degree fill → detour-relief shortcuts) with
  road-reuse corridors, settlement gravity, corridor consolidation (shared strokes), sea lanes with
  port logic, endpoint-exactness invariants (v0.92/v1.02), and a day-by-day journey planner.
- **Verdict**: Cartalith's network model is meaningfully richer (corridors, reuse, hierarchy);
  FMG's is simpler but fully editable in-UI. Cartalith lacks FMG's route hand-editing depth.

### 3.7 Rendering, UX, editing
- **FMG**: mature layer/style system (every layer restylable, style presets shareable), strong label
  placement, dedicated editors for *everything*, 3D scene + globe, submap/resample tooling.
- **Cartalith**: raster fidelity (multi-scale hillshade, materials, NPR parchment styles), LOD tile
  pyramid with baked-atlas persistence, 3D drape, phase-based UI (Generate→Explore), fill-mode
  viewport (v1.01), style presets.
- **Verdict**: FMG is the stronger *editor*; Cartalith the stronger *renderer/simulator*. FMG's
  **submap/resample** (cut a region out and re-generate it at higher effective resolution) is the
  single most strategically relevant UX idea for Cartalith's LOD story — it is the graph-world
  cousin of what the amplification/LOD pipeline already does numerically.

### 3.8 Persistence, interop, ecosystem
- **FMG**: single `.map` save, Dropbox cloud saves, SVG/PNG/tile-zip/**JSON (GIS-usable)** exports;
  a large community (Discord/Reddit), and an ecosystem of sibling tools (Armoria heraldry, Deorum
  characters, Watabou integrations).⁽ᵏ⁾ Notably imported *into* other pipelines (e.g. Unity worlds).
- **Cartalith**: ZIP project (params + f32 fields + PNG layers + Cartalith-loadable baked bins +
  atlas + asset library), in-app CC0 asset importer. No cloud saves; no GeoJSON.
- **Verdict**: FMG's JSON/GeoJSON export earns it a place in GIS/game pipelines; a GeoJSON exporter
  (settlements, ways, territories, rivers as features; height/biome as GeoTIFF-ish rasters) would be
  a cheap, high-leverage addition for Cartalith.

### 3.9 Determinism & verification
- **FMG**: seed-driven generation (seed visible/settable, maps reproducible); no public automated
  test culture to speak of — regressions are caught by the community.⁽ᵏ⁾
- **Cartalith**: seed box at setup (v1.06) with verified cross-boot bit-identity; 923-assertion
  engine suite + 831-assertion urban-morphology suite headless; A/B FNV bit-identity battery gating
  every release; 179-assertion Playwright smoke suite.
- **Verdict**: Cartalith's verification rigor is unusual for the genre and is a real moat for
  refactoring speed — FMG's TypeScript migration is partly compensating for the same need.

### 3.10 Performance envelopes
- **FMG**: interactive at 10k cells; the wiki itself warns cell count "highly affects performance"
  and recommends disabling relief icons; 100k cells is the practical ceiling.
- **Cartalith**: 512–2K interactive with GPU passes and worker pools; 4K/8K supported but
  memory-heavy; LOD tiles + atlas amortize deep zoom.
- Not directly comparable (10k Voronoi cells ≈ a 100×100 raster in *count*, but each FMG cell does
  far more semantic work; each Cartalith cell far more physical work).

## 4. What Cartalith should consider borrowing (ranked)

1. **Culture-flavored naming (namesbases).** Cartalith's `_civSettleName` is a single generator;
   FMG's per-culture Markov-ish namesbases make regions *feel* distinct at zero simulation cost.
   Small, self-contained, high flavor-per-line.
2. **Setup-gate world archetypes.** FMG's heightmap templates as one-click parameter bundles
   ("Archipelago", "Pangaea", "Two continents", "Inland sea") on the new-world screen — Cartalith
   has the parameters; it lacks the curated bundles.
3. **GeoJSON / GIS export.** Settlements, ways, territory outlines, river polylines as GeoJSON;
   opens the same downstream-pipeline door FMG's JSON export opened.
4. **Province tier + religions layer.** A mid-tier region between faction and settlement (FMG
   provinces) would give the territory paint administrative structure; religions are a similar
   spread-model layer if wanted.
5. **Submap/resample UX.** Framing the existing amplification/LOD machinery as an explicit "carve
   this region into its own higher-resolution map" tool — FMG proves users want this workflow.
6. **Label placement + per-layer style editors.** FMG's label engine and restyle-everything panels
   are the editor-maturity bar.

## 5. What FMG users would envy in Cartalith

- Caused terrain (tectonics→erosion) and raster relief quality; heightmap import that *infers* a
  tectonic substrate rather than just colorizing.
- **In-situ procedural towns** seamless with regional roads/water (no equivalent exists in FMG's
  ecosystem — its city plans are a separate website).
- Climate↔erosion coupling, seasons, Köppen, lithology/soil/affordance fields.
- Capacity-grounded populations and the timeline/collapse simulation.
- The journey planner's day-by-day staging.
- Release-gated bit-identity + 1,900+ automated assertions.

## 6. Bottom line

FMG and Cartalith are converging on the same product category from opposite ends. FMG started from
the *map as a document* (vector cartography + editors for every human layer) and is slowly deepening
its simulation (economy update, TS migration). Cartalith started from the *world as a simulation*
(fields, physics, coupled systems) and is climbing toward FMG's editorial breadth (factions,
timeline, popups, styles). The near-term strategy this suggests: **don't chase FMG's breadth
layer-for-layer** (provinces/religions/heraldry are commodity features FMG already does well);
instead double down on what the raster+simulation architecture uniquely enables — seamless
zoom-through-scales (region → town → street), physically-caused terrain, and grounded economies —
while cherry-picking FMG's cheap flavor wins (namesbases, archetype presets, GeoJSON export).
