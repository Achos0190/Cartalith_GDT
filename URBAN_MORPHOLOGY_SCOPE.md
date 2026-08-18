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
explicitly "for determinism goldens"; and a `_test` export exposing **fifteen**
internal functions (this document said fourteen until milestone 2 counted them:
`polyArea`, `polyCentroid`, `pointInPoly`, `segInt`, `insetPoly`, `clipConvex`,
`extractFaces`, `makeGraph`, `addStreet`, `ensureCCW`, `convexHull`, `simplify`,
`chaikin`, `astar`, `distPtSeg`). Golden verification here is not something this
port has to invent — the reference built the door.

**One caveat on `hashModel`, found at milestone 2**: it takes a finished
`generate()` model and reads `m.graph`/`m.blocks`/`m.parcels`/`m.buildings`, so
it cannot be fed a partial subsystem. It is a **milestone 16** instrument, not a
per-milestone one. Milestones before that get their goldens by dumping state
directly, which is also stricter — `hashModel` rounds coordinates to
`Math.round(n.x*100)`.

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

### Milestone 2 — planar street graph: **done** (2026-08-18)

All 15 functions, reference lines **28363-28512** (the plan said 28363-28513;
`edgeBetween` ends at 28512 and `astar` starts at 28514, so the range was one
line long): `makeGraph`, `gKey`, `gridCellsForSeg`, `indexEdge`/`unindexEdge`/
`edgesNear`, `addNode`, `nearestNode`, `rawEdge`, `splitEdge`, `attachPoint`,
`addStreet`, `addPolylineStreet`, `extractFaces`, `edgeBetween`. Module
`cartalith-urban::graph`; dependencies still `cartalith-rng` only. 7 new tests
(26 in the crate), 19 golden scenarios inside the main one.

**The index design is settled as the plan predicted**: dense `Vec` with
tombstones, ids never reused. Two things the plan did not say, both verified
rather than assumed:

- **`nextN`/`nextE` are not stored.** They are unconditionally `nodes.len()`
  and `edges.len()` — every increment is paired with a `push`, nothing is ever
  removed — and the capture asserts that against the reference's own counters
  on all 19 scenarios rather than leaving it as a claim.
- **`gKey` does not survive as a function.** Its only purpose is to make a
  `Map` key out of two integers; an `(i64, i64)` tuple key is the same
  partition, and the grid map is only ever *probed*, never iterated, so no
  ordering is lost. 15 reference functions land as 14 Rust items.

**`cls` is kept as `&'static str`, not promoted to an enum.** The reference
compares it by string in six places and `hashModel` serialises it verbatim, so
the string is the value; and an enum would have to guess now at the classes
later milestones introduce (`'ringroad'` arrives in milestone 10, `'lane'` in
11, and `grow` passes a variable).

#### Golden verification

`UME._test` reaches `makeGraph`, `addStreet` and `extractFaces` — and that is
enough for **all fifteen**, because the harness dumps the *entire* graph state
after each scripted scenario, not just return values: every node with its
adjacency, every edge including tombstoned ones, **the uniform grid cell by
cell**, and the extracted faces. `attachPoint`, `rawEdge`, `splitEdge` and
`nearestNode` live entirely inside `addStreet`; the index family's whole
observable effect *is* the grid. 19 scenarios, all matching exactly.

**`hashModel()` was not usable here, and that corrects an assumption this
document made.** It serialises `m.graph`, `m.blocks`, `m.parcels` and
`m.buildings` off a finished `generate()` model — there is no way to feed it a
bare graph. It becomes reachable at **milestone 16** and not before. The full
state dump is strictly stronger anyway: `hashModel` rounds node coordinates to
`Math.round(n.x*100)`, so it would not have caught the sub-centimetre
divergences the exact dump does.

**The goldens were mutation-checked**, because a full-state dump can look
thorough and still be vacuous. Perturbing the 26 m index cell, the 0.7 cell
step, the 3×3 cell dilation, the 11 m node snap, the 9 m edge snap, both 3.5 m
guards, the 2.5 m node-promotion radius, the `[0.03, 0.97]` t clamp, the spur
collapse's stack rule, the outer-face tie-break's strict `>`, and swapping
`js_hypot` for `f64::hypot` each break at least one golden. Two scenarios
(`clampT`, `hypotSnap*`) exist only because the first mutation round found
those constants unexercised.

