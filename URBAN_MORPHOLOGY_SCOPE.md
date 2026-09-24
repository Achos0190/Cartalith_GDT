# URBAN_MORPHOLOGY_SCOPE.md — Phase 5: settlement layout (`cartalith-urban`)

**What this is:** the definition of Phase 5 — the reference's town-layout
engine (script block 4, "UME") and its block-2 `_um*` adapter, as milestones
1-17 plus 8a and 17a, with what reading and porting each one found: ranges,
constants and why, fixture rules, and every mutation survivor with the
invariant it rests on. **What it is not:** status. Where any milestone stands is
`cartalith-native/docs/STATUS.md`'s "Phase 5 — Urban morphology" group; what
remains is `OUTSTANDING_WORK.md` §2.1.

Every "reference line N" resolves against `reference/Cartalith Gen1 v2.10.html`.
A milestone's range includes the section-header comment that introduces it (the
convention milestones 4-7 settled); where its first function starts later, that
line is named too. Every range was re-checked against the file on 2026-09-24;
the plan's original ranges, most of them wrong, are tabled under "Verification
convention" for anyone reading an older citation.

**Owner rulings that bind this subsystem** (`LARGE_ITEM_RULINGS.md`):

- **Ruling H (2026-09-12):** urban generation moves toward the owner's town plan
  (`design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`)
  by all three routes offered — renderer first, a new culture profile, and
  changing the ported algorithm — and a golden re-baseline is authorised, for
  `cartalith-urban` only, each departure disclosed with the golden it moved. **Ruling AA** (2026-09-23) and **Ruling AD** (2026-09-23)
  settle two of its candidates.
- **Ruling I (2026-09-12):** a citadel, and star forts that actually generate.
  **Ruling AC** (2026-09-23) sites the citadel by size tier.
- **Ruling J (2026-09-12):** a per-settlement city type, and regenerating one
  settlement at will.
- **Ruling N (2026-09-20):** a settlement binds to a river only through real
  geometry (milestone 5, finding 3).
- **`DECISIONS.md` §7p (2026-09-23):** a deliberate change this port made over
  the reference is the standard. Superseded behaviour below survives only as a
  dated line saying not to restore it.

What those rulings added is under "Beyond the reference", near the end.

**The source engine's later urban changes** are specified in
`RC_ENGINE_CHANGES.md`: §6p (site vectors and industry siting), §6q (the parcel
status gradient), §6r (the rules table the reference's host never set, and
§6r.5's unbounded frontage-retry loop in `buildParcels`), §6s (ward-set plot
grain, and §6s.4's corner-only water test), and §8.2's v2.66-v2.68 rows (v2.68:
`field`/`pasture` details the reference generated and never drew). That document
claims nothing about port status, and neither does this one. Each milestone
below names the section that changes its functions.

---

## What was verified, and what it turned out to be

`ROADMAP.md`'s Phase 5 entry originally read:

> Block 4, procedural city layouts. Already a self-contained DOM-free engine in
> the JS codebase, which suggests it ports cleanly into `cartalith-urban`,
> depending on `cartalith-civ` for settlement context.

Two of its claims held, one did not, and "cleanly" was true only of the
boundary.

### "Self-contained DOM-free engine" — confirmed, unusually strongly

Script block 4 (lines 28166-31104; 2,937 lines inside the `<script>` tags) is a
single `const UME = (() => { … })()` IIFE. Grepping it for `document`,
`window`, `canvas`, `ctx.`, `getElementById`, `localStorage` and
`requestAnimationFrame` returns **zero hits** (the only match in the range is
the word "context" in a comment). The reference brackets it with
`<!-- UM-ENGINE-START (pure JS, no DOM — extracted by tests/run_um.sh) -->`
(line 28164) and ends it with
`if(typeof module!=='undefined'&&module.exports)module.exports=UME;` — its own
authors already ran it headlessly under Node.

It ships its own parity apparatus: `hashModel(m)` (line 31087), a stable FNV
serialisation of graph, blocks, parcels and buildings written "for determinism
goldens", and a `_test` export of **fifteen** internal functions (`polyArea`,
`polyCentroid`, `pointInPoly`, `segInt`, `insetPoly`, `clipConvex`,
`extractFaces`, `makeGraph`, `addStreet`, `ensureCCW`, `convexHull`, `simplify`,
`chaikin`, `astar`, `distPtSeg`). **`hashModel` takes a finished `generate()`
model**, so it cannot be fed a partial subsystem: it is a milestone-16
instrument. Earlier milestones dump state directly, which is also stricter —
`hashModel` rounds coordinates to `Math.round(n.x*100)`.

### "Does not consume asset packs" — confirmed independently

`assetPack`, `AssetLibrary` and `AssetDB` return zero hits in block 4. It emits
**geometry with kind tags** (`b.kind` on buildings, `par.district` on parcels),
not image references; pack-driven town rendering would be a renderer decision,
not an engine input. Phase 4's own finding stands.

### "Depending on `cartalith-civ`" — wrong, and usefully so

`generate(seed, opts)`'s entire input surface is scalars and plain rasters:

| input | type |
|---|---|
| `seed` | `u32` |
| `pop`, `epochs`, `settlementAge`, `harbourScale` | numbers |
| `culture`, `site`, `faith`, `civicStyle`, `wallStyle`, `harbourDefence` | strings from fixed vocabularies |
| `walls`, `fortified`, `ruined`, `terrainAware`, `wallGenerations` | booleans |
| `rules` | a partial `DEFAULT_RULES` (milestone 4) |
| `opts.water` | `{mask, dt, mw, mh, cellM, riverPath, riverWidthM, riverOrder, seaLakeCells}` — a raster and a polyline |
| `opts.terrain` | `{grid, mw, mh, cellM, hMin, hMax}` — a heightfield raster |
| `opts.routeEnds` | `[{x,y}]` in the site box |
| `opts.primaryPaths` | `[[{x,y}…]]` in the site box |
| `opts.economy` | `{specialisation, oreBearing}` — `oreBearing` a nullable angle in radians |

No `Settlement`, no faction, no territory. The civ coupling lives **one layer
up**, in block 2's `_um*` adapter (lines 22036-22962), which turns a settlement
`p` into that object. The adapter holds **27** `_um*` functions: milestone 17's
20 (itemised there) plus 7 that "Out of scope for every milestone" excludes — 3
canvas-draw and 4 single-thread scheduling functions.

**The crate graph follows:** `cartalith-urban` never depends on `cartalith-civ`.
Its only dependencies are `cartalith-rng` and `cartalith-jsmath`, the
workspace's dependency-free leaf. The adapter lives in
`cartalith-civ::urban_adapter` (milestones 17 and 17a), the only piece that
needs civ types; `cartalith-civ` depends on `cartalith-urban`, not the other way.

### "Ports cleanly" — true of the boundary, false of the effort

| part | functions | lines | milestones |
|---|---|---|---|
| engine (block 4) | ~92 (90 top-level `function` declarations, plus `clamp` and `V`) | 2,937 | 1-16 and 8a |
| civ adapter (block 2) | 27 (20 in scope) | ~927 | 17 and 17a |
| **Phase 5** | **~119** | **~3,860** | **19** |

For scale, the Journey Planner was sized at ~70 functions / ~3,100 lines and the
Asset Library at 19 top-level functions / ~2,250 lines. Block 4 is also denser:
it is written several statements to a line, and several functions (`buildWall`
~190 lines, `grow` ~167, `buildBuildings` ~148, `applyStarFort` ~100) are single
algorithms, not dispatch tables. Read "ports cleanly" as "has no boundary
problems", not as "is small".

## What it actually generates, and how

A complete cadastral model, in this order:

1. **Site** (`buildSite`) — the setting in a fixed 1700 × 1250 m box
   (`SITE_WM`/`SITE_HM`). Either synthesises a river/coast/bay/landlocked site
   from the seed or — the host's path — wraps the real map's water mask,
   distance transform and river centreline (`opts.water`) and heightfield
   (`opts.terrain`). Returns closures: `height`, `slope`, `riverDist`,
   `isWater`, `bankSide`, plus `bridgePt`, `harbour`, `routeEnds`.
2. **Anchors** (`placeAnchors`) — the market square, scored over 400 seeded
   candidates against slope, flood band and distance from the break-of-bulk
   point (bridge or quay).
3. **Primary routes** (`buildPrimaries`) — an 8 m cost raster with a
   Tobler-flavoured slope penalty, then **A\*** from each route endpoint to the
   market, with reinforcement; or `buildPrimariesFromPaths` when the host
   supplies real roads; or, for the radial (Venus) culture, `buildRadialStreets`'
   concentric rings and spokes.
4. **Growth** (`grow`) — an epoch loop (default 8) spending a population-derived
   street-length budget on seeded candidate segments: near-perpendicular
   branching with jitter, a decaying exploration share, a market-distance
   density gradient, junction-angle and parallel-spacing rejection, bridgehead
   rules for the far bank, and optional successive wall generations gated on
   real elapsed years.
5. **Fortification** (`buildWall`, `applyStarFort`) — a curtain round the
   built-mass convex hull with gates at route crossings, or a bastioned *trace
   italienne* with a wet or dry moat, behind a population minimum and an
   anachronism guard.
6. **Cleanup** (`lanePass`, `removeWaterCrossings`, `pruneLargest`,
   `privatizeAlleys`, `clearFortZone`).
7. **Blocks** (`buildBlocks`) — planar face extraction over the street graph
   (angularly-sorted half-edge traversal with spur collapsing), each face inset
   by half the width of each fronting street.
8. **Parcels** (`buildParcels`) — series platting by vertex bisectors capped by
   ray-casts to the opposite boundary, log-normal frontages and depths, and a
   burgage re-subdivision cycle.
9. **Districts** (`assignDistricts`) and **buildings** (`buildBuildings`) —
   per-parcel footprints by building grammar, with ridge lines.
10. **Amenities** — `buildMarkets`, `buildCivic`, `buildFaithSites`,
    `buildGames`, `buildHarbour`, `addRiverBridges`.
11. **Hinterland and state** — `buildFarmland` (strip and ring fields),
    `buildDetails`, `applyDecay` (the "ruined" toggle), `computeMetrics`.

Streets **and** blocks **and** plots **and** footprints **and** districts **and**
walls **and** farmland. This port adds three stages of its own — see "Beyond
the reference".

## RNG: checked, not assumed

Block 4's header says `mulberry32` is "intentionally NOT redefined here … it
falls through to the byte-identical module-scope copy already in script block
1". There is no `mulberry32` in block 4, and the block-1 copy at line 2291 is
the one `cartalith-rng` already golden-verifies — literally the same function,
unlike Phase 2's `_civRng`, which was the same algorithm under a different
wrapper.

New here is the **seed derivation**: `stream(seed, label)` =
`mulberry32((seed>>>0) ^ fnv1a(label))`, giving labelled substreams (`'site'`,
`'anchors'`, `'grow/e3'`, `'parcels/blk7'`, …) so each stage draws
independently from one town seed. `stream` carries `range`/`int`/`pick`/`norm`/
`logn`/`chance` over one generator, so **call order is load-bearing**: `norm()`
is Box-Muller and consumes **two** draws, and `pick` consumes a draw even on an
empty array. Milestone 1 pins all of it.

## The V8 libm bill

Every transcendental `Math.*` block 4 calls must reproduce **V8's** result, not
the correctly-rounded one. ECMA-262 leaves them implementation-approximated;
V8 calls FDLIBM's `__ieee754_*`. The engine is full of threshold comparisons
(`attachPoint`'s 11 m snap, `rawEdge`'s 3.5 m minimum, `nearestNode`'s radius,
every A\* tie) where being *more* accurate than the reference is the wrong
answer, and a one-ulp difference was proved to change graph **topology**
(milestone 2). Measured against Rust's platform libm:

| JS | disagreements with the platform function | first needed | port |
|---|---|---|---|
| `Math.hypot` | 1,398 of the 4,096 integer offsets a 64 × 64 raster produces, all 1 ulp. On `(3, 3)`: true 4.242640687119285146…; `f64::hypot` 4.2426406871192847703 (correctly rounded); V8 4.2426406871192856585 (1 ulp high) | 1 | `js_hypot` |
| `Math.exp` | 20,721 of 240,000 | 5 | `js_exp` |
| `Math.sin` / `Math.cos` | 1,942 / 2,160 of 80,214 spanning every reachable reduction branch | 6 | `js_sin` / `js_cos` |
| `Math.log` | 1,647 of 60,009 | 6 | `js_log` |
| `Math.atan2` | 10,615 of 60,000 (17.7%; 20.4% over `JS_SEMANTICS_AUDIT.md`'s wider range) — the worst measured | 2 (`extractFaces`' sort key, line 28469); also `grow` (four sites) and `buildWall` (two) — seven in all | `js_atan2` |
| `Math.log10` | 960 of 60,000 | 14 (`buildCivic`'s rank scaling, line 29211 — the only call site) | `js_log10` |
| `Math.acos` | 544 of 60,000 | 10 (`cornerCut`, the only call site, feeding a threshold) | `js_acos` |
| `Math.min` / `Math.max` | on NaN: JS propagates it, `f64::min`/`max` absorb it | 4 | `js_min` / `js_max` |
| `Math.round` | negative halves: JS rounds toward +∞, `f64::round` away from zero | 6 | `js_round` |

`Math.pow(x, 2)` measured bit-identical to `x * x` on 60,000 arguments;
`Math.sqrt`, `abs`, `floor`, `ceil` and `sign` are exact by specification.

All of these live in `cartalith-jsmath` and are re-exported through
`cartalith-urban::geom`, which is the name every call site in this crate uses.
Their V8 goldens moved with them. The `hypot`, `exp`, `sin`/`cos`, `log` and
`round` ports carry rows the platform function is asserted to get wrong, so a
test fails if anyone "simplifies" one away. This is
`cartalith-rust-conventions`' float rule: match the reference, do not improve on
it.

---

## Milestones

Dependency-ordered. Each is self-contained and independently verifiable; the
reference's `_test` export and `hashModel` make most of them golden-verifiable
rather than hand-checked. 8a and 12 were built out of dependency order, and 17a
before 8-16 — each section says why.

### Milestone 1 — RNG substreams + geometry kernel

Reference lines 28177-28191 (`fnv1a`, `stream`) and 28282-28360 (`V` and the
polygon helpers), plus `convexHull` (29639-29646), which sits inside milestone
10's range but landed with this kernel. Modules `rng` and `geom`.

`fnv1a`, `stream` and its six derived draws; `V` (as `Vec2`), `polyArea`,
`polyCentroid`, `pointInPoly`, `segInt`, `distPtSeg`, `polySelfIntersects`,
`chaikin`, `simplify` (Douglas-Peucker), `ensureCCW`, `insetPoly`,
`clipConvex`, `convexHull`, and `js_hypot`.

Two reference behaviours are pinned as behaviours, not fixed as bugs:
`clipConvex` clips against the clip **segment** rather than the clip line (so a
subject poking past the window's corners can collapse to empty), and
`insetPoly` returns nothing at all — not a degenerate polygon — below area 15 or
on self-intersection at ≤60 vertices. Downstream code reads both.

`polySelfIntersects` is on neither `UME` export, so its test is a unit test of
the ported logic and is labelled as one; every other test here is golden
against the reference's output.

**Milestone 1's goldens passed on the wrong libm twice.** `rng::logn`
(`median * Math.exp(sig * norm())`) was on `f64::exp` until milestone 5, and
`rng::norm` (`Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * PI * u2)`) on
`f64::ln`/`f64::cos` until milestone 6. The goldens had landed on values the two
libms agree about — luck, not safety. They still pass on `js_*`, which is the
check.

### Milestone 2 — planar street graph

Reference lines 28362-28511 (`makeGraph` from 28363). All 15 functions:
`makeGraph`, `gKey`, `gridCellsForSeg`, `indexEdge`/`unindexEdge`/`edgesNear`,
`addNode`, `nearestNode`, `rawEdge`, `splitEdge`, `attachPoint`, `addStreet`,
`addPolylineStreet`, `extractFaces`, `edgeBetween`. Module `graph`.

- **Dense `Vec` with tombstones; ids never reused.**
- **`nextN`/`nextE` are not stored.** They are always `nodes.len()` and
  `edges.len()` — every increment is paired with a `push`, nothing is removed —
  and the capture asserts that against the reference's own counters on every
  scenario.
- **`gKey` does not survive as a function.** It only makes a `Map` key from two
  integers; an `(i64, i64)` tuple is the same partition, and the grid is only
  ever probed, never iterated, so no ordering is lost. 15 functions land as 14
  Rust items.
- **`cls` is `&'static str`, not an enum.** The reference compares it by string
  in six places and `hashModel` serialises it verbatim, so the string is the
  value; an enum would have had to guess the classes later milestones add
  (`'ringroad'`, `'lane'`, and `grow` passes a variable).

#### Golden verification

`UME._test` reaches `makeGraph`, `addStreet` and `extractFaces`, which is enough
for all fifteen because the harness dumps the **entire** graph state after each
scripted scenario: every node with its adjacency, every edge including
tombstoned ones, the uniform grid cell by cell, and the extracted faces.
`attachPoint`, `rawEdge`, `splitEdge` and `nearestNode` live inside
`addStreet`; the index family's whole observable effect is the grid.

**Mutation-checked**, because a full-state dump can look thorough and still be
vacuous. Perturbing the 26 m index cell, the 0.7 cell step, the 3×3 cell
dilation, the 11 m node snap, the 9 m edge snap, both 3.5 m guards, the 2.5 m
node-promotion radius, the `[0.03, 0.97]` t clamp, the spur collapse's stack
rule, the outer-face tie-break's strict `>`, and swapping `js_hypot` for
`f64::hypot` each break at least one golden. Two scenarios (`clampT`,
`hypotSnap*`) exist only because the first round found those constants
unexercised.

#### Findings

1. **Encapsulation, verified across all of block 4**: `cell`, `grid`, `nextE`
   and `nextN` are touched only by this milestone's functions; no later
   milestone reaches into the spatial index. `nodes`/`edges`/`adj` are read
   widely, always as `n.adj.filter(id => g.edges[id].alive)`.
2. **`g._fromPaths` is a dynamic property, and it needs a real field.**
   `buildPrimariesFromPaths` sets it (line 28830, milestone 6) and
   `builtMassHull` reads it (line 29709, milestone 10) to discount the bare
   degree-2 vertices a resampled real road drags in; without it the enceinte
   over-encloses along arterials, as the reference's own v1.01 note describes.
   Here it is `Graph::from_paths`.
3. **The reference is inconsistent about one splice, and the port reproduces
   it.** `splitEdge` removes an edge from `a.adj` with an **unguarded**
   `splice(indexOf(e.id), 1)`, where a miss would silently drop the *last*
   element (JS `splice(-1,1)`); milestone 11's `_killEdge` guards the identical
   splice with `if (k >= 0)`. Unreachable given `rawEdge`'s invariant. **Do not
   unify them.**
4. **`addStreet` leaves orphan nodes.** When both endpoints are fresh and every
   resulting link is rejected by `rawEdge`'s 3.5 m minimum, the nodes stay in
   `g.nodes` with empty `adj` (golden `tooShort`: 4 nodes, 1 edge). Every later
   pass must keep filtering on live adjacency.
5. **The stable hit sort in `addStreet` is a safety property; a tie is
   unreachable.** Two crossings at one `t` are one point, so those edges already
   crossed and share a node whose half-edges the `1e-4` guard excludes. Two
   on-segment nodes at one `t` lie on one perpendicular within 2.5 m of the
   segment, hence within 5 m of each other, which the 11 m snap prevents. A
   crossing tied with a node is at that node's foot, ≤2.5 m away, which
   `splitEdge`'s 3.5 m guard folds back. Confirmed by mutation (an unstable sort
   changes no golden) and by a test re-deriving every hit parameter.
6. **Two `addStreet` constants are redundant inside the site box.** The `1e-4`
   interior-crossing guards and `1e-3` node-parameter guards survive being
   loosened to `1e-9`: a hit at `t = 1e-4` is `1e-4·L` from an endpoint, so it
   escapes `splitEdge`'s 3.5 m fold-back only past `L > 35 km` (the node guard
   past `L > 3.5 km`) against a 1700 × 1250 m box. Kept — they stop the
   degenerate case if the box ever grows.
