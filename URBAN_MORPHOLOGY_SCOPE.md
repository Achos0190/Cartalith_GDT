# Urban morphology (Phase 5): investigation and milestone plan

`ROADMAP.md`'s Phase 5 entry reads:

> Block 4, procedural city layouts. Already a self-contained DOM-free engine in
> the JS codebase, which suggests it ports cleanly into `cartalith-urban`,
> depending on `cartalith-civ` for settlement context.

That sentence was written before anyone read the code. This document is the
result of actually reading it. **Two of its three claims hold, one does not,
and the word "cleanly" is doing work it cannot support.**

## What was verified, and what it turned out to be

### "Self-contained DOM-free engine" — CONFIRMED, unusually strongly

Script block 4 (reference `Cartalith Gen1 v2.10.html` lines 28166-31104, 2,937
lines inside the `<script>` tags) is a single `const UME = (() => { … })()`
IIFE. Grepping its line range for `document`, `window`, `canvas`, `ctx.`,
`getElementById`, `localStorage` and `requestAnimationFrame` returns **zero
hits** — the only match in the whole range is the word "context" inside a
comment. It is the cleanest subsystem boundary this port has met. The reference
even wraps it in an HTML comment saying so (`<!-- UM-ENGINE-START (pure JS, no
DOM — extracted by tests/run_um.sh) -->`) and ends it with
`if(typeof module!=='undefined'&&module.exports)module.exports=UME;`, i.e. the
reference's own authors already ran it headlessly under Node.

Better still, it ships its own parity apparatus: `hashModel(m)` (line 31087), a
stable FNV serialisation of the graph, blocks, parcels and buildings, written
explicitly "for determinism goldens"; and a `_test` export exposing fourteen
internal functions. Golden verification here is not something this port has to
invent — the reference built the door.

**Contrast with the two subsystems this project has already sized.** The Asset
Library turned out to be a UI browser wrapped around a frozen vocabulary; the
Journey Planner turned out to be ~70 functions with form-coupled orchestration.
Urban morphology is the opposite shape: almost no UI coupling at all, and far
more engine.

### "Does not consume asset packs" — CONFIRMED independently

Phase 4's own milestone-1 investigation recorded that "Phase 5's urban
morphology does **not** consume packs (block 4 has no `assetPack` reference)".
Re-checked from scratch here: `assetPack`, `AssetLibrary` and `AssetDB` all
return zero hits inside 28166-31104. Block 4 emits **geometry with kind tags**
(`b.kind` on buildings, `par.district` on parcels), not image references. Any
future pack-driven rendering of a town would be a *renderer* decision layered on
top, not an engine input. Phase 4's finding stands, unamended.

### "Depending on `cartalith-civ` for settlement context" — WRONG, and usefully so

The UME engine takes **no civ types whatsoever**. `generate(seed, opts)`'s
entire input surface is:

| input | type | this port's status |
|---|---|---|
| `seed` | `u32` | trivially available |
| `pop`, `epochs`, `settlementAge` | numbers | `cartalith-civ` has population/tier |
| `culture`, `site`, `faith`, `civicStyle`, `wallStyle` | strings from fixed vocabularies | partly available |
| `walls`, `fortified`, `ruined`, `terrainAware`, `wallGenerations` | booleans | derived, see below |
| `opts.water` | `{mask, dt, mw, mh, cellM, riverPath, riverWidthM, riverOrder, seaLakeCells}` — a plain raster + polyline | `cartalith-hydrology` has all of it |
| `opts.terrain` | `{grid, mw, mh, cellM, hMin, hMax}` — a plain heightfield raster | `cartalith-terrain` has it |
| `opts.routeEnds` | `[{x,y}]` in the site box | needs the road network |
| `opts.primaryPaths` | `[[{x,y}…]]` in the site box | needs the road network |
| `opts.economy` | `{specialisation, oreBearing}` | **not ported** (see below) |

That is scalars and rasters. No `Settlement`, no faction, no territory. The
dependency the roadmap describes is real but it lives **one layer up**, in
script block 2's `_um*` adapter (lines ~22036-22960, 28 functions, 925 lines),
which turns a settlement `p` into that opts object. That adapter is genuinely
civ-coupled; the engine is not.