#### The block-comment assertion: what it caught, and a real limit in it

It caught nothing this time — the slice boundaries were already established by
milestone 1 and are unchanged. But running it as a **negative control** (a
thing worth doing to any assertion you rely on) found a genuine hole:

| deliberately wrong slice | caught? |
|---|---|
| ends inside a block comment | yes, unterminated-open depth 1 |
| starts 3 lines into the header comment | yes, once an **orphan-close** counter was added |
| starts **1** line into the header comment | **no** |

The one-line-late case slips through because the scanner treats an apostrophe
at depth 0 as a string delimiter, and block 4's header comment contains the
prose `"Gen1's globals"` — so the stray `*/` gets swallowed as string content.
The orphan-close counter is a real improvement over milestone 1's version and
is kept; the residual hole is covered by the **two structural assertions**
(the slice must contain the `UME` IIFE header and must end at
`module.exports = UME;`), which is what actually pins the boundary. Worth
recording plainly: the balance assert is necessary, not sufficient.

#### Findings that change how later milestones must be built

1. **Encapsulation, verified by grep across all 2,937 lines**: `cell`, `grid`,
   `nextE` and `nextN` are touched **only** by this milestone's functions. No
   later milestone reaches into the spatial index. `nodes`/`edges`/`adj` are
   read widely, always as `n.adj.filter(id => g.edges[id].alive)`.
2. **`g._fromPaths` is a dynamic property, and it needs a real field.**
   `buildPrimariesFromPaths` sets `g._fromPaths = true` (line 28830,
   **milestone 6**) and `builtMassHull` reads it (line 29709, **milestone 10**)
   to discount the bare degree-2 vertices that a resampled real road drags in.
   Milestone 2 deliberately does **not** add the field — nothing sets or reads
   it yet — but milestone 6 must add `Graph::from_paths` and milestone 10 must
   read it, or the enceinte over-encloses along arterials exactly as the
   reference's own v1.01 note describes.
3. **The reference is internally inconsistent about one splice, and the port
   reproduces it.** `splitEdge` removes an edge from `a.adj` with an
   **unguarded** `splice(indexOf(e.id), 1)`, where a miss would silently drop
   the *last* element (JS `splice(-1,1)`); milestone 11's `_killEdge` guards
   the identical splice with `if (k >= 0)`. Unreachable given `rawEdge`'s
   invariant, reproduced rather than hardened, and flagged here so milestone 11
   does not "unify" them.
4. **`addStreet` leaves orphan nodes.** When both endpoints are fresh and every
   resulting link is then rejected by `rawEdge`'s 3.5 m minimum, the nodes stay
   in `g.nodes` with empty `adj`. Pinned by a golden (`tooShort`: 4 nodes,
   1 edge). Every later pass must keep filtering on live adjacency.
5. **The stable hit sort in `addStreet` is a safety property, not a
   behavioural one — a tie is unreachable.** Two crossings at one `t` are the
   same point, so those edges already crossed and share a node whose half-edges
   the `1e-4` guard excludes. Two on-segment nodes at one `t` lie on one
   perpendicular within 2.5 m of the segment, hence within 5 m of each other,
   which `attachPoint`'s 11 m snap prevents. A crossing tied with a node is at
   that node's own foot, ≤2.5 m away, which `splitEdge`'s 3.5 m guard folds
   back. Confirmed by mutation (an unstable sort changes no golden) and by a
   test that re-derives every hit parameter across all 19 scenarios.
6. **Two constants in `addStreet` are redundant inside the engine's own site
   box.** The `1e-4` interior-crossing guards and the `1e-3` node-parameter
   guards survive being loosened to `1e-9`: a hit at `t = 1e-4` is `1e-4·L`
   from an endpoint, so it only escapes `splitEdge`'s 3.5 m fold-back past
   `L > 35 km`, and the node guard past `L > 3.5 km`, against a 1700 × 1250 m
   site box. Kept as written — they are the reference's, and they are what
   stops the degenerate case if the box ever grows.