7. **`extractFaces`' guard drops, it does not truncate.** JS
   `while (guard++ < 20000)` leaves `guard` at 20001 when the bound stopped it,
   so the post-check `guard >= 20000` also discards a face that closed on step
   20000 exactly. Reproduced as written.
8. **The outer-face tie-break is observable.** A closed loop with one dead-end
   spur yields two faces of equal absolute area (±14400 on the golden), and the
   strict `>` makes the *lowest-indexed* one outer. `buildBlocks` skips the
   outer face, so this is not cosmetic.

### Milestone 3 — A\* over the cost raster

Reference lines 28513-28547 (`astar` from 28514). Module `astar`.

The plan said the heap's tie-breaking is what makes the path reproducible, so it
is ported literally rather than swapped for `BinaryHeap`. This milestone proved
that rather than asserting it — after the verification method itself failed
first.

#### The goldens were vacuous, and mutation testing said so

Seventeen hand-written scenarios — degenerate 9 × 1 and 3 × 17 strips, both
rectangle orientations, a 500-cost wall with one gap, an infinite moat, a NaN
band and a NaN seal, a zero-cost field, start-equals-goal, two rasters filled by
the reference's own `stream`, and every cell of a 6 × 5 raster as the goal —
all reproduced the reference on the first run. Then fifteen mutations ran
against them and **nine survived**: the `0.9` heuristic weight, the `0.5`
trapezoid factor, the `DIRS` order, all three heap-comparator tie-breaks,
`js_hypot` vs `f64::hypot`, the `if (i === gi) break` early exit, and the dead
`INFINITY` guard.

> A **continuously-valued** cost raster essentially never produces two frontier
> entries with exactly equal `f`, so it cannot observe a tie-break at all. Only
> a **quantised** raster can.

An exhaustive search over ~800,000 (raster family × size × endpoint pair)
combinations found a discriminator for every survivor, and every tie-break
discriminator came from a quantised field — costs from `{0.5, 1}`, `{1, 2}` or
`{1, 2, 3, 4}`. Eight such scenarios (`tiesHalf`, `tiesLeft`, `tiesRight`,
`tiesWide`, `tiesDiag`, `nearAdmissible`, `trapezoidal`, `greedyTrap`) took it
to **fourteen of fifteen dead**. Not an artificial regime: `buildPrimaries`'
raster is `(1 + (slope·3.2)²)·8` and a site is mostly flat, so the real 8 m cost
field is mostly constant — the tie-heavy case is the normal one. The one-ulp
`hypot` difference bites only when it makes or breaks an exact `f` tie, which
took a 64 × 48 quantised raster (`tiesWide`) to observe; the requirement is also
asserted directly.

The survivor is deleting `if (g0[i] === Infinity) continue;`, and it is
**unreachable in the reference too**: `g0[ni]` is assigned on the line before
every `push`, and `g0[si]` before the start's own push, so no popped index still
holds the fill value. The line is kept because the reference writes it; a test
asserts the invariant it depends on (no relaxation writes a non-finite `g`)
across the infinity and NaN scenarios.

#### What the reference's A\* actually is

Written down because a later reader will otherwise "fix" it:

- The heuristic is `0.9 ×` Euclidean distance **in cells**, while a step costs
  the trapezoidal mean of two metre-scaled raster values (`c·CS`, ~8-2000) — so
  it is wildly *under*-weighted normally and *over*-weighted where the raster is
  cheap.
- There is **no closed set** and no stale-entry check; cells are re-expanded.
- `if (i === gi) break` stops on the first *pop* of the goal, which under an
  inadmissible heuristic need not be its cheapest path.

The search is **reproducible, not optimal**, and the golden path is the
specification. A correctness-improving rewrite would move every primary route,
and every block, parcel and building grown against it.

#### Non-finite cost is the only route to `null`

An 8-connected full grid has no unreachable cell, so `astar` returns `null` only
by arithmetic: an `Infinity` or `NaN` tentative cost fails `c < g0[ni]` — every
comparison against NaN is false in Rust exactly as in JS. Goldens `moat` and
`nanSeals`; one of the few places where JS and Rust NaN semantics agreeing is
load-bearing.

#### One deliberate divergence

An out-of-range `start`/`goal` **panics** here. The reference reads past its
typed arrays, gets `undefined`, sails past its own guard (`undefined ===
Infinity` is false) and produces nonsense. Its only caller (`buildPrimaries`'
`toCell`) clamps to `[1, W-2] × [1, H-2]` first; loud beats silent for a case
that cannot happen.

The capture refuses to write unless every path is non-empty, starts at its start
cell and ends at its goal, the two sealed scenarios really returned `null`, and
the capture exceeds 300 path cells.

#### Corrections to later milestones

1. **Milestone 6 must not "improve" the search.** `buildPrimaries` runs `astar`
   once per route endpoint over a **copy** of the cost raster with already-used
   cells multiplied by `0.45`, so the reinforcement is order-dependent on
   `site.routeEnds` and each run inherits the previous run's exact cell set.
2. **This port's `astar` takes `(usize, usize)` cells and panics out of range**,
   so milestone 6 reproduces `toCell`'s clamp
   (`max(1, min(W-2, round(p.x/CS)))`) itself.
3. **Milestones 12 and 13 compare areas and lengths against thresholds**;
   goldens on continuous random inputs will not exercise their tie-breaks
   either. Build at least one quantised or symmetric fixture per milestone.

### Milestone 4 — generation rules + culture profiles

Reference lines 28193-28280 (`CULTURE_PROFILES` from 28199). `CULTURE_PROFILES`
(medieval/organic and Venus/radial), `resolveProfile`, `DEFAULT_RULES`,
`cloneRules`, `resolveRules`, `clamp`, `applyWildness`, `applyPlotChaos`. Module
`rules`.