**Consequence for the crate graph:** `cartalith-urban` must **not** depend on
`cartalith-civ`. It depends on `cartalith-rng` and nothing else. The adapter is
a separate, later milestone and is the only piece that needs civ types. This is
not a stylistic preference — it is what let milestone 1 be built and verified
in a session where `cartalith-civ` was mid-edit by a sibling fork.

### "Ports cleanly" — TRUE OF THE BOUNDARY, FALSE OF THE EFFORT

The boundary is clean. The volume is not. Measured against the two efforts this
project has already sized:

| subsystem | functions | lines | milestones |
|---|---|---|---|
| Journey Planner | ~70 | ~3,100 (17300-20400) | 6 |
| Asset Library | 19 top-level | ~2,250 | 7 |
| **Urban morphology, engine only** | **92** | **2,937** | **~13** |
| **plus the civ adapter** | **+28** | **+925** | **+2** |
| **Phase 5 total** | **120** | **~3,860** | **~15** |

And the engine's 2,937 lines are *denser* than the Journey Planner's: block 4 is
written in the reference's compressed multi-statement-per-line style, with
several functions (`buildWall` ~190 lines, `grow` ~167, `buildBuildings` ~148,
`applyStarFort` ~100) that are single algorithms, not dispatch tables.

**Phase 5 is the largest single unported subsystem remaining in this project.**
"Ports cleanly" should be read as "has no boundary problems", not as "is small".
Recording that correction is the most valuable output of this investigation.

## What it actually generates, and how

Not a street-network-only tool. `generate()` returns a complete cadastral
model, produced in this order:

1. **Site** (`buildSite`) — the physical setting in a fixed 1700 × 1250 m box
   (`SITE_WM`/`SITE_HM`). Either synthesises a river/coast/bay/landlocked site
   from the seed, or — the path the host app actually uses — wraps the real map's
   water mask, distance transform and river centreline (`opts.water`) and the
   real heightfield (`opts.terrain`). Returns closures: `height`, `slope`,
   `riverDist`, `isWater`, `bankSide`, plus `bridgePt`, `harbour`, `routeEnds`.
2. **Anchors** (`placeAnchors`) — the market square, scored over 400 seeded
   candidate points against slope, flood band and distance from the
   break-of-bulk point (bridge or quay).
3. **Primary routes** (`buildPrimaries`) — an 8 m cost raster with a
   Tobler-flavoured slope penalty, then **A\*** from each external route
   endpoint to the market, with reinforcement. Or `buildPrimariesFromPaths`
   when the host supplies real inter-settlement roads. Or, for the radial
   (Venus) culture, `buildRadialStreets` laying concentric rings and spokes.
4. **Growth** (`grow`) — the heart. An epoch loop (default 8) that spends a
   population-derived street-length budget on seeded candidate segments,
   branching off existing streets at near-perpendicular angles with jitter,
   with a decaying exploration share, a market-distance density gradient,
   junction-angle and parallel-spacing rejection, bridgehead rules for the far
   bank, and optionally successive wall generations gated on real elapsed years.
5. **Fortification** (`buildWall`, `applyStarFort`) — a curtain traced around
   the built-mass convex hull with gates at radial street crossings, or a
   bastioned *trace italienne* with a wet or dry moat, gated behind a
   population minimum and an explicit anachronism guard.
6. **Cleanup passes** (`lanePass`, `removeWaterCrossings`, `pruneLargest`,
   `privatizeAlleys`, `clearFortZone`).
7. **Blocks** (`buildBlocks`) — **planar face extraction** over the street
   graph (angularly-sorted half-edge traversal with spur collapsing), each face
   inset by half the width of each fronting street.
8. **Parcels** (`buildParcels`) — series platting by **vertex bisectors**
   capped by ray-casts to the opposite boundary, with log-normal frontage widths
   and plot depths and a burgage re-subdivision cycle.
9. **Districts** (`assignDistricts`) and **buildings** (`buildBuildings`) —
   per-parcel footprint polygons by building grammar, with ridge lines.
10. **Amenities** — `buildMarkets`, `buildCivic`, `buildFaithSites`,
    `buildGames`, `buildHarbour`, `addRiverBridges`.
11. **Hinterland and state** — `buildFarmland` (strip and ring fields),
    `buildDetails`, `applyDecay` (the "ruined" toggle), `computeMetrics`.

So: street networks **and** blocks **and** plot subdivision **and** building
footprints **and** districts **and** walls **and** farmland. All four of the
things the task's question listed, plus more.