7. **`extractFaces`' guard arithmetic is subtler than it looks.** JS
   `while (guard++ < 20000)` leaves `guard` at 20001 when the bound stopped it,
   so the post-check `guard >= 20000` also discards a face that closed on step
   20000 exactly. A traversal that hits the guard is **dropped**, not
   truncated. Reproduced as written.
8. **The outer-face tie-break is observable.** A closed loop with one dead-end
   spur yields exactly two faces of equal absolute area (±14400 on the golden),
   and the strict `>` makes the *lowest-indexed* one outer. `buildBlocks`
   (milestone 12) skips the outer face, so this is not cosmetic.

### Milestone 3 — A\* over the cost raster: **done** (2026-08-18)

`astar`, reference lines **28514-28547** (the plan said 28514-28556; `astar`'s
last line is `path.reverse();return path;}` at 28547, and 28548-28556 is a blank
line plus the *site model* header comments that belong to milestone 5's range —
so the stated range over-claimed by nine lines, the same shape of off-by-a-few
the milestone-2 range had). Module `cartalith-urban::astar`; dependencies still
`cartalith-rng` only. 7 new tests (33 in the crate), 25 golden scenarios plus a
30-goal sweep inside two of them.

The plan's own sentence — "the tie-breaking behaviour of that heap is what makes
the path reproducible, so it is ported literally rather than swapped for
`BinaryHeap`" — turned out to be exactly right, and this milestone is the first
in the project that **proved** such a claim instead of asserting it. It is also
the milestone where the verification method itself failed first and had to be
fixed, which is the more useful finding.

#### The goldens were vacuous, and mutation testing is what said so

Seventeen scenarios were written by hand first: degenerate 9 × 1 and 3 × 17
strips, both rectangle orientations, a 500-cost wall with one gap, an infinite
moat, a NaN band and a NaN seal, a zero-cost field, a start-equals-goal case,
two RNG-driven rasters filled by the reference's own exported `stream`, and a
sweep taking **every** cell of a 6 × 5 raster as the goal in turn. All of them
reproduced the reference's paths exactly on the first run.

Then fifteen mutations of the ported algorithm were run against them and **nine
survived**: the `0.9` heuristic weight, the `0.5` trapezoid factor, the `DIRS`
order, all three heap-comparator tie-breaks, `js_hypot` vs `f64::hypot`, the
`if (i === gi) break` early exit, and the dead `INFINITY` guard. Nine of the
twelve behaviours that make this function's output reproducible were not being
tested at all, by a suite that looked thorough and passed.

**The reason is one fact, and it generalises well past this milestone:**

> A **continuously-valued** cost raster essentially never produces two frontier
> entries with exactly equal `f`, so it cannot observe a tie-break at all. Only
> a **quantised** raster can.

An exhaustive search over roughly 800,000 (raster family × size × endpoint pair)
combinations found a discriminator for every surviving mutation, and every
tie-break discriminator came from a quantised field — costs drawn from
`{0.5, 1}`, `{1, 2}` or `{1, 2, 3, 4}`. Eight such scenarios were added
(`tiesHalf`, `tiesLeft`, `tiesRight`, `tiesWide`, `tiesDiag`, `nearAdmissible`,
`trapezoidal`, `greedyTrap`), captured from the reference like all the others.
With them, **fourteen of fifteen mutations die**.

This is not an artificial regime. `buildPrimaries` builds its raster as
`(1 + (slope·3.2)²)·8` and slope over most of a site is flat, so the real 8 m
cost field is *mostly constant* away from the river and the bridge band — the
tie-heavy case is the normal one, not the exotic one.

The single surviving mutation is deleting `if (g0[i] === Infinity) continue;`
from the expansion loop. That is reported rather than papered over: it is
**unreachable in the reference too** — `g0[ni]` is assigned on the line before
every `push`, and `g0[si]` before the start's own push, so no popped index can
still hold the fill value. The line is kept because it is what the reference
writes, and a test asserts the invariant it depends on (no relaxation ever
writes a non-finite `g`) across the infinity and NaN scenarios rather than
asserting the dead branch.

#### `js_hypot` is not a rounding detail here either

Milestone 1 found the V8 discrepancy and milestone 2 proved it changes graph
*topology*. Milestone 3 adds the frequency: over the 4,096 integer offsets a
64 × 64 raster produces, `js_hypot` and `f64::hypot` disagree on **1,398** —
better than a third, all by one ulp. It still took a 64 × 48 quantised raster
(`tiesWide`) to build a golden that notices, because a one-ulp difference only
bites when it makes or breaks an exact `f` tie. The requirement is therefore
asserted directly as well as golden-enforced.