Data, not algorithm — and it holds the most dangerous line in the subsystem.
Seven of the eight are on `UME`'s **public** export (lines 31096-31097) and
`clamp` is observed through `applyWildness`/`applyPlotChaos`, so no indirection
was needed.
The source engine later gave this table a host UI and one new field
(`RC_ENGINE_CHANGES.md` §6r.1; §6s.3's `parcels.wardGrain`); this port's host
sets `opts.rules` itself (see "Beyond the reference").

#### `clamp` is where a naive port silently builds a different town

`const clamp=(v,lo,hi)=>Math.max(lo,Math.min(hi,v));` The obvious Rust,
`lo.max(hi.min(v))`, is **wrong**: JS `Math.min`/`Math.max` *propagate* NaN,
Rust's `f64::min`/`max` *absorb* it. So `applyWildness(rules, NaN)` leaves eight
NaN street fields in the reference, while the naive port's inner `hi.min(NaN)`
returns `hi` and lands **every clamped field on its own upper bound** — a
maximally-wild rule set that looks entirely plausible, fed straight into `grow`.
(`cartalith-assets` milestone 3 hit the same trap from the other direction.) The
port routes `clamp` through `js_min`/`js_max`, goldens `wild_NaN` and
`chaos_NaN` pin it, and a test carries the `assert_ne!`-style device with the
reason written out.

`f64::clamp` would have agreed on every reachable input (it propagates NaN), but
it panics when `min > max` where the reference returns `lo`. **One documented,
unreachable divergence remains**: `Math.min(+0,-0)` is `-0` and
`Math.max(+0,-0)` is `+0`, where the comparison form returns whichever operand
`<` lands on. Only two of the eleven clamps have a zero bound, and neither can
reach a `-0` argument (`0.10*(2-w)` is `-0` only if `2-w` is, which subtracting
two finite doubles never produces; `deadEndBias+(w-1)*0.15` is `+0` at `w == 1`).
It is why two mutations survive, below.

#### Findings in the data

1. **`applyWildness` is not idempotent, because of one field.** Ten of eleven
   assignments recompute from a hardcoded literal times `w`; `deadEndBias` is
   `clamp(s.deadEndBias + (w-1)*0.15, 0, 0.40)` — it reads its own value and
   **accumulates**. Five applications of `w = 2` walk it 0.15 → 0.30 → 0.40
   (capped) while nothing else moves. Pinned by `wildTwice1p5`, `wildThrice2`,
   `wildFive2`; `applyPlotChaos` by contrast is idempotent.
2. **The sliders overwrite custom values they never read.** A custom
   `branchAngleJitter` set through `resolveRules` is lost to `applyWildness`,
   whose base is the literal `0.26`. Pinned by `wildOverCustom`.
3. **Four `street` fields and two whole groups are untouched by either slider**
   — `explorationDecay`, `segmentLengthMedian`, `marketGradientDecay`,
   `bridgeheadDistance`, all of `settlement`, and the other slider's group.
   Asserted, so a later milestone that finds one moved knows it did not come
   from here.
4. **`profile.deadEndBias` exists on neither live profile.** `privatizeAlleys`
   (line 30097, milestone 11) reads
   `clamp((profile.deadEndBias||0) + (rules.street.deadEndBias||0), 0, 0.40)`,
   so the profile side is *always zero* — the hook for the removed 17 profiles.
   The capture asserts the absence against the reference's own key list; the
   port carries the field as `0.0` so milestone 11 writes the expression as the
   reference does.
5. **Four profile fields are read by nothing at all**, verified across block 4
   *and* the host app: `parcelPattern` (its death is documented at lines
   30225-30227), `orientation`, `civicAnchorLabel` and `defaultWalls`. `venus`'s
   own `prov` says "the UI unchecks the wall box on selecting this profile", and
   `defaultWalls` has zero reads in v2.10. All four are carried with the note
   that killed them. `defaultWalls` is `Option<bool>` so "no opinion" (key
   absent, `medieval`) stays distinct from "says no" (`venus`); `waterway`,
   read only as a truthiness test, is a plain `bool`.
6. **Nothing outside block 4 uses this milestone's exports.** The host touches
   exactly three names on `UME` — `SITE_WM`, `SITE_HM`, `cityGen`. These are
   exported for the reference's headless tests and consumed internally only by
   `generate()` at lines 30933-30934.
7. **`resolveProfile` has a prototype-chain hole, and the port hardens it.**
   `CULTURE_PROFILES[id]` indexes an object literal, so five `Object.prototype`
   names come back truthy and pass the `||` fallback:
   `resolveProfile('toString')` returns a function, `'__proto__'` returns
   `Object.prototype`, and `generate()` would crash at
   `profile.wallGates.scheme`. All five are captured as the reference's real
   behaviour, and a golden asserts this port returns `medieval` for each. A
   `match` has no prototype chain.
8. **`cloneRules` is `JSON.parse(JSON.stringify(r))`** — `#[derive(Clone)]` on a
   well-formed rule set, except that a NaN round-trips to `null`, which the
   capture pins. A typed `Rules` has no `null`, so the port keeps the NaN.
   Unreachable in the engine: `resolveRules` clones the all-finite
   `DEFAULT_RULES` and assigns the caller's partial on top, so nothing a caller
   supplies is round-tripped.
9. **`subdivisionCap` stays an `f64`.** `applyPlotChaos` writes
   `Math.round(clamp(2*c,1,4))`, NaN for a NaN slider, and milestone 12 reads it
   only through `Math.min(P.subdivisionCap, Math.floor(age/3))`, where NaN runs
   the re-subdivision loop zero times. A `u32` would have to decide what NaN
   becomes. `Math.round` is safe as `f64::round` on this domain (`[1,4]` plus
   NaN); the goldens include the three `c` values landing exactly on 1.5, 2.5
   and 3.5.
10. **`resolveRules` merges per *field*, not per group**, and skips a falsy
    group wholesale (`if(partial[grp])`). Two structural differences from
    `Object.assign`, both unobservable: an unknown *group* is ignored (a typed
    patch has none), and an unknown *field* inside a known group is copied
    where nothing reads it. Pinned by `resolveUnknownGroup` and
    `resolveFalsyGroups`.

#### Mutation testing: 120 mutations, 114 dead, 4 survivors, 2 killed by the compiler

Every numeric literal on a non-comment line (84), plus 36 structural mutations:
both clamp semantics, both `js_min`/`js_max` comparators, every `js_round`
alternative, the `deadEndBias` accumulation, the `2-w` inversions, both `meta`
write-backs, `resolveRules`' merges, `resolveProfile`'s fallback and arm order,
and eleven profile-table values including the profile array's order. Two are
killed by the compiler: `[CultureProfile; 2] → 3` and `[f64; 24] → 25`.

| survivor | why it survives |
|---|---|
| `js_min`'s `b < a` → `b <= a` | the branches return numerically identical values when `a == b`; which operand matters only for `+0` vs `-0`, the documented unreachable divergence |
| `js_max`'s `b > a` → `b >= a` | same |
| `clamp(2*c, **1.0**, 4.0)` → `1.01` | `subdivisionCap` is a **quantised output**: a rounded value cannot see an input change smaller than half its step. `1.0 → 1.6` and `1.0 → 0.0` both **die** |
| `clamp(2*c, 1.0, **4.0**)` → `4.01` | same; `4.0 → 4.4` survives, `4.0 → 4.6` and `4.0 → 3.0` **die** |

A fifth survivor — the `2` in `clamp(2*c,1,4)` — was killable, and three
scenarios killed it: `chaos_0p7475`, `chaos_1p2475`, `chaos_1p7475`, just
*below* the rounding boundaries `chaos_0p75`/`chaos_1p25`/`chaos_1p75` sit on.
Milestone 3 needed a quantised *input* to see a tie-break; this needed an input
just below a quantised *output's* boundary. Both are one fact: *a golden can
only test what its inputs let the function express.*

#### Golden verification

53 rule cases (defaults, the clone, ten `resolveRules` merge shapes, fifteen
`applyWildness` arguments including all three non-finite ones, four
repeat-application cases, seventeen `applyPlotChaos` arguments, five combined
sequences), both profiles field by field including the two keys `medieval`
leaves off, and fifteen `resolveProfile` ids. Rule sets are flattened into one
canonical field order and compared **bit for bit** via `to_bits`, so a NaN must
be a NaN and a `-0` cannot pass for a `+0`. The capture asserts the reference's
`DEFAULT_RULES` still has exactly that key set in that order (so an upstream
rule cannot silently drop out) and that neither profile defines `deadEndBias`;
its gate requires ≥40 scenarios, all the right width and numeric, ≥30 differing
from the defaults, exactly two profiles with non-empty provenance, and
`applyWildness(NaN)` really poisoning the set.

#### Corrections to later milestones

1. **`grow` falls back to the raw table**: `const rules = opts.rules ||
   DEFAULT_RULES` (line 29446), not a `resolveRules` call. Reproduce it.
2. **`privatizeAlleys` gets zero from the profile side** — finding 4.
3. **`subdivisionCap` is read as a float** (finding 9), and
   `Math.min(P.subdivisionCap, Math.floor(age/3))` must keep NaN-propagating
   semantics if restructured.
4. **`profile.id` and `profile.name` are values, not labels.** `id` keys
   `GAMES_SPEC` (line 29278) and `FARM_SPEC` (lines 30775, 30887); `name` is
   `cultureName`. That is why `CultureProfile`'s fields are `&'static str`, the
   call milestone 2 made about `Edge::cls`.

### Milestone 5 — the site model

Reference lines 28549-28741 (`shoreFromMask` from 28557). `shoreFromMask`,
`buildSite`, `terrainSuitability`. Module `site`.

#### `Math.exp`, the second libm divergence

The first golden run failed on one probe of one site, one ulp out — as milestone
1's first `dist_pt_seg` failed on `hypot`. FDLIBM's `__ieee754_exp` promises
under one ulp, not correct rounding. **One measured special case, reported
rather than explained:** across 244,000 arguments (240,000 random, every half-
and quarter-integer to ±20, and `1.0` at ±1 and ±2 ulp), V8 and FDLIBM agree
everywhere **except exactly `x == 1.0`**, where V8 returns the correctly-rounded
`e` and FDLIBM one ulp above it. Unreachable from the site model, whose `exp`
arguments are `-(d²)/(2σ²)`, never positive.

**`rng::logn` moved to `js_exp` here.** It has **five call sites** — 29524 in
`grow`, 30242 and 30288 in `buildParcels`, 30523-30524 in `buildBuildings` — so
every frontage width, plot depth and building dimension is drawn through it. The
next direct `Math.exp` is `logisticRamp`'s `1/(1+Math.exp(...))` (line 29392,
milestone 7).

#### Findings

1. **`buildSite` is two sites wearing one name, and which is live is decided per
   *field*.** A real water mask with no river centreline still runs the
   synthetic hills; a real heightfield with no water context still invents a
   synthetic channel. The port carries `Option<WaterCtx>`/`Option<TerrainCtx>`
   rather than one source enum, which would lie about the mixed cases the host
   produces. Four goldens are mixed on purpose.
2. **`kind` is not a closed vocabulary.** `kind = kind || 'river'` defaults only
   the falsy case; an unrecognised string falls through to the **coastline**
   branch while being returned verbatim — and milestone 9 compares
   `site.kind === 'coast'` directly (lines 29061, 29081), so an unknown kind and
   a real coast are different sites. `kind` stays a `String`. Pinned by `atoll`,
   which shares a seed with `coast` and produces a byte-identical shoreline under
   another name.
3. **A river path needs two points to make a site river-bound** (Ruling N,
   2026-09-20, which names this as the render half of the settlement
   river-binding defect). `build_site` reads `WaterCtx::has_real_river_path` —
   the predicate its own geometry branch and `cartalith_civ::um_water_ctx`
   already applied — for both `rk` and `real_river`. `pathOfOne`/`pathEmpty` are
   deliberately re-baselined: `golden_build_site` skips exactly those two
   (failing if either name stops existing) and
   `a_short_river_path_now_draws_as_no_river_at_all` asserts the new behaviour.
   No production world reaches the case: `um_water_ctx` assigns `river_path`
   only inside its own `hi - lo + 1 >= 2` guard. *(Superseded 2026-09-20: the
   reference's `!!W.riverPath` truthiness, under which an empty or one-point
   path made a site river-like. Do not restore it.)*
4. **A bay draws one fewer number than a coast.** The coastline branch draws its
   harbour abscissa only when the site is not a bay (a bay reuses its indent
   centre), so `bay` consumes 31 site-substream draws and `coast` 32, and their
   `routeEnds` diverge. `bay` and `coast` share seed 5 on purpose, and a test
   advances a fresh stream by hand through the whole budget (12 hills, the
   branch's own, then 3 or 4 endpoints) and rebuilds the endpoints.
5. **One mask, two truthiness tests.** `shoreFromMask` takes any non-zero cell
   as water; `isWater` tests `=== 1`. A cell holding `2` is water to the tracer
   and land to the query. Reproduced — golden `maskTwo`.
6. **`shoreFromMask`'s principal axis can collapse to `(0, 0)`.** One water cell
   in a 5 × 5 land field leaves four shore points with an isotropic scatter:
   `sxy == 0`, `l1 - sxx == 0` and `l1 - syy == 0`, so the fallback eigenvector
   is degenerate too, the `|| 1` on the axis length fires, every projection is
   zero, and the stable sort returns row-major order.
7. **The fallback eigenvector is ordinary.** With `sxy == 0` and `sxx > syy`,
   `(sxy, l1-sxx)` is `(0, 0)` on every symmetric coast. It is unobservable
   unless the shore has points in **two** rows (sorting a row-major list by y is
   the identity); `twoRowShore` is the fixture that sees it.
8. **Out of bounds is `undefined`, not a panic**, reached three ways: a NaN
   probe coordinate, a `dt` array shorter than its mask, and a terrain raster
   with `mw < 2`. All become `f64::NAN` here (goldens `shortDt`,
   `terrainShortGrid`, `terrainOneColumn`) — the deliberate divergence taken
   **the other way** from milestone 3's panic: quiet here because the case can
   happen.
9. **`bankSide` never returns 0.** `Math.sign(x) || 1` sends a point on the
   centreline, a `-0` and a NaN cross product to `+1`. `grow`'s bridgehead rule
   and `buildWall`'s far-bank test read it, so that definite answer is
   load-bearing. Swept over every vertex of every golden site.
10. **The bridge index starts at `-1`, and `Math.max(0, bi)` alone places the
    bridge when no slope ever compares.** An all-NaN heightfield never satisfies
    `s < bs`, so the bridge lands on `river[0]`; `terrainAllNaN` exists for it.
11. **The three analytic hills are drawn even when a real heightfield makes them
    dead** — twelve draws nothing reads, but twelve *positions* in the site
    substream. Skipping them on the real-terrain path would move every route
    endpoint.
12. **`waterPoly` is empty on two of the four paths** (landlocked, and coastal
    with real water), and nothing in block 4 reads it — only `generate()`'s
    return object (line 31081), i.e. the renderer. It is not the town's water.
13. **Six `||` defaults; only the NaN arm ever bites:** `riverWidthM || 20`,
    `riverOrder || 0`, `seaLakeCells || 0`, `hMax || 0`, `hMin || 0`, and
    `terrainSuitability`'s `site.riverW || 0`. A `0` width becomes 20
    (`widthFallbackZero`); a NaN Strahler order becomes 0 (`orderNaN`).

#### Golden verification

None of the three functions is on either `UME` export — the first milestone to
need the anchored `return {` replacement and the `globalThis.__UME` handoff
(both under "Verification convention"). Rasters are emitted into the golden
file so both sides run on identical inputs; everything is compared bit for bit,
including `height` and `slope`, which run through `exp` and `js_hypot`.

The capture refuses to write unless: ≥19 shore and ≥30 site scenarios; ≥3
shorelines `null` and ≥6 non-trivial; `plusShape` really came back row-major
(the tie fixture is tying); every site's river has ≥2 finite vertices, 3 or 4
route endpoints and 106 probes; height varies across a site's probes; some probe
is wet and some site is dry everywhere; `bankSide` took both signs;
`terrainSuitability` reached 0 and >0.5; the NaN probe produced a NaN; `bay` and
`coast` drew different endpoints; `atoll` took the coast branch under its own
name; both `riverWidthM` fallbacks landed on 20 while an explicit 26 survived;
`orderNaN` zeroed; the all-NaN slope field fell back to `river[0]`;
`pathOfOne` is river-like in the reference without being the river (finding
3); `landlocked` has no harbour and no water polygon; the short `dt` produced a NaN; and a mask
of 2s read as land. The Rust side mirrors the shape half as its own test.

#### Mutation testing

Every numeric literal on a non-comment line of `site.rs` (207) plus 64
structural mutations: every `js_min`/`js_max`/`js_hypot`/`js_exp` call site,
every comparator and tie-break, both Chaikin passes, the draw order and count,
all six `||` defaults, the two mask truthiness tests, the sort's stability, the
fallback eigenvector and the bilinear term order.

**The first sweep left 46 survivors, and almost none were equivalent mutants —
they were fixture gaps of two shapes:**

- **Every water raster was uniform along one axis.** `j >= 9 ? water : land`
  makes every mutation of `maskIdx`'s `i` clamp invisible. Fixed with a
  per-column ripple so **no two adjacent columns agree**.
- **A fixed fractional probe grid never lands near anything.** The interesting
  geometry is a 10-40 m band round a polyline, which a `[0.1, 0.5, 0.9]²` grid
  never enters. Fixed by deriving probes **from the site's own river**: offsets
  straddling the band at three points along it, and a ±0.25 m ladder either side
  of the real waterline at nine abscissae (`yAtX` reimplemented in the capture —
  fixture code, not ported code).

Probe count went 16 → 106 and scenarios 13/31 → 19/36 across three rounds, and
survivors **46 → 35 → 31**. **271 mutations, 240 died (2 at the type level:
`[Hill; 3] → 4` and the transposed raster index), 31 survived**, all re-run in
isolation.

| class | n | why they survive |
|---|---|---|
| dead stores | 10 | `[Hill { x: 0.0, y: 0.0, amp: 0.0, rad: 0.0 }; 3]` — four initialiser fields, all overwritten by the loop below; `harbour_idx`'s declaration and its landlocked assignment (both overwritten, and a landlocked harbour is `{idx: -1}` regardless); the `Harbour { idx: 0, pt: None }` placeholder, replaced at the end of `build_site`; `bi = -1 → -2`, read only through `.max(0)`; and both `Vec::with_capacity(n + 1) → n + 2`, which is capacity, not length |
| equivalent by the surrounding arithmetic | 6 | `i0 + 1.0 → i0 + 1.5` on both bilinear axes — `i1` is used only as `i1 as usize` and `i0` is integral, so truncation erases it, and the integral `js_min` bound cannot be crossed by half a step; `if s > 0.0 → s > 1.0` in `bank_side` — the true arm and the final `else` **both return `1.0`**; and all three forms of `vl`'s `\|\| 1` (the `== 0.0` test, the `1.0` substitute, removing it) — it fires only when the axis length is zero, where `vx` and `vy` are both zero, and `0 / anything` (including `0 / 0`, whose NaN comparator results map to `Ordering::Equal`) leaves every projection tied |
| boundary tests whose two branches compute the same value | 2 | `y_at_x`'s `x <= c[0].x → <` and `x <= c[i+1].x → <`. At a vertex abscissa the early return gives `c[i].y` and the interpolation `c[i].y + 0 · Δ`; at the far end, `c[i+1].y` and the next segment's `c[i+1].y + 0 · Δ` |
| guards against data the reference cannot produce | 6 | the `(c[i+1].x - c[i].x) \|\| 1` denominator, both halves — needs two shoreline vertices at one `x`, and the abscissae are `i/26 · Wm` through two Chaikin passes, strictly increasing; the degenerate-axis test's `1e-9` — observable only if `hypot(sxy, l1-sxx)` lands in `[1e-9, 1.5e-9)`, and every fixture gives exactly `0` or more than `1e-8`; removing `max(0, ·)` from the eigenvalue — `tr²/4 - det` is algebraically `((sxx-syy)/2)² + sxy²`, never negative (**its constant is not dead** — `0.0 → 1.0` dies on `twoRowMicro`); `river[idx] \|\| river[0]` losing its fallback — `harbour_idx` is valid on every path; and the drift clamp on `f64::min`/`max`, which differ from `js_min`/`js_max` only on NaN, and the drift is a sum of finite draws |
| need an exact tie a continuous field cannot produce | 4 | the coast harbour search's `<` → `<=` (two shore vertices exactly equidistant from a drawn abscissa); `isWater`'s channel band `<` → `<=` (a distance exactly `riverW/2 + 2`); and `js_hypot → f64::hypot` at two call sites, where one ulp flips a strict `<` only if two candidates already agree to within one ulp. Milestone 3's finding, but not closable by a quantised raster — these are polyline distances, not cell costs |
| unobservable through Rust's stable sort | 3 | `else if d > 0.0 → d > 1.0` in the comparator, the same rewritten to compare projections rather than their difference, and `sort_by → sort_unstable_by`. The first was **checked**: Rust's stable sort reaches every ordering decision through the `Less` arm, so downgrading `Greater` to `Equal` still sorts fully (verified on a 16-element `f64` vector with gaps below the mutated threshold). The second differs only on NaN/±∞ projections. `sort_unstable_by` survives because the only fixture with ties between distinguishable points is the fully degenerate `plusShape`, whose four projections are all zero |

Fifteen first-sweep survivors were killed by purpose-built fixtures rather than
argued away:

| fixture | the constant it makes observable |
|---|---|
| a per-column ripple in every water mask | both of `mask_idx`'s `i`-axis clamps |
| probes derived from the site's own river | the whole `riverW/2 + 2` band, and `bank_side` on both banks |
| a ±0.25 m ladder round `yAtX(x)` at nine abscissae, plus one probe exactly on it | the sea half-plane's `-1.0` offset and its `>` |
| `riverCeiling` / `throughCeiling`, found by **scanning** seeds | the channel drift's upper clamp — no hand-picked seed saturates it |
| `quayLadder`, 18.85 m per segment (five = 94.25 m) | the quay walk's 95 m stop and its accumulator's start |
| `twoRowShore` (water along the top edge and the bottom rows) | the fallback eigenvector |
| `twoRowMicro`, the same cloud at 4 mm cells | the eigenvalue guard's own `0.0`, by pushing the discriminant below 1 |
| `vertShore`, a vertical real shoreline | the real-water harbour search's reference y |
| `exactlyTwo`, a mask producing exactly two shore points | `pts.len() < 2` |
| `northWater` / `westWater` | the north and west adjacency tests, individually |
| `twoPointPath` | `riverPath.length >= 2` |

#### Corrections to later milestones

1. **`placeAnchors`' literal fallback is live.**
   `site.bridgePt || (site.harbour && site.harbour.pt) || {x: Wm*0.52, y: Hm*0.42}`
   — a landlocked site has neither.
2. **Milestone 9 compares `site.kind === 'coast'` as a string** (finding 2); an
   enum mapping unknown kinds to `Coast` would change those two branches.
3. **`site.waterPoly` is not the water** (finding 12). Use
   `isWater`/`riverDist`.
4. **Milestones 12 and 13 draw every dimension through `rng::logn`.** A
   whole-town divergence that looks like noise: check the libm first.

### Milestone 6 — anchors and primary routes

Reference lines 28743-28833 (`placeAnchors` from 28744). `placeAnchors`,
`buildPrimaries`, `buildPrimariesFromPaths`. Module `routes`. The first
milestone to produce a real street graph end to end, so the first whose golden
is a whole-subsystem artefact.

#### `Math.sin`, `Math.cos`, `Math.log` — measured before trusting

Milestones 1 and 5 found their libm divergences **after** a golden failed; this
one measured first (numbers in "The V8 libm bill"), which is why every scenario
matched on its first run. `Math.sin` and `Math.cos` are the third and fourth
most-used functions in block 4 (27 and 26 call sites, after `Math.min`/`max`),
and `placeAnchors` calls both on each of its 400 candidates.

`rng::norm` moved to `js_log`/`js_cos` here. It is the highest-leverage function
in the subsystem — `logn` sits on it.

**One branch is deliberately not ported, and says so.** For
`|x| >= 2^19 · π/2` (~8.2e5) FDLIBM switches to Payne-Hanek reduction
(`__kernel_rem_pio2`, a hundred-odd lines of multi-precision arithmetic over a
66-word table). Every trig argument here is built from `range(-PI, 0)`,
`i/n * 2PI`, an `atan2` result or a bearing, none of which leaves `[-4π, 4π]`,
so the branch would be dead code with a real chance of being silently wrong.
`js_sin`/`js_cos` hand off to the platform libm above the threshold, a test
asserts they do, and the doc comment names it as the one unreproduced input
class.

#### Findings

1. **Neither route builder draws a random number.** Both take a `seed` and
   neither reads it — asserted by running each with a wildly different seed and
   requiring a byte-identical graph. `placeAnchors` draws exactly **800** times,
   two per candidate **before** any rejection test, so the sequence is
   independent of the site's shape.
2. **Both return values are dead.** `generate()` calls whichever builder applies
   for its effect on `g` and discards the routes (lines 31022-31023). Returned
   anyway: they make a stricter golden than the graph alone.
3. **The two builders disagree about their return shape.** `buildPrimaries`
   pushes `{pts, i}`; `buildPrimariesFromPaths` pushes `{pts}`. Carried as
   `Route { pts, i: Option<usize> }` rather than erased.
4. **`riverthrough` shares `river`'s candidate band but not its preferred
   distance.** `dBand` tests `kind === 'river' || kind === 'riverthrough'` and
   widens to `[60, 240]`; the score's `Math.abs(d - (kind === 'river' ? 120 : 100))`
   tests `'river'` alone. So a bisecting river prefers 100 m like a coastal town
   while still reaching 240 m. Two fixtures share seed 7 for this.
5. **The market reference's third `||` arm is live** (landlocked); two fixtures
   reach it.
6. **`best === null` is reachable, only on a small box.** If all 400 candidates
   are rejected the market goes to `{ref.x, ref.y - 120}` — the only place the
   market can land **outside the site box** (at 150 × 150, y = -57).
7. **The 80 m margin is unobservable on the engine's own box** — a fact about
   the geometry, not a fixture gap. On 1700 × 1250 the reference point is at
   (884, 525) and candidates reach at most 240 m from it, so none is within
   400 m of the margin; it takes a ~520 m box to make the constant do anything.
8. **The flood-band penalty `Math.max(0, rd - 260)` is dead on every site this
   engine builds.** A candidate is drawn ≤240 m from a reference point that lies
   *on* the water on every watered site, so `rd` cannot exceed 260; a landlocked
   site's river is a dummy segment at `(-1e4, -1e4)`, making the term a ~81-unit
   constant that cannot move the argmax. A test asserts the invariant across all
   fixtures rather than asserting the dead branch.
9. **`buildPrimariesFromPaths`' final `sm.length < 2` guard cannot fire**: `pts`
   has ≥2 entries, `simplify` is the identity below three points and never drops
   an endpoint, and `chaikin` on an open 2-point line returns 4. Reproduced as
   written.
10. **Its `path.length < 2` guard is redundant, but its `pts.length < 2` one is
    not.** A path whose *second* point leaves the box leaves only the market in
    `pts`; without the second guard that survives as a two-identical-point street
    that adds a **node** and no edge. Fixture `pathsOnlyMarket`.
11. **A metre offset added to a metre coordinate cannot express a one-ulp
    boundary.** The host's paths are offsets from the market, and
    `(386.6 + 1.0000000000000002) - 386.6` is exactly `1.0`. `> 1` is straddled
    with 1 m and 1.25 m, the 6 m box tolerance with -5, -6 and -7. **A boundary
    fixture built by offsetting a large coordinate must clear that coordinate's
    ulp, not the constant's.**
12. **`toCell`'s clamp absorbs the `Math.round` question.** JS and `f64::round`
    differ only on negative halves, and a negative cell index clamps to `1`
    regardless. `js_round` is written correctly anyway (`rules`' private copy
    routes through it, provably identical on `[1, 4]`), because the next caller
    may not clamp.
13. **The reinforcement's `Set` iteration order cannot matter**: each cell is
    multiplied by `0.45` once per route and the indices are disjoint. The route
    *sequence* is what is order-dependent: a test reverses `site.routeEnds` and
    requires the town to change, so the `0.45` cannot be quietly neutralised.

#### Golden verification

The functions are exposed with `buildSite` and `makeGraph` by the anchored
`return {` replacement. Everything is compared bit for bit; the spatial index is
pinned by the reference's own `fnv1a` over its canonical grid dump rather than
cell by cell (milestone 2 golden-tested the index itself; restating ~400 cells
per scenario would have added 40,000 lines for no strength).

The capture refuses to write unless: ≥37 scenarios, ≥18 driving `buildPrimaries`
and ≥12 `buildPrimariesFromPaths`; every market finite and every provenance
string non-empty; `nextN`/`nextE` still equal the array lengths; every edge a
7 m-wide epoch-0 `'primary'`; the 80 m margin **rejects >20 and admits >20** on
the two mid-box fixtures and rejects **zero** on the full-size one;
`lastCandidateWins` really wins on candidate 399; `shortDtWater` admits >100
candidates and scores every one NaN; `tinyBox` takes the `best === null`
fallback and `landlocked3` does not; `bay`/`coast` diverge on one seed while
`atoll`/`coast` coincide; `nanCost` produced no routes; `_fromPaths` agrees
with the route count on every paths scenario and is false on at least one; the
1 m unshift boundary is straddled both ways; the box-edge triple keeps 3 of its
4 points; `bendPath`'s Chaikin corners separate `simplify(1.2)` from
`simplify(1.3)`; and the capture carries ≥400 edges and ≥550 route points.

#### Mutation testing

Every numeric literal on a non-comment line of `routes.rs` and of the FDLIBM
block (231), plus 69 structural mutations: every draw and its order, every
comparator and tie-break, both `||` fallback chains, the cost field's three
terms, `toCell`'s clamp order and rounding, the reinforcement factor and its
accumulation, the `astar` endpoint order, both smoothing pipelines, the street
class and width, `_fromPaths`, the in-box break, the market unshift, and every
branch of `kernel_sin`/`kernel_cos`/`rem_pio2`/`js_log`/`js_round`.

**Five sweeps: 300 / 98 survivors, 300 / 79, 306 / 73, 306 / 74, and finally 306
mutations, 233 died, 73 survived**, with zero false survivors in any round. Six
are **graded perturbations** and all six die: the sea cost `240 → 5`, both
second-simplify tolerances `1.2 → 4.0`, `toCell`'s lower clamp `1 → 3`, the
margin `80 → 200` on all four sides, and the flood-band penalty `260 → 20000`.
The round that went **73 → 74** is the "add, do not substitute" rule: swapping a
trig band with a ~1e-9 remainder for one at ~1e-13 gained the third correction
round and lost the kernels' `|x| < 2^-27` shortcut. Both bands are present now.

##### The 19 survivors in `routes.rs`

| class | n | why it survives |
|---|---|---|
| the 80 m margin, three of its four sides | 3 | `marginWinner` is a scanned site whose *winning* candidate sits 80-110 m from **one** edge, so only that side is observable; each other side needs its own scanned site. The graded `80 → 200` **dies** |
| the flood-band penalty's `0` and `260` | 2 | proven dead (finding 8), asserted across every fixture; the graded `260 → 20000` **dies** |
| the `240` sea cost | 1 | a **barrier, not a cost**: any value large enough to make water non-optimal gives the same path, so `240 → 328.93` cannot move one. The graded `240 → 5` **dies** |
| `toCell`'s two `1.0` lower clamps | 2 | the result is immediately `as usize`, so a sub-cell change truncates away — the quantised-output pattern. The graded `1 → 3` **dies** |
| five comparators that need an exact tie | 5 | the margin's `<` → `<=`, the flood band's `<` → `<=`, the score's `>` → `>=`, the bridge window's `<` → `<=`, the bank band's `<` → `<=` — continuous distances and sums of RNG draws, not closable by a quantised raster |
| `bs = −∞ → −1e308` | 1 | no reachable score is below `−1e308`; the initial value only has to lose to the first accepted candidate |
| `toCell`'s clamp **order** | 1 | `max(1, min(W−2, ·))` and `min(W−2, max(1, ·))` differ only when `W < 3`, a box under 24 m |
| `js_round → f64::round` | 1 | differ only on negative halves, which clamp to `1` either way |
| `fromPaths`' `path.len() < 2` → `is_empty()` | 1 | a one-point path yields a one-point in-box run, which the next guard drops (finding 10) |
| `rem_pio2`'s round triggers, `16 → 17` and `49 → 50` | 2 | both rounds are load-bearing and tested; no fixture produces an exponent gap of *exactly* 17 or 50 |

##### The 54 survivors in the FDLIBM block

That block was `geom.rs`'s when this sweep ran; it has since moved to
`cartalith-jsmath` (`libm.rs`) with its goldens.

| class | n | why they survive |
|---|---|---|
| dead in **this port's** call path | 11 | `js_sin`/`js_cos` filter `\|x\| ≤ π/4` and Inf/NaN before calling `rem_pio2`, so its own early return and Inf/NaN branch are unreachable through the public API (8); `HUGE_ARG_HI` only decides where the platform hand-off starts (1); the `ix == 0x3ff921fb` sub-branch needs `\|x\|` inside a 2.3e-8-wide window at π/2 (2) |
| `iy` is a flag, not a value | 5 | `kernel_sin`'s third argument is only tested `== 0`, so `1 → 2` is the same call at all four sites; on the `\|x\| ≤ π/4` path `y` is unused, so `0.0 → 1.0` is too |
| ±1-ulp threshold constants | 18 | every `0x…` bound — the four `0x7fff_ffff` masks, `0x3e40_0000`, `0x3fd3_3333`, `0x3fe9_0000`, `0x3fe9_21fb`, `0x4002_d97c`, `0x4139_21fb`, `0x7ff0_0000`, `0x0010_0000`, `0x6147a`, `0x6b851`. One ulp of a **high word** changes behaviour only for an argument in that window; 54,000 uniform draws never land in one |
| provably equivalent arithmetic | 13 | `0x95f64` is **even**, so the bit its mask can add to `i0` is one `hx` already carries (checked by hand — it looks catastrophic); `qx`'s `0x0020_0000` cancels in `a − (hz − …)`, which is why FDLIBM may pick `0.28125` arbitrarily; `(x as i32) == 0 → == 1` skips the tiny-x shortcut and the polynomial returns `x` (or `1.0`) anyway; `hx > 0 → > 1` and `hx < 0 → < 1` sit where `hx ∈ {0, 1}` is unreachable; and `js_log`'s five branch selectors pick between two algebraically identical final formulas |
| the staged reduction refines the **tail**, not the returned double | 7 | the four `y[0] → y[1]` index mutations in the medium branch, both `0x7ff` exponent masks, and one more trigger form. **Evidence**: never running the second round (`i > 100000`) **dies**, always running the third (`i > −1`) **dies**, always running the second (`i > −1`) **survives** — FDLIBM's first round is already "good to 85 bit" against a 53-bit result, so one round more is free and one fewer is not |

#### Corrections to later milestones

1. **Port every libm function against a bulk hash golden, not a dozen rows** —
   the first sweep here left 63 survivors inside `js_sin`/`js_cos`/`js_log`
   (see "Verification convention"). Note `JS_SEMANTICS_AUDIT.md`'s measurement of
   `js_atan2`'s trap: the `m &= 1` correction V8 carries and the 1993 fdlibm
   source does not.
2. **`Graph::from_paths` exists for milestone 10**, whose `builtMassHull` skips
   `g._fromPaths && alive.length < 3 && every cls === 'primary'`.
3. **Milestone 16 inherits no draws from the route builders** — only the graph
   and `placeAnchors`' 800-draw `'anchors'` substream.
4. **The adapter's offsets are the ulp trap** (finding 11): `_umPrimaryPaths` and
   `_umRouteEnds` produce host metre offsets, so a boundary fixture on them must
   clear the market coordinate's ulp — ~1e-13 m at this box size.
5. **The market can land outside the box** (finding 6). Milestones 7 and 10
   measure from `anchors.market`, which is not guaranteed to be inside the town.
6. **`extractFaces`' half-edge sort key is `js_atan2`** (`graph::extract_faces`),
   not the `f64::atan2` milestone 2 first wrote before `Math.atan2` was
   measured. A one-ulp angle change bites only when two half-edges at one node
   point within an ulp of each other — the argument once made for `hypot` before
   milestone 2 proved it changes topology.

### Milestone 7 — organic growth

Reference lines 29384-29630 (`logisticRamp` from 29390; 29384-29389 is its doc
comment, which flags `k = 6.5` as tuned rather than measured).
`logisticRamp`, `estimateCarryingCapacity`, `wallOccupancy`, `grow`,
`supersedeWall`. Module `growth`. Its golden is a **per-epoch** graph hash, not
a single end-state hash, as the plan asked; it carries the total street length
placed, a per-epoch trace of the whole graph, every node and edge, a hash of
every provenance string, the spatial index, every `buildWall` call and every
supersession record.

#### What this milestone ported that belongs to later ones

`grow` calls `buildWall` (line 29748, milestone 10), `ringCrossings` (line
29631, milestone 10) and `distToLine` (line 28971, milestone 9). The last two
(six and three lines) are here as `ring_crossings` and `dist_to_line`, and
milestones 9 and 10 read them from `growth`.

`buildWall` arrives as a **`WallBuilder` trait object**, which is what made the
rest testable: the capture stubs the reference's own `buildWall` by one anchored
insertion, so both sides run the same no-op recorder and every branch that
*leads* to a wall — the fire epoch, the M-GRW-2b age gate, the M-GRW-2a
occupancy gate, the generation cap, the supersession — is golden-verified.
**What the stub changes:** it never writes `wallState.ring` or advances
`wallState.epoch`, so (a) a run starting at `ring: null` cannot reach
supersession, which is why the supersession fixtures **preset** a ring; and (b)
the age gate is measured from the initial epoch for every generation instead of
being re-armed by each circuit, which is why `genSupersede` supersedes twice in
successive epochs. Identical on both sides, so parity-neutral.

`WallState` holds all nine fields `buildWall` writes (`waterWalls`, `spurs`,
`spansWater`, `style`, `prov`, `fort`, `centroid`, `terrainDeflected`,
`_waterClosure`) and `WallGeneration` the six `supersedeWall` copies into its
history record (correction 5).

#### Findings

1. **`kept` is dead.** `grow` pushes `made[0].id` into a local array never read,
   returned or exported. Omitted.
2. **The wet-crossing walk takes six samples, and the last is the segment's own
   endpoint.** `for (let t = 0.15; t <= 1; t += 0.17)` gives `0.15`, `0.32`,
   `0.49`, `0.66`, `0.8300000000000001` and exactly `1.0` — read out of `node`.
   The reasoned answer (drift to `1.0000000000000002`, so five samples) was
   **wrong twice over**; expectations come from running `node`, not from
   reasoning about decimals.
3. **The accumulation is not load-bearing at these three constants**: `0.15 +
   k * 0.17` for `k` in `0..6` is bit-identical to the accumulated walk, and
   `1.17` either way past the end — measured, so a milestone that changes the
   step knows to re-measure.
4. **A NaN slope does not reject.** `NaN > 0.34` is false, so an all-NaN
   heightfield stops nothing in legalisation. It poisons
   `estimateCarryingCapacity` instead: the ring average is NaN, `clamp` returns
   NaN, `maxR` is NaN for the whole run, and every `dM > maxR` is false — the
   **reach limit disappears** rather than growth stopping. Fixtures
   `nanSlopeTown`, `genCcNanTerrain`.
5. **`opts.rules || DEFAULT_RULES` is the raw table, confirmed by golden**: the
   capture asserts a run with no `opts.rules` equals one passing an explicit copy
   of `DEFAULT_RULES`, and the Rust shape gate re-asserts it on the two graph
   hashes.
6. **`primEdges` is captured once per epoch, before any street is placed**, so
   streets laid this epoch cannot anchor this epoch's ribbon suburbs. A real
   ordering decision that a "hoist the filter" refactor would silently invert.
7. **`wallState.generation || 1` reads a stored `0` as `1`** — `genGenerationZero`
   supersedes like a first circuit, while a preset `3` hits
   `maxWallGenerations` and blocks.
8. **`Math.max(3, Math.floor(epochs * 0.6))` needs three fixtures**: at 2 epochs
   the wall never fires (floor 3, run ends at 2); at 3 and at 5 it fires at
   epoch 3 (the `max` arm and the `floor` arm); at 8 it fires at 4.
9. **An empty path list is the empty-graph fixture.** `grow` on no nodes runs
   `g.nodes[r.int(0, -1)]` — `undefined` in JS, `None` here — spends its 2,600
   tries per epoch and places nothing, with the same RNG budget on both sides.
10. **A harbour with a one-point quay is still a harbour.** The reference tests
    the object for truthiness and indexes `.quay`; `distToLine` over fewer than
    two points is `Infinity`, so `Math.min(dM, Infinity + 35)` is `dM`.
    `harbourEmptyQuay`'s graph hash equals `coastTown`'s, asserted.
11. **`estimateCarryingCapacity` is a declared placeholder, ported as one.** Its
    header pins the contract — one number in ~`[0.3, 1.0]`, never a hard zero,
    every consumer treating it as "whatever this returns". Replacing it is a
    Cartalith decision, not a porting one.
12. **The carrying-capacity ring is not clipped to the site box**, and
    `anchors.market` need not be inside it either. On a raster-backed site a
    probe outside the box can return NaN — finding 4's path.

#### Golden verification

Three anchored text edits, each asserted to match **exactly once**: the
`return {` replacement exposing the five functions and the builders the fixtures
need; the `buildWall` stub; and a per-epoch observer inside `grow`'s loop.
`graph_hash` is the reference's own `fnv1a` over a canonical dump of every node
and edge with each double as its exact 64 bits — a bit-for-bit statement, not a
tolerance. The explicit node/edge dump is kept only for scenarios under 170
edges, so a failure is readable (785 KB → 244 KB). `prov_hash` pins the
Exploration/Densification split, the epoch stamp and the ring-road string's
interpolated `Math.round(fillFraction * 100)`.

The capture refuses to write unless: ≥40 scenarios, ≥30 grew a street, ≥3,000
edges in total; ≥8 called `buildWall` and ≥3 superseded; at least one laid a
ring road and `genSupersedeNoArc`/`genSupersedeShortArc` laid none;
`genAgeGapBlocks`/`genCapBlocks`/`genOccupancyBlocks` blocked and
`genAgeGapDelays` superseded exactly once; `genGenerationZero` read `0` as `1`;
the four fire-epoch fixtures fired at exactly `[]`/`[3]`/`[3]`/`[4]` and the
preset-ring one not at all; `emptyGraph` stayed empty; `seedShortOnly`'s first
grown street was an exploration one; `nanSlopeTown` grew; `genCcNanTerrain`
produced a NaN capacity; the two harbour fixtures and the two ring fixtures
diverge; the four rules variants produce four distinct towns; the raw-table
fallback equals the explicit one; and every trace has one record per epoch. The
Rust side mirrors all of it.

#### Two rounds of fixtures lost to one lesson

**The terrain rasters were in metres.** `site.height` reads `opts.terrain.grid`
raw and `site.slope` multiplies a per-metre central difference by **900**, so
40-95 m of elevation gives slopes of 2 to 204 and `slope > 0.34` rejected every
candidate on every raster-backed site: fifteen fixtures grew nothing. A
realistic normalised grid varies by ~0.1 across the box; `TERRAIN_RIDGE` exists
so the 0.34 rejection does fire.

**A hand-drawn ring can never be 80% full.** The M-GRW-2a gate needs
`fillFraction >= 0.8` **and** `exteriorCount >= max(10, interior * 0.15)`.
Ellipses on the market topped out at 0.44 (0.58 when scaled); the hull of the
**whole** built mass at epoch 3 reaches the box edges along the primaries, so
inflating it enclosed everything and left `exteriorCount` at zero. What works is
the epoch-3 hull **restricted to 260 m of the market**, inflated 6% — roughly
what `buildWall` itself constructs. Sweeping the radius shows the gate opening
between 180 m and 220 m and staying open.

#### Round 2: twelve fixtures, and seven survivors turned into assertions

The first sweep left 51 survivors. Twelve new scenarios closed the closable
ones, and every one matched the reference on its first run:

| fixture | the constant it exists for |
|---|---|
| `seedExact38` | a closed square of four **exactly-38 m** edges with no degree-1 node, so neither the mid-edge tap (`dist < 38`) nor the dead-end continuation can fire; `<` and `<=` are different towns |
| `smallBox`, `smallBoxRiver` | 520 × 420 and 560 × 460 boxes, where the `40 m` box margins actually reject |
| `harbourClose` | a quay 40 m off the market, so `distToLine(quay) + 35` is the smaller term |
| `genAgeGapExact` | 160 years over 8 epochs is 20 a year and `120 / 20` is **exactly 6.0**, so `>=` and `>` differ by one epoch — the only integer-vs-integer boundary in the function |
| `genNoAgeRing` | `settlementAge` absent, rule gap `262.5`, so `262.5 / (300/8)` is **exactly 7.0** — making the `300` default observable |
| `genZeroAgeRing` | `settlementAge: 0` is falsy, so it must equal the absent town; without it dropping `js_truthy_num` is invisible |
| `genTinyAgeRing` | `settlementAge: 0.5` with a 1-year gap: the only setting where `Math.max(1, …)`'s floor decides inside 8 epochs |
| `genExtramuralHigh` | `share = 0.8`, so `interior · share` exceeds the exterior count — proving it multiplies the **interior** count |
| `genExtramuralFloor` | **scanned**: `share = 0` pins `max(10, …)` to its floor, and the ring radius (592 m) was searched for the first supersession at an exterior count of **exactly 10** |
| `genRingReversed` | the same circuit wound the other way — what `Math.abs(polyArea(ring))` is for |
| `genSupersedeTwoArc` | a two-point `landArc`, between the one-point arc that lays no road and the long one that does |

Seven were dealt with the other way. A proof does not *kill* a mutant, so these
still count as survivors, but each now rests on an **executable** statement:

- `estimateCarryingCapacity`'s clamp bounds are dead by construction —
  `terrainSuitability` is a product of two `[0, 1]` factors, so
  `0.3 + 0.7·mean` is already in `[0.3, 1.0]`. Asserted over 720 probes.
- `wallOccupancy`'s `alive` filter cannot bite here: `rawEdge` is the only
  writer of `adj` and `splitEdge` removes the id when it kills an edge. Asserted
  over all 60 scenarios; milestone 11's `_killEdge` makes the filter live.
- The junction-angle double wrap, `abs(((a−b) % π + π) % π)`, is undone by the
  `min(dd, π − dd)` after it at both call sites — measured over 200,000
  arguments.
- The twelve ring angles are `2π·i/12`, and V8's FDLIBM and the platform agree
  on **all twelve**, so `js_cos → f64::cos` survives *here*. The test asserts
  both halves — agreement on the twelve and >100 disagreements in 40,000
  arbitrary angles — so it cannot be read as a licence elsewhere.
- A zero-area ring cannot contain a node, so `wallArea > 0` sits beside an
  `interior.length >= 8` that can never hold with it.
- `convexHull`'s winding never varies, so `abs` on the hull area is a no-op —
  the `abs` on the **ring** is not (`genRingReversed`).
- `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0` are assigned only when
  `wallGenerations` is off and read only when it is on — asserted from the other
  side: with it off, neither the capacity weight nor the age can move the town.

#### Mutation testing

Every numeric literal on a non-comment, non-string line of `growth.rs` (96) plus
118 structural mutations: every draw and its order, every comparator and
tie-break, both `||` fallbacks, the epoch loop's origin branches, the reach and
bank tests, the ribbon-suburb rule, the demand gradient, every legalisation
guard, the wet walk, the wall-permeability and parallel-spacing loops, the
street class and width, the provenance strings, all four arms of the wall
episode, every field of the supersession record, and both borrowed helpers.

**Two sweeps: 214 / 51 survivors, then — after the twelve fixtures and seven
assertions — 214 mutations, 176 died, 38 survived**, with no false survivors.
**Eleven are graded perturbations and all die**: `k` `6.5 → 30`, the mid-edge
minimum `38 → 300`, the junction minimum `18 → 400`, the slope limit
`0.34 → 0.001`, the gate radius `20 → 4000`, the tapped-frontage skip
`1.5 → 500`, the parallel-angle limit `0.5 → 3.2`, the exploration band
`+140 → +5`, the ribbon radius `90 → 2`, the interior-node floor `8 → 400`, and
the try budget `2600 → 12`.

##### The 38 survivors, by the invariant each rests on

| class | n | why they survive |
|---|---|---|
| **an exact tie on a continuous value** | 13 | `len < budget`, the bridgehead distance and probability, the 90 m ribbon radius, `h.u > 1e-3`, `h.t > 0.03`, `h.t < hitT`, the 18 m junction minimum, the junction-angle limit, the 0.34 slope limit, the 20 m gate radius, the parallel spacing, and `fillFraction >= 0.8`. Every input is a polyline distance, an angle, a hull-area ratio or a raw `mulberry32` draw, none of which a quantised raster can pin. Where the boundary *was* integer arithmetic — the age gate, the extramural floor, the 38 m minimum — round 2 built the fixture and the mutant died |
| **proved dead or a no-op, with an executable assertion** | 11 | both carrying-capacity clamp bounds; `wallArea > 0` twice; `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0`; the probe-ring rotation `i → i+1` (twelve evenly spaced angles are the same twelve points); `js_cos → f64::cos` on these twelve angles; the `alive` filter on `adj`; `abs` on the hull area; and the junction-angle double wrap |
| **an exact integer count no town produced** | 4 | `interior.len() >= 8` both ways and `hull.len() >= 3` both ways. Closable in principle — it needs a circuit containing exactly eight built interior nodes, or an interior hull of exactly three vertices, while passing the fill and extramural gates — but these counts are outputs of the growth loop, not inputs, and cannot be constructed by hand |
| **a bound no reachable value approaches** | 5 | `tries < 2600` → 2601; `h.u < 1 − 1e-3` widened twice (`segInt` returns `u ∈ [0, 1]`); the wet walk's start `0.15 → 0.3155` (no fixture is wet *only* in that opening slice); and `fmt_js_int`'s `n > 0` sign test, which needs an infinite `fillFraction` |
| **three of the four 40 m box margins** | 2 | the small-box fixtures bound growth against one edge and killed that side; each other side needs a site whose growth is bounded by that edge. Milestone 6's margin finding exactly — a margin is invisible until the candidates it removes were going to be kept |
| **provably equivalent rewrites** | 3 | the tapped-frontage skip `1.5 → 2.165` (the frontage sits at ~0 and every other edge is far past both); `edgesNear(midp, midp) → edgesNear(O, B)` (a superset of cells, but `d < 24` measured from `midp` rejects every extra one); and `arc.length > 1 → > 0` (a one-point polyline yields no consecutive pair) |

#### Corrections to later milestones

1. **This milestone's range is 29384-29630**: the six-line `logisticRamp` doc
   comment belongs to it.
2. **Milestone 14 ends at 29382**, not 29389, which ran seven lines into
   `logisticRamp`'s doc comment.
3. **`distToLine` is `growth::dist_to_line`**; milestone 9 does not port it
   again, and its range opens on the harbour header at 28967.
4. **`ringCrossings` is `growth::ring_crossings`**; milestone 10 does not port it
   again. 29638 is the `wall + gates` header, so milestone 10's range holds two
   sections.
5. **`WallState` and `WallGeneration` extend together.** Adding a field to
   `WallState` without adding it to `WallGeneration`'s copy produces a silently
   lossy history that every structural test still passes. The six are copied by
   the same statement that copies the other four.
6. **These goldens run against a stubbed `buildWall`.** With the real builder
   the fire-epoch fixtures produce a ring, and `genSupersede`'s two-in-two-epochs
   supersession becomes one, because the real builder sets
   `wallState.epoch = ep` and re-arms the age gate. The real builder is
   golden-verified by milestone 10 and in the whole town by milestone 16.
7. **From `generate()`, `grow` always enters with `ring: null` and a resolved
   rule set.** `generate()`'s only pre-`grow` `buildWall` (line 31017) is in the
   **radial** branch, which does not call `grow` (lines 31011-31028 are an
   `if/else`). So the `ep === fireEpoch` arm is always live in production, the
   preset-ring fixtures are a superset of what `generate()` reaches, and the
   raw-table fallback is reachable only by a direct call.
8. **`grow`'s `opts` is `generate()`'s literal at line 31027.** `wallStyle` and
   `fortified` are read only by `buildWall`; `pop` is read by nothing in the
   subsystem.
9. **A raster-backed fixture must use a normalised heightfield** — this hits
   `buildWall`'s terrain deflection (10), `terrainAware` parcels (13) and
   `computeMetrics` (15).

### Milestone 8 — radial (Venus) streets and waterway

Reference lines 28835-28939 (`buildRadialStreets` from 28844, `buildWaterway`
from 28928). Module `radial`. `buildPlaza`, which sits between them in the
reference, is milestone 8a.

The second planning mode: `generate()` forks on `profile.planning` (line
31011); `'radial'` calls `buildRadialStreets` once — concentric rings, twelve
spokes off a central hub, twelve cross-spokes in the outer band — and **never
calls `grow`**; the ordinary face detector then turns ring × spoke crossings
into annular-wedge blocks. `buildWaterway` is a closed decorative canal outside
the outermost built ring, where nothing is built. Every ring radius is modulated
by two summed sines (amplitude 5.5%, too small for consecutive rings to cross)
and every spoke angle jittered ±0.045 rad — the reference's own post-review
softening. `stream(seed, 'radial-organic')` takes exactly **28** draws, none
conditional: the per-spoke draw is taken **before** `landSeg` decides whether
that spoke is laid. This port gives radial towns the wall lots and faubourg
under Ruling AA (see "Beyond the reference").

### Milestone 8a — the plaza

Reference lines 28941-28965. `buildPlaza` alone. Module `plaza`. Split out of
milestone 8 because `buildPlaza` runs on **both** branches of `generate()`
(lines 31018 and 31024) while the rest of milestone 8 serves the radial branch.
It is the difference between a town with an open market square and a town with
a block platted over its own anchor.

#### Where it runs is part of the port

On the organic branch `generate()` calls it **between the primaries and
`grow`**, so the town accretes around the square; on the radial branch it is the
branch's last call, **after** `buildWall` (line 31018). Putting it after `grow`
still produces a plaza and produces a different town.

#### Nothing new was built for it

`distPtSeg`, `V.norm`/`lerp`/`rot90`, `polyCentroid` and `addStreet` are
milestones 1-2; `stream`/`range` milestone 1; `site.riverDist` milestone 5.
`stream(seed, 'plaza')` is its own substream taking exactly two draws, so this
stage **cannot** perturb any other milestone's sequence.

#### The mutation sweep, and why its survivors were closable

20 mutations, 20 killed — the first sweep in this subsystem to close with zero
survivors. Five survived the first pass, all of milestone 7's *"exact tie on a
continuous value"* class:

| survivor | closed by |
|---|---|
| side probe `20 → 21` | a river centreline laid **parallel to the street under test** |
| side probe `-20 → -21` | the same fixture at a 0.25 m offset |
| `>` → `>=` in the side ternary | the same fixture at an exact tie |
| `rot90()` → `-rot90()` | the same exact tie — see below |
| `d < bd` → `d <= bd` | two primaries exactly equidistant from the market |

These rested on **distance to a centreline**, and `site.river` is a plain field a
fixture may overwrite on a real `build_site` site — so the gap becomes an
*input*. Parallel makes it a razor: along the edge normal the distance to a
parallel line changes metre for metre, so `c = 0` is an exact tie and
`c = 0.25` a 0.5 m gap, inside the window a one-metre mutation moves the answer
through. **A survivor resting on a continuous comparison is closable exactly
when one side of the comparison is a field the fixture can set, rather than an
output of an earlier stage.**

**Negating the edge normal is not the no-op it looks like.** `nl` is read twice —
for the two probe points and in `nl * (side * wd)` — and away from a tie the
two negations cancel bit-exactly, which is why it survived all 15 real towns. At
an exact tie both ternary arms give the same `side`, so the product flips and
the square opens the other way. Without the tie fixture it would have been
recorded as proved dead.

#### Findings

1. **`buildPlaza` mutates `g` before its return value exists, and the two
   disagree.** The three streets go in through `addStreet`, whose 11 m snap binds
   a plaza corner to an existing node — up to **6.1 m** of movement across the
   fixtures — while `plaza.poly` and `plaza.center` are built from the
   **pre-snap** points. That is why `buildBlocks` tests a *point* against each
   face, and why a consumer must not assume the returned quad is the face the
   graph holds.
2. **The fourth side is not laid.** Three `addStreet` calls: `p1 → p2` is the
   primary being widened. Laying four gives the same picture and a different
   graph.
3. **"Away from the river" is a statement about 20 m, not about the square.**
   The probe sits 20 m either side of the street's midpoint and the square is up
   to 40 m wide, so on a curving channel the far edge can end *nearer* the water
   than the rejected side's would have — 0.05 m on `river7`. Captured and
   asserted, so it is not read as a port bug.
4. **A landlocked site still resolves the ternary**: `riverDist` answers from
   the dummy centreline. Three landlocked scenarios are in the golden for it.
5. **Every scenario produced exactly one flagged block** — asserted as a
   property: a change upstream that split the widened band into two faces shows
   up here first.

#### Corrections to later milestones

1. **`Plaza` is `blocks`' input and `plaza`'s output**: defined in `plaza.rs`,
   re-exported from `blocks.rs`; `build_blocks` takes `Option<&Plaza>`.
2. **Milestone 12's goldens change with the input graph.** This milestone changed
   `blocks.rs`'s input; `lanePass` and `removeWaterCrossings` (milestone 11)
   change it again, so milestone 12's sweep is only as current as the graph it
   last ran on.

The source engine later split the plaza into a market square and a village green
(`RC_ENGINE_CHANGES.md` §8.2, v2.73).

### Milestone 9 — water infrastructure

Reference lines 28967-29154 (`distToLine` from 28971). `distToLine` (already
`growth::dist_to_line`, milestone 7), `buildHarbour`, `addRiverBridges`,
`detectRiverCrossings`. Module `water`. Quays, back streets, herringbone stubs,
the harbour road, piers, the breakwater mole and three harbour-defence
repertoires; bridges, fords, and the navigability guards that invalidate a
harbour on a stream too small to carry one.

- **`detectRiverCrossings` must run after every pass that can kill an edge** —
  `removeWaterCrossings` (line 31030), `privatizeAlleys` (31069) and
  `clearFortZone` (31072) — so a recorded bridge always has a live road on it
  (the reference's own comment at 31073-31075). This constrains runtime order
  (milestone 16's), not porting order.
- **`addRiverBridges` returns immediately on `site.usesRealWater`**, so the two
  crossing functions are never both active on one town.
- **`site.kind === 'coast'` is compared as a string twice** (lines 29061,
  29081) — whether a mole is built, and `'molefort'` as the `auto` defence.
  `rk` (`'river'`/`'riverthrough'`) is a different string test, not
  `Site::river_like`.
- `buildHarbour` writes `site.harbourInvalid` and `detectRiverCrossings`
  `site.bridges`/`site.ford`; nothing in block 4 reads them, so the port returns
  them as values (`HarbourOutcome`, `Crossings`) and `Site` stays immutable.

`water.rs`'s header records the `'harbour'` substream's draw order, including its
two conditional draws.

### Milestone 10 — fortification

Reference lines 29631-30032 (the `wall + gates` header at 29638). Nine
functions: `ringCrossings` (already `growth::ring_crossings`), `convexHull`
(already `geom::convex_hull`, milestone 1), `densifyLoop`, `nearestIdx`,
`cornerCut`, `townBank`, `builtMassHull`, `buildWall`, `applyStarFort`. Module
`fortify`. The largest single milestone: curtain tracing round the built-mass
hull, gates where primary routes cross it, wet/dry ditches, and the bastioned
trace.

- `builtMassHull` reads `Graph::from_paths` (milestone 2, finding 2).
- The `'ringroad'` street class arrived a milestone early, from
  `supersedeWall` (milestone 7).
- `cornerCut` is the subsystem's only `Math.acos` call site and feeds a
  threshold, so it uses `js_acos`.
- `WallState`/`WallGeneration`: milestone 7, correction 5.

`fortify/tests.rs`'s header records the fixtures and what each exists for.

#### Mutation testing: 225 mutations, 188 dead, 19 equivalent, 18 open (2026-09-24)

The first sweep of this module (`OUTSTANDING_WORK.md` §2.11). The mutants cover
every numeric literal on a code line of `fortify.rs`, every comparator on a
tuning path, and every boolean gate: the `usesRealWater`, `usesRealTerrain`,
`_fromPaths`, `noWater`, `fortified` and `wetMoat` guards, the style ternary, both
prov ternaries, and the builder's three pass-throughs. Each constant was moved
one step, and where a step could hide under rounding it was moved both ways.
Provenance prose was not mutated; the goldens hash it. The mutants ran one at a
time in a scratch mirror of the crate with its own target directory. Each
anchor was asserted to match exactly once, each file was restored in `finally`
and hash-checked, and every survivor was re-run in isolation after a clean
rebuild: **91 of 91 survived again, so there were no false survivors**. There
were no build errors.

**134 died to the existing suite; 54 more died to fixtures written for them.**
The new tests are in `fortify/tests.rs` under *mutation-sweep fixtures*. Each is
built from the geometry under test: hand-laid junction rings, a channel or
shoreline overwritten on the site, the market moved, and every tie made exact
and asserted exact before use.

| new test | kills |
|---|---|
| `nearest_idx_starts_from_infinity_not_from_any_finite_bound` | `bd = Infinity` → 1000 |
| `corner_cut_clamps_the_cosine_to_exactly_minus_one_and_one` | both clamp bounds (a 0.1 rad needle at `minAng` 0.3; a 3.06 rad vertex at 3.0) |
| `corner_cut_keeps_a_vertex_exactly_at_the_threshold` | `angI < minAng` → `<=` (a square at `minAng = js_acos(0)`) |
| `town_bank_keeps_its_normal_when_the_market_lies_on_the_tangent` | both `nl·(market−p) < 0` → `<=` (channel and real coast) |
| `from_paths_discounts_only_all_primary_vertices_under_degree_three` | `alive.len() < 3` → 4; `.all` → `.any` |
| `built_mass_hull_keeps_a_junction_exactly_at_the_channel_margin` | `< riverW/2 + 14` → `<=` |
| `a_through_site_still_needs_eight_near_bank_junctions` | `near.len() < 8` → 7 (seven near and eight far on a riverthrough site) |
| `the_bridge_town_rule_is_strictly_more_than_max_twenty_and_thirty_two_percent` | `>` → `>=`, 20 → 21, 0.32 → 0.33 (exact integer counts on each bank) |
| `the_percentile_cut_keeps_a_junction_exactly_on_it` | `<= cut` → `<` (market at the origin, so the distance is exact) |
| `the_aspect_cap_is_for_real_water_only_and_compresses_the_long_axis` | the `usesRealWater` gate; the `ul < 1e-6` fallback axis |
| `the_aspect_cap_applies_to_a_three_vertex_hull` | `hull.len() >= 3` → `> 3` |
| `terrain_deflection_needs_real_terrain_not_merely_a_relief_value` | the `usesRealTerrain` gate |
| `terrain_deflection_engages_at_exactly_the_relief_floor_and_on_a_triangle` | `relief >= 0.01` → `>`; `hull.len() >= 3` → `> 3` |
| `terrain_deflection_prices_distance_at_three_point_three_e_minus_four` | `3.3e-4` → `3.4e-4` (relief set so the threshold falls between two hull vertices) |
| `terrain_deflection_can_choose_the_thirty_metre_inward_step` | the −30 offset (a 1 m-cell crest 30 m inside every vertex) |
| `terrain_deflection_keeps_twenty_metres_off_the_near_box_edges` / `…_far_box_edges` | all four 20 m box margins (candidate at 20.5 m) |
| `a_hull_vertex_exactly_at_the_channel_margin_is_water` | `isLand`'s `> riverW/2 + 1` → `>=` |
| `two_river_crossings_merge_into_one_water_gate_only_inside_40_metres` | the spanning circuit's water-gate dedupe 40 → 41 (a square notch in the centreline) |
| `the_harbour_mouth_is_a_strict_48_metre_gap_and_needs_a_quay` | `GAP_R` both ways, `<` → `<=`, the `quay.length` gate (`riverTown`'s own bank, mouth at 47.5/48/48.5 m) |
| `an_empty_wall_style_is_the_legacy_curtain` | the `s.is_empty()` arm |
| `a_shoreline_gate_is_a_water_gate_within_24_metres_of_the_straddling_vertex` | 24 both ways, `<` → `<=`, both straddle comparators, `i == 0`, the first-vertex seed |
| `a_channel_gate_is_a_water_gate_strictly_within_half_width_plus_22` | 22 → 23, `<` → `<=` |
| `two_land_gates_merge_strictly_inside_40_metres` | 40 → 41, `<` → `<=` (two primaries across a flat wall side) |
| `the_bastioned_cap_spreads_its_gates_at_least_0_9_rad_apart` | 0.9 both ways, `2π − da` → `π − da`, the clash gate |
| `a_star_fort_applies_to_a_triangular_circuit` | `base.len() < 3` → 4 |
| `a_ditch_floods_strictly_within_175_metres_of_the_waterline` | 175 in `wet` both ways and `<` → `<=`, 175 in `canalFed`, the `noWater` gate |
| `the_builder_passes_wall_style_and_wet_moat_through` | `FortificationBuilder`'s `wall_style` and `wet_moat` pass-throughs |

**Equivalent (19):**

| mutant | why it cannot change an output |
|---|---|
| `cornerCut`'s `if (!cut) break` removed | a pass that cuts nothing returns its input, so every later pass is the same pass |
| 6 × `minAng`/`passes` at the three closed double-Chaikin call sites (spanning, all-land, needle fallback) | two Chaikin passes leave every angle over 1.75 rad; `the_closed_ring_corner_cut_is_a_no_op_after_two_chaikin_passes` asserts it over every golden hull |
| `js_max(0, tr²/4 − det)` → `js_max(1, …)` | the discriminant is ≥ 0; below 1 it gives `l1 − l2 ≤ 2`, so with `l2 > 1` the aspect is ≤ √3, under the 2.4 cap either way |
| `l2 > 1` → `> 2` and → `>=` | `l2` sums squared across-axis spreads of a hull inflated 16 m outward. It is hundreds at least, and never near 1 or 2 |
| `aspect > CAP` → `>=` | at aspect exactly 2.4, `k = 1` and the map is a rotation there and back: the identity up to an ulp |
| the all-water `return` removed | the empty land run gives an empty `land_arc`, and the `let … else { return }` guard below refuses the same way |
| `je <= js` → `<` | at `je == js` both arms produce `[bank[je]]` |
| `min(3, …)` → `min(4, …)` | `nSeg ≤ 9`, so `round(n/3) ≤ 3`; already asserted in `the_golden_file_is_the_shape_it_claims_to_be` |
| `da > π` → `>=` | at `da = π`, `2π − π = π` |
| `pc.length < 3` → 4 | `acc + d` on the last side is summed in `arc`'s own order, so it equals `arc` bit for bit; every target is under `arc`, so `pc` always has `nSeg ≥ 4` points |
| `ab.len() \|\| 1` → `\|\| 2` | resampled corners are ≥ `arc/9` apart, so `ab` is never zero-length |
| `nrm·(a − c) < 0` → `<=` | zero would put the hull centroid on a side's own line, which a convex polygon of positive area cannot do |
| `o > maxBulge` → `>=` | on a tie it assigns the value already held |

**Open (18)** — none is proved equivalent, and none has a fixture:

| mutant(s) | why no fixture reaches it yet |
|---|---|
| `net > bestNet` → `>=` | needs a height difference minus a price to equal `minGain` to the bit; a ramp's bilinear samples do not give that |
| spanning water-gate dedupe `< 40` → `<=` | needs two crossings exactly 40.0 m apart; `seg_int`'s crossing point carries rounding (the 41 m mutant is killed) |
| `count > bestLen` → `>=` | two land runs of exactly equal sample count on one densified hull |
| the needle guard off synthetic water, `water.length > 1` → 2, 1.6, 500, `wl >` → `>=` (5) | no synthetic town's bank walk approaches `max(1.6·ll, 500)`; the real-water goldens that trip it (`straitTown`, `islandTown`) exceed it by far |
| `ring.length < 6` both ways | a finished ring of exactly 5 or 6 points is an output count no constructed town produces |
| the land/water-gate dedupe 40 → 41 and `<` → `<=` | needs a primary crossing within 40-41 m of a spanning water-gate |
| `da < 0.9` → `<=` | two land gates exactly 0.9 rad apart as seen from the circuit's centroid |
| the four bulge-window bounds (−0.05, 1.05, both ways) | needs a ring point just past a chord's end that out-bulges every in-window point |
| `maxBulge` starts at −1 | believed equivalent: some ring point always lies on or outside a chord, so the maximum is never negative. A 230 m square, whose resample falls exactly on its corners, still reads 0. Not proved |

### Milestone 11 — graph cleanup passes

Reference lines 30034-30190 (the `clearFortZone` header comment; `_killEdge`
from 30038). `_killEdge`, `pruneLargest`, `removeWaterCrossings`,
`privatizeAlleys`, `clearFortZone`, `lanePass`. Module `cleanup`.

- **The ordering is load-bearing and is the reference's.**
  `removeWaterCrossings` and `clearFortZone` each end by calling `pruneLargest`
  themselves; the second water sweep must see the first's kills; and
  `detectRiverCrossings` runs after all of them (milestone 9). Written as the
  reference writes it.
- **`_killEdge` guards its `adj` splice; `splitEdge` does not** (milestone 2,
  finding 3). **Do not unify them.**
- **`_killEdge` does not unindex**, so a dead edge stays in the spatial grid and
  is filtered by `e.alive` in `nearestNode`/`addStreet` — which matters to
  `lanePass`, laying streets into a graph these passes have thinned.
- **`_killEdge` is the first writer that removes an edge id from `adj` without
  tombstoning through `splitEdge`**, so from here the `alive` filters milestone
  7 proved dead are live.

### Milestone 12 — blocks and parcels

Reference lines 30192-30342 (`buildBlocks` from 30193). `buildBlocks`,
`buildParcels`. Module `blocks`. The first stage whose output is building-sized.

Built out of dependency order, for a reason worth keeping: a street graph has
nothing discrete in it for a renderer to fill, and parcels are the smallest
stage that produces such shapes; every primitive both functions need was
already built and golden-tested at milestones 1-2 (`ensureCCW`, `insetPoly`,
`polyCentroid`, `pointInPoly`, `polyArea`, `polySelfIntersects`, `segInt`,
`edgeBetween`, `extractFaces`, and the `logn`/`chance`/`range` draws). That was
smaller than inventing a Voronoi or straight-skeleton subdivision to fake the
same shapes, and it is the reference's own algorithm. `buildBlocks` skips
`extractFaces`' outer face on milestone 2's first-index-wins tie-break, and
`hashModel` cannot take a partial model, so the golden dumps state directly.

**`Parcel::tone` is this port's own field**, a stable 0..1 scalar a renderer
varies rooftop brightness and saturation with. It is drawn from a **separate**
substream (`'roof-tone'`), never from the per-block `'parcels/…'` stream the
geometry comes out of: one extra draw there would shift every later frontage
and the parcels would stop matching the reference.

**Verification.** Golden, on milestones 2 and 7's terms: five scenarios, ~5,400
parcels, compared by a hash over the complete state (both polygons, face ids,
edge distances, every parcel field) plus written-out anchors. Every constant
was then mutated by one unit: ten were caught, and three survivors were real
coverage holes:

1. **The 2000 m probe ray survived**, because the fixtures' blocks were far
   deeper than the 14-46 m plot depth, so `min(t_min*0.42, depthTarget*1.35)`
   was always won by the depth term and the ray-cast caps never bound.
   `narrow_rows` (~30 m rows) closed it.
2. **`depthTarget*1.35`, the 120 m² floor and the 0.97 area-conservation trim
   survived**, because rectangular faces produce no acute vertices, no slivers
   and no over-filled block. `wedges` (diagonal cuts) closed the first.
3. **The 120 m² floor cannot be reached by a clean street lay** — measured, not
   assumed. `attach_point`'s `SNAP` is 11 m, so any two nodes closer than that
   merge, and an ~11 m cell (the only rectangle near 120 m²) collapses before
   `extract_faces` sees it. The floor guards the slivers `splitEdge` and
   crossing resolution can produce; `lanePass` (milestone 11) is the first stage
   that could produce one, and the place to revisit it.

The 140,000 m² ceiling is pinned by its own boundary test. The 7 m minimum
frontage, 4 m minimum depth, `riverW/2 + 1` wet margin and 0.97 trim are pinned
by the hash for every value the fixtures produce but not at their own boundaries
— `blocks/tests.rs` says so in its header.

**Later source-engine changes to `buildParcels`**: `RC_ENGINE_CHANGES.md` §6r.5
(the frontage-grant retry loop has no upper bound and hangs at low
`frontageWidthVariance`) and §6s.1-§6s.4 (the ward sets the plot grain; the
water test samples only a lot's four corners).

### Milestone 13 — districts and buildings

Reference lines 30344-30682 (`assignDistricts` from 30345). `assignDistricts`,
`bmap`, `rectPoly`, `buildBuildings`, `_rectPts`, `_peristyle`,
`buildFaithSites`. Module `districts`. Building grammars (burgage,
venus-mixed), the terrain-suitability building gate, churches and temples.

- The reference mutates its parcels in place (`district`, `provDistrict`,
  `suitability`, `empty`, `unsuitable`, `built`, `churchyard`); here those seven
  fields live on `Lot`, which borrows the milestone-12 `Parcel`.
  `assignDistricts` produces the `Lot` list.
- **`oreBearing` is a nullable angle in radians** (`_umOreBearing`, line 22613,
  returns `Math.atan2(by,bx) - orient` or `null`), and the ore-yard rule scores
  parcels by projection onto it. `site::Economy::ore_bearing` is declared `bool`,
  so the bearing travels as its own parameter (`GenOpts::ore_bearing`) until
  that field is corrected.
- `assignDistricts` and `buildFaithSites` read one field of `buildHarbour`'s
  return, `quay`, and take it as `Option<&[Vec2]>`; an empty quay is still a
  harbour (milestone 7, finding 10).

**Later source-engine changes here**: `RC_ENGINE_CHANGES.md` §6p (two site
vectors and the industry siting that consumes them), §6q (a `status` gradient on
every parcel) and §6s.1 (one ward classifier shared with `buildParcels`).

### Milestone 14 — amenities

Reference lines 29156-29382 (`buildMarkets` from 29160; `GAMES_SPEC` at 29263 is
inside the range). `buildMarkets`, `buildCivic`, `orientedRect`,
`gamesShapeAt`, `buildGames`. Module `amenities`. Rank-scaled specialised markets
(M-AMEN-1), the civic hall (M-ADMIN) and the population-gated games site
(M-GAMES), honestly omitted where nothing fits.

- Three arguments are narrowed to what the reference reads, and the two it
  mutates come back as reports: `build_markets` takes centroid lists and returns
  index lists (`Markets`) rather than setting `par.cleared` and splicing
  buildings; `buildGames` reads only `p.poly` and `wallState.ring`.
- `buildCivic`'s `anchors` argument is dead in the reference and absent here.
- `buildCivic`'s rank scaling (line 29211) is block 4's only `Math.log10`
  (`js_log10`).

### Milestone 15 — hinterland, decay, details, metrics

Reference lines 30684-30928 (the `details:` header and `FARM_SPEC` table at
30684-30710; `crossesStreet` from 30711). `crossesStreet`, `stripFields`,
`ringFields`, `buildFarmland`, `applyDecay`, `buildDetails`, `computeMetrics`,
and `FARM_SPEC`. Module `hinterland`.

- `buildFarmland` dispatches on the culture's `FARM_SPEC.pattern` (medieval
  strips, Venus rings); `crossesStreet` is the guard both share.
- `buildDetails`' `wallState` and `maxRF`, and `stripFields`' `rng`, are never
  read by the reference; dropped. (`buildFarmland` still *creates* the stream
  `stripFields` ignores — creating a substream is unobservable.)
- `applyDecay` returns index lists instead of writing `ruined`, as milestone 14
  does for markets.
- **`p.churchyard` is unreachable at the reference's own call site**:
  `generate()` calls `applyDecay` at line 31035, five lines before
  `buildFaithSites` (31040), the only function that sets it. Ported anyway,
  because `apply_decay` is a function, not a call site, and tested directly.

**Later source-engine change here**: `RC_ENGINE_CHANGES.md` §8.2, v2.68 —
`buildFarmland`'s `field`/`pasture` polygons were never drawn by the reference.
Assert that every detail kind the generator can emit reaches the renderer,
derived from a real town rather than a hand-written list; and do not build the
furrow hatching the row says a port should not make.

### Milestone 16 — `generate()` orchestration + `hashModel`

Reference lines 30930-31094 (`generate` 30931-31084; `hashModel` 31086-31094).
Module `generate`. The payoff: with every stage ported, this port's `hashModel`
is compared against the reference's over a matrix of seeds, cultures, site
kinds and population targets — the whole-subsystem golden the reference wrote
for exactly this. `hashModel` is checked **last**, because it rounds coordinates
to the centimetre and is the least informative failure; counts, histograms and
branch choices fail first.

- **Two orderings are not interchangeable**: `detectRiverCrossings` after every
  edge-killing pass (milestone 9), and `buildPlaza` in two positions (milestone
  8a).
- **`profile.noWalls` does not exist.** Line 30955 is
  `const walls = opts.walls !== false && !profile.noWalls`, and `noWalls`
  appears in the whole reference exactly twice — that line and the comment above
  it. No profile defines it, so the term is inert and `CultureProfile` has no
  such field.
- Four stages were ported as reporting functions (`build_markets`,
  `apply_decay`, `clear_fort_zone`, `detect_river_crossings`), and `generate()`
  applies their reports; `generate.rs`'s header tables how.
- The capture harness is `cartalith-native/tools/um_capture.js` — see
  "Verification convention" for why it had to be reconstructed.

This port's own stages (wall lots, courtyard rings, citadel) are in
`generate()` too — see "Beyond the reference".

### Milestone 17 — the civ adapter

Block 2, reference lines 22036-22962. The 20 pure functions of the `_um*`
adapter: `_umSiteBoxKm`, `_umWaterNearKm`, `_umWaterReachKm`,
`_umSiteKindFromTerrain`, `_umInferAge`, `_umWallSpec`, `_umInferWalls`,
`_umHarbourScale`, `_umPt`, `_umRayBoxExit`, `_umTerrainOrient`,
`_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`, `_umWaterCtx`,
`_umTerrainCtx`, `_umSiteProfile`, `_umOreBearing`, `_umPlaceContext`,
`_umCacheKey`. The only piece that needs `cartalith-civ`, `cartalith-hydrology`
and `cartalith-terrain`, so it lives **outside** `cartalith-urban`, in
`cartalith-civ::urban_adapter`, and the engine crate stays dependency-light.

- `_umWallSpec` and `_umInferWalls` live in `cartalith-civ::military`, because
  their first real consumer was the faction aggregates' fortified fraction; the
  adapter calls them from there.
- `_umPt` is a JS `[x,y]`-vs-`{x,y}` normaliser with no port target: `Way::pts`
  is typed.
- `_umCacheKey` is pure but keys only the LRU "Out of scope for every milestone"
  excludes.

**Settlement data the reference reads**: `p.specialisation` (→ `opts.economy`,
thence districts and details) and `p.traits` (→ `fortified`) reach the adapter
through `PlaceOverrides` (`specialisation`, `fortified_trait`), written by the
place editor into `cartalith-godot`'s `civ_roster_bridge::PlaceExtrasTable`.
Absent, the fallback is the reference's own — `economy: null`,
`fortified: false` — exactly the reference running on a world where nobody set
them. `civFactionCulture[p.faction]` has no counterpart (this port has no
faction-culture table), so a town is `medieval` unless its own town plan says
otherwise (Ruling J).

### Milestone 17a — the adapter and the first consumer

Out of dependency order on purpose. `PARITY_AUDIT.md` §3.4 found 4,516 lines of
golden-tested engine across milestones 1-7 with **zero consumers**, because this
document's old "don't wire in what nothing calls" rule had held for so long that
the largest subsystem was also the least visible. *(That rule was superseded on
2026-08-23.)*

**The shape:**

- **`cartalith-civ::urban_adapter`** — `um_place_context(_with)` builds an
  `UrbanContext` from a settlement; `run_layout(ctx, rules)` builds
  `cartalith_urban::GenOpts` from it and calls `cartalith_urban::generate`. It
  runs **no stage of its own, in no order of its own**: every stage and its order
  is milestone 16's. `settlement_layout(_with)` is the one call a caller needs.
  *(Until 2026-09-02 `run_layout` ran a hand-ordered subset of `generate()` that
  skipped `buildHarbour`, `addRiverBridges`, `lanePass` and
  `removeWaterCrossings`, and so platted blocks off a graph the reference never
  hands `buildBlocks`. A second pipeline beside a verified one does not stay
  equivalent to it; do not reintroduce one.)*
- **`cartalith-godot::urban_bridge::urban_layouts(indices)`** — the batched
  entry point.
- **The consumers**: the City Viewer (`shell/city_viewer_window.gd`) and the
  map's deep-zoom town layer (`map_overlay.gd`), both drawn by
  `shell/urban_layout_draw.gd`.
- **`compute_civilisation()` never calls this subsystem.** A town is generated
  on demand, per settlement, never as a generation stage.

**Two deliberate deviations:**

1. **`traceRiverPolylines` is hoisted out of `_umWaterCtx`.** The reference calls
   it per settlement — a full-grid walk — and pays for that with the LRU this
   document rules out. The bridge traces once per batch
   (`urban_bridge::traced_river_polys`, shared by `urban_layouts` and
   `settlement_diagnostics`). The call and its result are unchanged.
2. **The map's reveal gate is `_umLayoutAlpha`'s own crossfade band**, 24 km →
   10 km of map-area span, ported verbatim as `map_overlay.gd`'s
   `_urban_layout_alpha`, with `URBAN_MIN_BOX_PX` (16 px) kept underneath as a
   floor so a narrow map area never draws a sub-pixel town. *(Superseded
   2026-08-24: a site-box-in-screen-pixels gate, adopted while the camera
   clamped at a zoom that could not reach a 24 km span. Measured, it first fired
   at a 47 km span on a 16 px speck; do not restore it.)*

**Block 2 is golden-verified too, and the recorded reason it could not be was
wrong.** The blocker read that `_um*` runs inside the host's full civ scope
(`field`, `flowField`, `civWays`, `state`, `_riverNet`, `currentWaterBodies`)
while the harness slices block 4 only. `cartalith-native/tools/um_block2_capture.js`
evaluates all four of the reference's `<script>` blocks in a bare `vm` context,
in the browser's order, with one self-similar `Proxy` standing in for the DOM;
its fixtures are `crates/cartalith-civ/tests/golden_parity_urban_adapter.rs`. It
found two port defects the synthetic-field unit tests had not: `slope_at` on
`f64::hypot` where the reference uses `Math.hypot`, and `um_site_profile`
clamping the resource-context centre where the reference passes the raw point.
`urban_adapter.rs`'s header states which `_um*` functions that golden covers.

---

## Beyond the reference: owner-ruled additions

None of these has an ancestor in v2.10 or v2.11. Under `DECISIONS.md` §7p each
is the standard, not a divergence to undo. Every golden one of them moved is
disclosed case by case beside the test that moved — `generate/tests/golden.rs`'s
header for the whole town, `site/tests.rs`'s for Ruling N.

| Addition | Where | Ruling | What it is |
|---|---|---|---|
| Host-set generation rules | `urban_adapter::run_layout(ctx, rules)`; `shell/generation_rules_window.gd` | none — the Settlement Editor design canvas, artboard `1h` | The reference's host never set `opts.rules` (`RC_ENGINE_CHANGES.md` §6r.1). Here a world-level active rule set reaches `GenOpts::rules` as `Rules::to_patch()`, a fully populated patch; `None` is exactly `DEFAULT_RULES`, which every golden is pinned to. The window's two sliders call the real `apply_wildness`/`apply_plot_chaos` |
| Walled market-town rule set | `rules::MARKET_TOWN_RULES` in `RULES_PRESETS` | H | Shaped on the owner's plan. A `Rules` preset, not a third `CultureProfile`: every knob the plan needs is on `Rules`, and `amenities::games_spec` keys on the profile id, so a new id would silently take Venus's games table |
| Per-settlement town plan | `PlaceOverrides::{culture, variant}` and a per-settlement `rules_preset`, set from the City Viewer's *Town plan* | J | A settlement's culture profile, and its rule set: the world's active rules, `DEFAULT_RULES`, or a named preset. *Regenerate* sets a non-zero variant, which re-keys the position-derived seed |
| Wall lots and faubourg | `wallside::build_wall_lots`, after `buildParcels` on both planning branches | H; AA extends it to radial towns | Intramural lots backing onto the curtain's inner face, and a faubourg cluster against its outer face that `clearFortZone`'s rampart sweep exempts |
| Courtyard perimeter blocks | `courtyard::build_courtyard_rings`, after the wall lots, organic plan only | H; AD sites it | The outermost ring of dense blocks by market distance, re-platted as perimeter lots round an open court |
| Citadel | `citadel::build_citadel`, after the rampart sweep | I; AC sites it | A walled enclosure astride the curtain on its highest ground — towers, keep, court, inner gate and approach street — on organic towns with `pop_target >= CITADEL_MIN_POP` (10 000) and a curtain that is not a bastioned trace |
| Real river binding | `site::WaterCtx::has_real_river_path` | N | Milestone 5, finding 3 |

**Star forts** are the reference's own behaviour (Ruling I found them reachable,
not missing): the place editor's `fortified` trait reaches `GenOpts::fortified`,
and `generate()` grants the bastioned trace to a walled town on the `organic`
gate scheme with `pop_target >= FORT_MIN` (2 500).

### Mutation testing: `courtyard.rs` and `wallside.rs` (2026-09-24)

Neither module is a reference port, so neither has a reference golden. Their
acceptance bar was hand-built geometry plus properties of generated towns, and
until this sweep no mutation run had checked it (`OUTSTANDING_WORK.md` §2.11,
*"Smaller gaps"*). The method is milestone 10's, above: every constant,
comparator and gate, one mutant at a time in an isolated mirror. **Every
survivor was re-run in isolation after a clean rebuild, and all 78 survived
again.** The new tests are at the end of `courtyard/tests.rs` and
`wallside/tests.rs`, under *mutation-sweep fixtures*.

#### `courtyard.rs`: 59 mutations, 54 dead, 4 equivalent, 1 open

29 died to the existing suite and 25 to new tests:

| new test | kills |
|---|---|
| `the_dense_court_window_is_80_square_metres_to_35_percent_inclusive` | `COURT_MAX_FRAC` 0.34, `COURT_MIN_AREA` both ways, both window comparators (a 26 × 24 block's 80 m² court; a 64 × 30 block's court of exactly 35 %) |
| `a_target_met_exactly_by_the_deepest_ring_takes_it_without_bisecting` | `court(hi) >= target` → `>` |
| `the_bisection_moves_down_on_a_midpoint_that_meets_the_target_exactly` | `court(mid) > target` → `>=` (the first midpoint, 15, is exact for a 60 m square) |
| `a_lot_of_exactly_26_square_metres_is_kept` | `>= MIN_LOT_AREA` → `>` |
| `ring_lots_carry_their_own_streets_class_and_age` | edge lookup `(i+1) % n`, the depth mean, both `age` arms, the `'street'` default |
| `the_outer_ring_skips_plazas_and_is_inclusive_at_seventy_percent` | the plaza exclusion; `>= max_d · 0.7` → `>` |
| `a_block_whose_every_lot_is_a_sliver_keeps_its_strip_plat` | the `lots.is_empty()` refusal |
| `every_corner_of_every_lot_must_be_dry_by_both_tests` | the `'river'` margin test, both margins, `riverW / 2`, both halves of `dry`, `>=` → `>`, both `.all`s |

| equivalent | why |
|---|---|
| `court(lo) <= target` → `<` | at a tie the bisection keeps `lo` at 8 (the court shrinks strictly with depth), so it returns 8 either way |
| `poly_self_intersects(quad) \|\|` removed | `court_of` accepts only an inset whose every edge runs the same way as its original. Front and back of each lot are then parallel and same-directed, which is a trapezoid and cannot be a bowtie |
| `area <= 0` → `< 0`, and the area test removed | the same trapezoid has a front of positive length and a depth of at least 8 m, so its area is positive |

| open | why |
|---|---|
| `court_of`'s `> 0` → `>=` | needs an inset edge closed to exactly zero length that `inset_poly` still returns. The collapse measured, a 30 × 16 block at 8 m, is refused by `inset_poly` itself |

#### `wallside.rs`: 161 mutations, 153 dead, 6 equivalent, 1 open, 1 build error

The sweep had 160 mutants plus 1 re-typed. `deepest = 0.0` did not compile (an
ambiguous float for `.max`) and is recorded as a build error, not a survivor;
`0.0f64` is its re-typed twin. 111 died to the existing suite and 42 to new
tests:

| new test | kills |
|---|---|
| `arc_at_a_vertex_is_the_vertex_itself` | `partition_point(c <= s)` → `<` (a lerp that misses its endpoint by an ulp) |
| `arc_at_a_zero_length_last_segment_is_its_start` | `seg <= 0` → `<` |
| `bow_walks_a_closed_arc_one_lap_either_side` | all three lap-offset mutants |
| `project_keeps_the_first_of_two_equidistant_segments` | `d < best` → `<=` |
| `street_face_keeps_the_first_of_two_streets_hit_at_the_same_point` | `h.t < t` → `<=` |
| `crosses_street_ignores_a_dead_street` | the `alive` gate |
| `accepts_bounds_every_test_it_makes` | the area floor both ways, the self-intersection gate, the box's `y` and `hm` sides, the gate clearance `<` → `<=`, the plaza, the fourth (closing) edge's street test |
| `accepts_measures_the_water_margin_on_every_corner_and_the_centroid` | the `'river'` test, both margins, `<` → `<=`, both halves of the water test, the centroid |
| `accepts_refuses_a_lot_touching_a_taken_one_where_the_half_open_test_says_inside` | the bounding-box skip's min-side comparators |
| `outward_probes_three_metres_off_the_chord` | the 3 m probe |
| `a_faubourg_lot_allows_a_bow_of_exactly_one_metre` | `> MAX_BOW` → `>=` |
| `a_faubourg_lot_looks_eight_metres_past_its_depth_for_a_street` | `depth + 8` → 9 |
| `a_faubourg_lot_takes_a_streets_class_only_when_the_street_cuts_it` | both `d < da/db` → `<=`; the class taken from `fb`'s street |
| `a_two_point_land_arc_is_platted_and_every_wall_lot_is_as_old_as_the_town` | `land.len() < 2` → 3; `age` |
| `the_last_intramural_lot_needs_four_metres_of_arc_and_gets_it` | `total − 4` → 5; the 4 m chord minimum → 5 |
| `an_intramural_lot_exactly_six_metres_deep_is_kept` | `< MIN_DEPTH` → `<=` |
| `an_intramural_wedge_of_exactly_twelve_metres_is_kept` | `> 12` → `>=` |
| `gate_quality_is_measured_round_the_closed_circuit_to_the_nearest_gate` | the seam wrap, the short-way-round `min`, the `max(0)` floor |
| `a_faubourg_run_stops_at_the_end_of_an_open_arc` | both open-arc `break`s — see the finding below |
| `a_city_has_three_faubourg_runs` | the third run |

| equivalent | why |
|---|---|
| `project`'s `l2 > 0` → `>=` | a zero-length segment's one point is also an endpoint of its neighbour, at the same arc length, and it wins that tie first; for a two-point arc of identical points both answer 0 |
| the bounding-box skip's two max-side comparators, and the skip removed entirely | the skip is only an optimisation. A box that is disjoint, or that touches on a max side, puts no corner inside under the half-open `point_in_poly` |
| `s < total − 4` → `s < total` | the remainder is under 4 m, so its chord is too and the 4 m guard skips it; the extra draw is on `"wallside/in"`, which nothing reads afterward |
| `deepest` starts at `0.0f64` | every drawn depth is clamped to ≥ `MIN_DEPTH`, and at least one is drawn whenever the run has length |

| open | why |
|---|---|
| the row loop's bound `row_len · (TAPER_ONSET + TAPER_SPAN)` → `· 0.9` | at the original bound `taper_end_chance` is already 1, so the bound itself is redundant; the mutant tightens it to 1.395 of nominal, which only a row that survives every taper draw past that point would expose |

**Finding.** The open-arc `break` in both faubourg loops is backstopped by
`Arc::bow`. `bow` walks every arc one lap either side, open arcs included, so it
finds vertex 0 at arc length `total` and refuses any lot that crosses the end.
The one exception is an arc whose start lies within `MAX_BOW` of the lot's
chord, which is a nearly-closed arc. That is the fixture above: an arc that ends
1 m short of its own start. On an open arc the lap walk is otherwise harmless,
and no behaviour was changed.

## Out of scope for every milestone

- **`_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha`** (block 2), the
  City Viewer renderer that follows them (`_cv*`, from line 22964), and the
  block-1 LOD hook at lines 15603-15610 — canvas rendering and zoom crossfade.
  Not engine port targets: the shell draws towns (`shell/urban_layout_draw.gd`)
  and gates the map layer with `_umLayoutAlpha`'s band (milestone 17a).
- **`_umModelCache`, `_umCacheKey`, `_umScheduleGenStep`, `_umCacheEvict`,
  `_umModelFor`, `_umModelForNow`** — an LRU plus a one-per-frame
  `setTimeout(…,0)` queue, a workaround for the browser's single thread, not
  transliterated. The shell's replacement: `map_overlay.gd` requests at most
  `URBAN_BATCH_MAX` uncached towns per frame (24, the reference's
  `_UM_MODEL_CACHE_MAX`) and keeps one layout per settlement index. A cache of
  generated layouts must be invalidated by every input, the rules included
  (`RC_ENGINE_CHANGES.md` §6r.2).
- **The removed 17 culture profiles.** Documented upstream as history (the
  source repo's `urban-morphology/docs/07-culture-architecture.md` §3.10 — one
  of nine UME design documents under `Cartalith_RC`'s `urban-morphology/docs/`,
  **not vendored here**, unlike `docs/`) after a post-launch pass found them
  visually indistinguishable. Only `medieval` and `venus` are live and ported.
- **`buildGridStreets` and the palimpsest planning mode** — removed upstream,
  with no live caller.

## Verification convention for this subsystem

### The harness

Slice reference lines **28167-31103 as one contiguous block**, plus line 2291
(`mulberry32`, which block 4 deliberately does not define), and evaluate them in
a bare Node `vm.runInContext` with no DOM. Four assertions, all live:

1. **a block-comment balance scan on both boundaries**, with an orphan-close
   counter — Journey Planner milestone 4's design, adopted because an
   unterminated `/*` at a boundary silently swallows the rest of the slice;
2. **the slice's first line is block 4's header comment opening** (tightened
   from "the slice *contains* the `UME` IIFE header", which caught the
   one-line-late case only by luck);
3. **the slice ends at the `module.exports=UME;` export**;
4. **a negative control: block 4 must not define `mulberry32`** — the reason
   line 2291 is spliced in at all.

**The balance scan is necessary, not sufficient; keep the structural asserts.**
Run as a negative control:

| deliberately wrong slice | balance scan | structural asserts |
|---|---|---|
| ends inside a block comment | caught (depth 1) | caught |
| starts 3 lines into the header | caught (1 orphan `*/`) | caught |
| starts **1** line into the header | **not caught** — the header's prose contains `"Gen1's globals"`, and an apostrophe at depth 0 is read as a string delimiter that swallows the stray `*/` | caught |
| starts at the `<script>` tag | not caught | caught |
| ends one line early | not caught | caught |
| starts 7 lines early, swallowing the end of block 3 | not caught | caught |

**`const UME = …` is a lexical binding, not a property of the `vm` context's
global object**, so `ctx.UME` is `undefined` however well the slice ran — one of
the silently-empty-output incidents this project has shipped. Append
`globalThis.__UME = UME;` and assert a real object with a real `cityGen`.

**Exposing a function.** Where `_test` or the public export reaches it, the
expected values are the reference's own output. Where neither does (milestone 5
first), add it with **one anchored replacement of the `return {` line, asserted
to match exactly once**, and check the injected names are functions before
capturing anything; further anchored insertions (milestone 7's `buildWall` stub
and per-epoch observer) are likewise asserted exactly once. **The frozen file is
never written.** Rasters travel in the golden file, or — when too large — are
rebuilt from the same closed form and checked against the fnv1a of the
reference's own cells, so both sides provably run on identical inputs. Compare
bit for bit through `to_bits`, no tolerances; an fnv1a over an exact 64-bit dump
is a bit-for-bit statement, not a tolerance.

**Commit the capture script.** Milestones 1-15's scripts were thrown away and
survive only as prose in their `golden.rs` headers; milestone 16's
`cartalith-native/tools/um_capture.js` had to be reconstructed from those
headers, and is kept. So is block 2's `um_block2_capture.js`.

**Gate the capture on shape.** It refuses to write unless the output is
non-empty and the right shape — endpoints right, expected `null`s really `null`,
each discriminating fixture really discriminating — and the Rust side mirrors
the shape half as its own test, because `zip` stops at the shorter side and a
truncated `golden.rs` would otherwise pass. Three subsystems in this project
shipped a silently empty harness before this rule.

### A golden that passes is not a golden that tests anything

Every golden in milestones 3-8a and 12 matched the reference on its first run,
bar milestone 5's one `Math.exp` probe, and mutation testing is what showed how
little some of them tested. So: mutation-check every constant and comparator;
include at least one quantised or symmetric fixture beside the random ones; and
report every survivor with the invariant it rests on. **"Survived" and "cannot matter" are
different reports, and the second is a test** — an executable assertion of the
invariant. Use a **graded perturbation** (a large change to the same constant)
to show a constant is tested where a 37% nudge is simply below what the fixture
can express.

**Fixture rules, each from a sweep that needed it:**

- **Quantised inputs** (milestone 3). A continuous field never produces an exact
  tie, so a tie-break is invisible until the input is quantised.
- **Just-below-a-boundary inputs** (milestone 4). A quantised output — anything
  rounded, floored or bucketed — cannot see an input change smaller than half
  its step, so a constant inside a quantiser survives every small perturbation.
  Build fixtures just below each boundary as well as on it.
- **Build fixtures out of the geometry under test** (milestone 5). Masks uniform
  along one axis and probe grids of round fractions test almost nothing where
  the thresholds are metres wide; derive probes from the site's own polyline and
  ripple every mask.
- **Clear the coordinate's ulp** (milestone 6). A metre offset added to a large
  coordinate cannot express a one-ulp boundary.
- **Add fixtures; do not substitute them** (milestone 6). Replacing a fixture
  can lose a branch the old one reached — a sweep went 73 → 74 on a round meant
  to improve it. Re-run the full sweep after every fixture change.
- **Use the units the code reads** (milestone 7). A heightfield in metres makes
  every slope test reject; read the consumer before building the input.
- **A two-halved threshold needs a fixture that satisfies both halves**
  (milestone 7), and an invented shape usually satisfies neither; build it the
  way the engine would.
- **A continuous-comparison survivor is closable when one side of the comparison
  is a field the fixture can set** (milestone 8a).
- **A dozen hand-picked rows cannot test a bit-twiddling port** (milestone 6).
  Its first sweep left **63 survivors inside `js_sin`/`js_cos`/`js_log`** from a
  golden table built the way `js_exp`'s and `js_hypot`'s were — twelve rows
  cover twelve paths, not the branches. Pin a libm port with an FNV-1a **hash**
  over tens of thousands of results (24,000 sin, 24,000 cos, 30,000 log there),
  arguments drawn by the reference's own `mulberry32` so both sides evaluate the
  same points, with bands chosen to enter each branch on purpose — two of
  milestone 6's six trig bands exist only to reach `rem_pio2`'s second and third
  correction rounds, which no uniform band reaches.

### Mutation-harness rules, each from an incident

- **Stamp the file's mtime forward, and anchor patterns on code that cannot
  appear in prose** (milestone 3). A mutation written in the same second as the
  previous build was not rebuilt, and a pattern (`dl * 0.5 *`) matched inside a
  doc comment first — two false survivors.
- **Re-run every survivor in isolation** (milestone 4). A combined run reported
  34 survivors that all died individually: it had been reading a stale binary,
  most likely because a sibling fork was building in the same `target/`. A stale
  binary reports a healthy `N passed`, so a "did the tests run" gate cannot
  catch it — but add a parsed-count gate anyway, which catches a filter that
  silently matches nothing.
- **Give the sweep its own `CARGO_TARGET_DIR`** (milestone 6): zero false
  survivors across 600 mutations, where the shared directory produced 34.
- **Validate every structural pattern to match exactly once before the sweep**
  (milestone 5). A pattern matching zero times is otherwise counted as a kill.
- **Snapshot before writing anything; restore from the snapshot; re-run the
  suite as a post-sweep baseline; refuse to start while a lock file exists**
  (milestone 6). Two runners overlapped on one target directory: the first was
  killed mid-mutation, the second read the mutated file as its "original" and
  restored it to that, and `routes.rs` shipped `-(s * 5.61)` for `-(s * 4)`. A
  per-edit `finally` restores to whatever it read. Milestone 4's incident
  corrupts the *report*; this one corrupts the *source*.

### Range corrections

For anyone reading an older citation of this document's ranges. Every range the
plan first wrote needed correcting. **An end that runs too late
silently pulls in the next milestone's header; a start that is too late silently
omits a definition** — so check a range against the file before slicing, and
assert its boundaries in the capture.

| milestone | the plan said | the reference range | what was wrong |
|---|---|---|---|
| 2 | 28363-28513 | 28362-28511 | `edgeBetween` closes at 28511; 28513 is milestone 3's header |
| 3 | 28514-28556 | 28513-28547 | ran nine lines into milestone 5's site-model header |
| 4 | 28212-28289 | 28193-28280 | the start excluded `CULTURE_PROFILES`; the end ran into the `V` helpers |
| 5 | 28557-28742 | 28549-28741 | omitted the site-model header and `shoreFromMask`'s v0.98 note |
| 6 | 28744-28843 | 28743-28833 | ran ten lines into milestone 8's radial header |
| 7 | 29390-29630 | 29384-29630 | omitted `logisticRamp`'s doc comment |
| 8, with 8a | 28844-28970 | 28835-28939 and 28941-28965 | the start omitted the radial header; the end ran into the harbour header |
| 9 | 28971-29159 | 28967-29154 | ran five lines into milestone 14's header |
| 10 | 29631-30037 | 29631-30032 | ran into milestone 11's `clearFortZone` header |
| 11 | 30038-30192 | 30034-30190 | ran into milestone 12's blocks header |
| 12 | 30193-30344 | 30192-30342 | ran into milestone 13's districts header |
| 13 | 30345-30710 | 30344-30682 | ran 28 lines into milestone 15's header and `FARM_SPEC` |
| 14 | 29160-29389 | 29156-29382 | ran seven lines into `logisticRamp`'s doc comment |
| 15 | 30711-30930 | 30684-30928 | `FARM_SPEC` sat in milestone 13's claimed range; the end ran into 16's header |
| 16 | 30931-31086, plus `hashModel` | 30930-31094 | `generate` closes at 31084; 31086 is `hashModel`'s comment |
| 17 | ~22036-22960 | 22036-22962 | `_umDrawLayoutPreview` closes at 22962 |

Several module headers in `cartalith-urban` (`astar.rs`, `blocks.rs`,
`districts.rs`, `amenities.rs`, `hinterland.rs`, `water.rs`) quote the plan's
ranges from before this table.