## RNG: checked, not assumed

Block 4's own header comment states that `mulberry32` is "intentionally NOT
redefined here … it falls through to the byte-identical module-scope copy
already in script block 1". Verified: there is no `mulberry32` in 28166-31104,
and the block-1 copy at line 2291 is the one `cartalith-rng` already
golden-verifies. So unlike Phase 2 milestone 9's `_civRng` — which turned out to
be the same *algorithm* under a different wrapper — this is literally the same
function.

What is new is the **seed derivation**: `stream(seed, label)` =
`mulberry32((seed>>>0) ^ fnv1a(label))`, giving labelled substreams
(`'site'`, `'anchors'`, `'grow/e3'`, `'parcels/blk7'`, …) so each stage draws
independently from one town seed. `fnv1a` has no Gen1 equivalent. Both are
ported and golden-verified in milestone 1.

`stream` also carries `range`/`int`/`pick`/`norm`/`logn`/`chance` over one
shared generator, so **call order is load-bearing**: `norm()` is Box-Muller and
consumes **two** draws, and `pick` consumes a draw even when the array is empty.
Milestone 1 pins all of that.

## A parity trap found while building milestone 1

`V.len`/`V.dist` are `Math.hypot`. **V8's `Math.hypot` is not correctly
rounded, and differs from Rust's `f64::hypot`.** ECMA-262 leaves it
implementation-approximated; V8 scales by the largest magnitude and Kahan-sums
the squared ratios. On `(3, 3)`:

| | value |
|---|---|
| true 3√2 | 4.242640687119285146… |
| Rust `f64::hypot` | 4.2426406871192847703 (correctly rounded) |
| V8 `Math.hypot` | 4.2426406871192856585 (1 ulp high) |