#### What the reference's A\* actually is

Worth writing down, because a later reader will otherwise "fix" it:

- The heuristic is `0.9 ×` Euclidean distance **in cells**, while a step costs
  the trapezoidal mean of two raster values that are metres-scaled (`c·CS`, on
  the order of 8-2000). It is therefore wildly *under*-weighted normally and
  *over*-weighted wherever the raster is cheap.
- There is **no closed set** and no stale-entry check, so cells are re-expanded.
- `if (i === gi) break` stops on the first *pop* of the goal, which under an
  inadmissible heuristic need not be its cheapest path.

So the search is **reproducible, not optimal**, and the golden path is the
specification. A correctness-improving rewrite would silently move every primary
route, and with it every block, parcel and building grown against it.

#### Non-finite cost is the only route to `null`

An 8-connected full grid has no unreachable cell, so `astar` can only return
`null` by arithmetic: an `Infinity` tentative cost fails `c < g0[ni]`, and a
`NaN` one fails it too — every comparison against NaN is false in Rust exactly
as in JS. Both are pinned by goldens (`moat`, `nanSeals`), and the NaN case is
one of the few places in this port where JS and Rust NaN semantics agreeing is
load-bearing rather than incidental.

#### One deliberate divergence, stated plainly

An out-of-range `start`/`goal` **panics** in this port. The reference reads past
its typed arrays, gets `undefined`, and — because `undefined === Infinity` is
false — sails past its own guard and produces nonsense. Its only caller
(`buildPrimaries`' `toCell`) clamps to `[1, W-2] × [1, H-2]` first, so the branch
is unreachable in the engine; loud beats silent for a case that cannot happen.

#### The slice assertions, and one improvement over milestone 2's

Same contiguous 28167-31103 slice plus line 2291, same balance scan with
milestone 2's orphan-close counter. Re-run as a negative control:

| deliberately wrong slice | balance scan | structural asserts |
|---|---|---|
| ends inside a block comment | caught (depth 1) | caught |
| starts 3 lines into the header | caught (1 orphan `*/`) | caught |
| starts **1** line into the header | **not caught** | **caught** |
| starts at the `<script>` tag | not caught | caught |
| ends one line early | not caught | caught |

Milestone 2's residual hole is confirmed to still exist and is confirmed to be
covered. The improvement made here: the first structural assertion is tightened
from "the slice *contains* the `UME` IIFE header" to "**the slice's first line
is** block 4's header comment opening", which is what catches the one-line-late
case directly rather than by luck. A fourth assertion was added as a live
negative control in the other direction — block 4 must **not** define
`mulberry32`, since the whole reason line 2291 is spliced in is that it falls
through to block 1.

The capture also refuses to write a file unless every path is non-empty, starts
at its start cell, ends at its goal cell, the two deliberately-sealed scenarios
really returned `null`, and the whole capture exceeds 300 path cells — the
explicit emptiness gate that three earlier subsystems in this project needed and
did not have.

#### One tooling trap worth recording

The first mutation run reported two false survivors. Cause: `cargo`'s freshness
check is mtime-based, and a mutation written into the same second as the
previous build was silently not rebuilt; and one mutation pattern
(`dl * 0.5 *`) matched inside the function's **doc comment** before it matched
the code, so `String.replace`'s first-occurrence rule mutated a comment and
nothing else. Both were caught by hand-checking a "survivor" and finding it dies
immediately. Any later milestone that mutation-tests should stamp the file's
mtime forward and anchor its patterns on code that cannot appear in prose.

#### Corrections to later milestones

1. **Milestone 5's range is right; milestone 3's was not.** `astar` ends at
   28547, not 28556. Nothing else moves — `shoreFromMask` really does start at
   28557.
2. **Milestone 6 must not "improve" the search.** `buildPrimaries` runs `astar`
   once per external route endpoint over a **copy** of the cost raster with the
   already-used cells multiplied by `0.45`, so the reinforcement is order-
   dependent on `site.routeEnds` and each run inherits the previous run's exact
   cell set. Any change to which cells a path occupies compounds across routes.
3. **The port's `astar` takes `(usize, usize)` cell coordinates and panics out
   of range.** Milestone 6 must reproduce `toCell`'s clamp
   (`max(1, min(W-2, round(p.x/CS)))`) itself rather than relying on the search
   to tolerate a stray endpoint.
4. **Milestone 12 and 13 will hit the same coverage trap.** `buildBlocks` and
   `buildParcels` compare areas and lengths against thresholds; goldens built
   only on continuous random inputs will not exercise their tie-breaks either.
   Build at least one quantised or symmetric fixture per milestone from here on,
   and mutation-check rather than assuming a full state dump is enough.

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

**Must add `Graph::from_paths`** (milestone 2's finding 2): the JS sets
`g._fromPaths = true` as a dynamic property at line 28830, and milestone 10's
`builtMassHull` reads it at line 29709. Milestone 2 left the field out because
nothing set it; it is milestone 6's to add, and skipping it silently
over-encloses the enceinte along arterial roads.

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

`builtMassHull` reads `Graph::from_paths` (milestone 2's finding 2, milestone
6's to add). Also introduces the `'ringroad'` street class.

### Milestone 11 — graph cleanup passes (lines 30038-30192, 6 functions)

`_killEdge`, `pruneLargest`, `removeWaterCrossings`, `privatizeAlleys`,
`clearFortZone`, `lanePass`. Ordering between these is load-bearing —
`detectRiverCrossings` deliberately runs after all of them so a recorded bridge
always has a live road on it.

`_killEdge` guards its `adj` splice with `if (k >= 0)` where milestone 2's
`splitEdge` does not (finding 3). **Do not unify them** — the port reproduces
both as written, and the difference is the reference's, not the port's.

### Milestone 12 — blocks and parcels (lines 30193-30344, 2 functions)

`buildBlocks`, `buildParcels`. Dense: the bisector platting method with
ray-cast depth caps, log-normal frontage/depth draws, overlap filtering and
area conservation. `hashModel`'s block/parcel terms cover this, but only from
milestone 16 — it needs a whole model (milestone 2's finding). Until then, dump
state directly, as milestone 2 did. Note also that `buildBlocks` skips
`extractFaces`' outer face, which milestone 2 pinned as a **first-index-wins**
tie-break on absolute area.

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

Milestone 2 ran that assertion as a **negative control** and found one case it
does not cover (a slice starting exactly one line into the header comment, whose
orphan `*/` gets eaten by an apostrophe in the comment prose). An orphan-close
counter was added — it catches the three-lines-late variant — and the residual
hole is covered by the two structural assertions, which are what actually pin
the boundary. See milestone 2's section for the table. **The balance assert is
necessary, not sufficient; keep the structural asserts.**

Milestone 3 re-ran the same negative control (confirming the hole is still
there and still covered) and **tightened the first structural assertion**: from
"the slice *contains* the `UME` IIFE header" to "**the slice's first line is**
block 4's header comment opening", which catches the one-line-late case directly
rather than incidentally. It also added a fourth assertion as a live negative
control in the other direction — block 4 must **not** define `mulberry32`, since
the entire reason line 2291 is spliced in is that it falls through to block 1.
Use that version.

**And a golden that passes is not a golden that tests anything.** Milestone 3
wrote seventeen scenarios that reproduced the reference exactly, then found by
mutation testing that **nine of fifteen** mutations survived them — because a
continuously-valued input never produces an exact tie, so no tie-break was ever
observed. Every milestone from here on should mutation-check its constants and
comparators and should include at least one **quantised or symmetric** fixture
alongside its random ones. Report survivors rather than hiding them; both
milestones that have done so found their survivors were genuinely dead branches,
which is itself worth knowing. Add an explicit **emptiness / shape gate** to the
capture script too (non-empty output, right endpoints, expected `null`s really
`null`) — three subsystems in this project have shipped a harness that produced
silently empty output and passed every structural check.

Where `_test` or the public export reaches a function, expected values are the
reference's own output. Where it does not (`polySelfIntersects` is the only
milestone-1 case), the test is a real unit test of the ported logic and is
labelled as such — the precedent territory, provinces and `cartalith-spatial`
all set.