This is not hypothetical: the very first golden run of `dist_pt_seg` failed on
it. Every distance in this engine flows through `Math.hypot`, and many are
threshold comparisons (`attachPoint`'s 11 m snap, `rawEdge`'s 3.5 m minimum
segment, `nearestNode`'s search radius) where being *more* accurate than the
reference is the wrong answer. `cartalith-urban::geom::js_hypot` reproduces V8's
algorithm and is golden-tested against twelve captured values, including an
explicit `assert_ne!` against `f64::hypot` so nobody "simplifies" it away.

Every later milestone must use it. This is exactly the class of thing
`cartalith-rust-conventions` exists to catch, and it would have silently
poisoned every downstream comparison.

## Milestones

Dependency-ordered. Each is a real, self-contained, independently verifiable
piece; the reference's `_test` export and `hashModel` make most of them
golden-verifiable rather than hand-checked.

### Milestone 1 — RNG substreams + geometry kernel: **done** (2026-08-18)

`fnv1a`, `stream` and its six derived draws; `V` (as `Vec2`), `js_hypot`,
`polyArea`, `polyCentroid`, `pointInPoly`, `segInt`, `distPtSeg`,
`polySelfIntersects`, `chaikin`, `simplify` (Douglas-Peucker), `ensureCCW`,
`insetPoly`, `clipConvex`, `convexHull`. 19 tests, all but one golden.
Crate: `cartalith-urban`, dependencies: `cartalith-rng` only.

Two reference behaviours are pinned as behaviours, not fixed as bugs:
`clipConvex` clips against the clip **segment** rather than the clip line (so a
subject poking past the window's corners can collapse to empty), and
`insetPoly` returns nothing at all — not a degenerate polygon — below area 15
or on self-intersection at ≤60 vertices. Downstream code reads both.

### Milestone 2 — planar street graph (lines 28363-28513, 15 functions)

`makeGraph`, `gKey`, `gridCellsForSeg`, `indexEdge`/`unindexEdge`/`edgesNear`
(the uniform-grid spatial index), `addNode`, `nearestNode`, `rawEdge`,
`splitEdge`, `attachPoint`, `addStreet`, `addPolylineStreet`, `extractFaces`,
`edgeBetween`. The planarity invariant (every crossing becomes a node) lives
here, and `extractFaces` is what makes blocks possible at all.

Golden path: `UME._test` exports `makeGraph`, `addStreet` and `extractFaces`
directly. This is the milestone where arena-vs-`Vec` index design gets settled
for the whole crate — the JS uses dense integer ids into `g.nodes`/`g.edges`
with a soft-delete `alive` flag, and the id **stability** matters
(`splitEdge` leaves dead edges in place), so the port should keep dense
`Vec`-with-tombstones rather than "improving" it into a slotmap.

### Milestone 3 — A\* over the cost raster (lines 28514-28556, 1 function)

`astar`. Small, isolated, exported from `_test`, and needed by milestone 5. A
hand-rolled binary heap with a 0.9-weighted Euclidean heuristic and
trapezoidal edge costs — the tie-breaking behaviour of that heap is what makes
the path reproducible, so it is ported literally rather than swapped for
`BinaryHeap`.

### Milestone 4 — generation rules + culture profiles (lines 28212-28289, 8 functions)

`CULTURE_PROFILES` (medieval/organic and Venus/radial), `resolveProfile`,
`DEFAULT_RULES`, `cloneRules`, `resolveRules`, `clamp`, `applyWildness`,
`applyPlotChaos`. Data, not algorithm, but every later milestone reads
`rules.street.*`/`rules.parcels.*`, and `applyWildness`/`applyPlotChaos` are
publicly exported so they golden-verify directly. Small; sequenced here because
milestone 7 cannot start without it.

### Milestone 5 — site model (lines 28557-28742, 3 functions)

`shoreFromMask`, `buildSite`, `terrainSuitability`. Defines the input contract
for everything downstream, including the **real-water and real-terrain raster
paths** that the host app actually uses. The JS returns closures (`height`,
`slope`, `isWater`, `riverDist`, `bankSide`); the port needs a `Site` struct
with methods and an enum for the synthetic-vs-real branch. Golden-verifiable by
driving `buildSite` with a fixed synthetic seed and with a fixed hand-built
raster, and comparing sampled fields.

### Milestone 6 — anchors and primary routes (lines 28744-28843, 3 functions)

`placeAnchors`, `buildPrimaries`, `buildPrimariesFromPaths`. First milestone
that produces a real street graph end to end, so the first that can be
golden-checked with a hash over graph state.

### Milestone 7 — organic growth (lines 29390-29630, 5 functions)

`logisticRamp`, `estimateCarryingCapacity`, `wallOccupancy`, `grow`,
`supersedeWall`. The single most behaviour-defining function in the subsystem
and the one most sensitive to RNG draw order: `grow` is an epoch loop drawing
from `stream(seed,'grow/e'+ep)`, and one extra or missing draw diverges every
later epoch. Expect this to be the hardest milestone to land, and expect its
golden to be a per-epoch graph hash rather than a single end-state hash, so a
divergence localises to an epoch.

### Milestone 8 — radial (Venus) streets, plaza, waterway (lines 28844-28970, 3 functions)

`buildRadialStreets`, `buildWaterway`, `buildPlaza`. The second planning mode,
independent of `grow`. Separable from milestone 7 and cheaper.

### Milestone 9 — water infrastructure (lines 28971-29159, 4 functions)

`distToLine`, `buildHarbour`, `addRiverBridges`, `detectRiverCrossings`.
Quays, moles, breakwaters, bridges, fords, and the navigability guards that
invalidate a harbour on a stream too small to carry one.

### Milestone 10 — fortification (lines 29631-30037, 9 functions, ~407 lines)

`ringCrossings`, `densifyLoop`, `nearestIdx`, `cornerCut`, `townBank`,
`builtMassHull`, `buildWall`, `applyStarFort` (`convexHull` already landed in
milestone 1). **The largest single milestone in this plan.** Curtain-wall
tracing around the built-mass hull, gate placement at radial crossings, wet/dry
ditches, and the bastioned trace with its own geometry.

### Milestone 11 — graph cleanup passes (lines 30038-30192, 6 functions)

`_killEdge`, `pruneLargest`, `removeWaterCrossings`, `privatizeAlleys`,
`clearFortZone`, `lanePass`. Ordering between these is load-bearing —
`detectRiverCrossings` deliberately runs after all of them so a recorded bridge
always has a live road on it.

### Milestone 12 — blocks and parcels (lines 30193-30344, 2 functions)

`buildBlocks`, `buildParcels`. Dense: the bisector platting method with
ray-cast depth caps, log-normal frontage/depth draws, overlap filtering and
area conservation. Directly hashable via `hashModel`'s block/parcel terms.

### Milestone 13 — districts and buildings (lines 30345-30710, 7 functions)

`assignDistricts`, `bmap`, `rectPoly`, `buildBuildings`, `_rectPts`,
`_peristyle`, `buildFaithSites`. Building grammars (burgage, venus-mixed),
the terrain-suitability building gate, churches and temples.

### Milestone 14 — amenities (lines 29160-29389, 5 functions)

`buildMarkets`, `buildCivic`, `orientedRect`, `gamesShapeAt`, `buildGames`.
Rank-scaled specialised markets, the civic hall, and the games/arena sites.

### Milestone 15 — hinterland, decay, details, metrics (lines 30711-30930, 7 functions)

`crossesStreet`, `stripFields`, `ringFields`, `buildFarmland`, `applyDecay`,
`buildDetails`, `computeMetrics`.

### Milestone 16 — `generate()` orchestration + `hashModel`

`generate` (lines 30931-31086) and `hashModel`. The payoff milestone: with
every stage ported, the port's `hashModel` output can be compared against the
reference's for a matrix of seeds, cultures, site kinds and population targets.
That is a **whole-subsystem** golden, and the reference wrote it for exactly
this purpose.

### Milestone 17 — the civ adapter (block 2, lines ~22036-22960)

The 20 pure functions of the `_um*` adapter: `_umSiteBoxKm`, `_umWaterNearKm`,
`_umWaterReachKm`, `_umSiteKindFromTerrain`, `_umInferAge`, `_umWallSpec`,
`_umInferWalls`, `_umHarbourScale`, `_umPt`, `_umRayBoxExit`,
`_umTerrainOrient`, `_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`,
`_umWaterCtx`, `_umTerrainCtx`, `_umSiteProfile`, `_umOreBearing`,
`_umPlaceContext`, `_umCacheKey`. This is the only piece that needs
`cartalith-civ`, `cartalith-hydrology` and `cartalith-terrain`, and it should
live **outside** `cartalith-urban` (in `cartalith-civ`, or in a thin
`cartalith-urban-adapter`) so the engine crate stays dependency-light.

**Two known gaps in this port's own data, to be honest about now rather than
discover at milestone 17:** the reference's settlements carry
`p.specialisation` (feeding `opts.economy` and thence districts/details) and
`p.traits` (feeding `fortified`), and this port has neither. Both have the same
honest fallback the reference itself uses when the data is absent —
`economy: null`, `fortified: false` — so a port without them behaves exactly
like the reference running on a world where nobody set them.

## Out of scope for every milestone

- **`_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha`** (block 2) and
  the block-1 LOD hook around line 15606 — canvas rendering and zoom-crossfade
  logic. That is Godot's job, not a port target, and belongs to whatever
  rendering milestone eventually draws a town.
- **`_umModelCache`/`_umScheduleGenStep`/`_umCacheEvict`/`_umModelFor`/
  `_umModelForNow`** — an LRU plus a one-per-frame `setTimeout(…,0)` generation
  queue, a workaround for the browser's single thread. This port has real
  threads; whatever scheduling it needs will be designed against those, not
  transliterated.
- **The removed 17 culture profiles.** The reference documents them as history
  (docs/07 §3.10) after a post-launch pass found them visually
  indistinguishable. Only `medieval` and `venus` are live; only those get
  ported.
- **`buildGridStreets` and the palimpsest planning mode** — likewise removed
  upstream, with no live caller.
- **Wiring into `compute_civilisation()`, `cartalith-godot`, or the GUI.** Same
  standing "don't wire in what nothing calls" discipline every subsystem port
  in this project has held to. Urban morphology's real integration is a
  rendering decision that does not exist yet.

## Verification convention for this subsystem

The harness slices reference lines **28167-31103 as one contiguous block**,
plus line 2291 (`mulberry32`, which block 4 deliberately does not define), and
evaluates them in a bare Node `vm.runInContext` with no DOM. **A block-comment
balance assertion runs on both slice boundaries** — Journey Planner milestone 4's
design, adopted here for the same reason: an unterminated `/*` at a boundary
silently swallows the rest of the slice, and one contiguous slice plus a
balance assert removes the whole class. Two further assertions check the slice
really starts at the IIFE and ends at the export.

Where `_test` or the public export reaches a function, expected values are the
reference's own output. Where it does not (`polySelfIntersects` is the only
milestone-1 case), the test is a real unit test of the ported logic and is
labelled as such — the precedent territory, provinces and `cartalith-spatial`
all set.
