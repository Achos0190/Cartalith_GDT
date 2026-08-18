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

### Milestone 4 — generation rules + culture profiles: **done** (2026-08-18)

`CULTURE_PROFILES` (medieval/organic and Venus/radial), `resolveProfile`,
`DEFAULT_RULES`, `cloneRules`, `resolveRules`, `clamp`, `applyWildness`,
`applyPlotChaos` — reference lines **28193-28280**. Module
`cartalith-urban::rules`; dependencies still `cartalith-rng` only. 10 new tests
(43 in the crate), 53 golden rule cases, 2 golden profiles and 15 golden
`resolveProfile` cases.

**The stated range was wrong at both ends, in opposite directions** — the third
range in this plan to need correcting, and the first whose *start* was wrong:

| | plan | real | |
|---|---|---|---|
| start | 28212 | **28193** (comment) / 28199 (code) | 13 lines late — 28212 is `resolveProfile`, so the stated range **excluded `CULTURE_PROFILES` entirely**, the first item the milestone's own list names |
| end | 28289 | **28280** | 9 lines late — 28281 is blank and 28282-28289 is the `V` vector helper object, which milestone 1 already shipped |

Nothing else moves; milestone 5's `shoreFromMask` still starts at 28557.

Data, not algorithm — but the milestone turned out to contain **the single most
dangerous line in the subsystem so far**, and two survivors' worth of honest
reporting about what a mutation test can and cannot see.

#### `clamp` is where a naive port silently builds a different town

`const clamp=(v,lo,hi)=>Math.max(lo,Math.min(hi,v));` The obvious Rust
transliteration is `lo.max(hi.min(v))`, and it is **wrong**: JS `Math.min` /
`Math.max` *propagate* NaN, Rust's `f64::min` / `f64::max` *absorb* it and
return the other operand. So `applyWildness(rules, NaN)` leaves eight NaN
street fields in the reference, while the naive port's inner `hi.min(NaN)`
hands back `hi` and the outer `max` keeps it — landing **every clamped field on
its own upper bound**. A NaN wildness slider becomes a maximally-wild rule set
that looks entirely plausible, is fed straight into `grow` (milestone 7), and
produces a town nobody can trace back to a rounding rule.

This is the same trap `cartalith-assets` milestone 3 hit from the opposite
direction (`f64::min` absorbing a NaN density where `Math.min` propagated it),
and it is exactly what `cartalith-rust-conventions` exists to catch. The port
routes `clamp` through explicit `js_min` / `js_max` that mirror the source
expression, `wild_NaN` and `chaos_NaN` goldens pin it, and a test carries the
`assert_ne!`-style device `geom::js_hypot` uses so the simplification fails
loudly and with the reason written out.

Two smaller notes on the same function. `f64::clamp` would in fact have agreed
on every reachable input (it is written as comparisons, so it propagates NaN
too) — but it panics when `min > max` where the reference returns `lo`, and it
would have hidden the question entirely. And there is **one documented,
unreachable divergence left**: `Math.min(+0,-0)` is `-0` and `Math.max(+0,-0)`
is `+0`, where the port's comparison form returns whichever operand `<` lands
on. Only two of the eleven clamps have a zero bound, and neither can reach a
`-0` argument (`0.10*(2-w)` is `-0` only if `2-w` is, which subtraction of two
finite doubles never produces; `deadEndBias+(w-1)*0.15` is `+0` at `w == 1`).
Recorded rather than coded around — and it is precisely why two mutations
survive, below.

#### Findings in the data itself

1. **`applyWildness` is not idempotent, and only because of one field.** Ten of
   its eleven assignments recompute from a *hardcoded literal* times `w`, so
   re-applying the same `w` is a no-op for them. `deadEndBias` is
   `clamp(s.deadEndBias + (w-1)*0.15, 0, 0.40)` — it reads its own current
   value and **accumulates**. Applying `w = 2` five times walks it
   0.15 → 0.30 → 0.40 (capped) while nothing else moves. Golden-pinned three
   times over (`wildTwice1p5`, `wildThrice2`, `wildFive2`); `applyPlotChaos`
   by contrast is idempotent.
2. **The sliders overwrite custom values they never read.** A caller who sets
   `branchAngleJitter` through `resolveRules` and then calls `applyWildness`
   loses it, because the formula's base is the literal `0.26`, not the current
   field. The reference's own comment says the sliders "compute new values for
   the individual street/parcel fields"; this is what that means in practice.
   Pinned by `wildOverCustom`.
3. **Four `street` fields and two whole rule groups are untouched by either
   slider**: `explorationDecay`, `segmentLengthMedian`, `marketGradientDecay`,
   `bridgeheadDistance`, and all of `settlement` / the other slider's group.
   Asserted, so a later milestone that finds one of them moved knows it did not
   come from here.
4. **`profile.deadEndBias` does not exist on either live profile.**
   `privatizeAlleys` (line 30097, **milestone 11**) reads
   `clamp((profile.deadEndBias||0) + (rules.street.deadEndBias||0), 0, 0.40)`,
   and the profile side is therefore *always zero* — it was the hook for the
   removed 17 profiles. The capture asserts the absence against the reference's
   own key list and fails if a re-freeze ever adds it; the port carries the
   field as `0.0` so milestone 11 can write the expression as the reference
   writes it.
5. **Four profile fields are read by nothing at all**, verified by grep across
   block 4 *and* the whole host app: `parcelPattern` (the reference documents
   its own death at lines 30225-30227 — the insula platting method it
   dispatched went with the other 17 profiles), `orientation`,
   `civicAnchorLabel`, and `defaultWalls`. **The reference's own provenance
   prose is stale about the last one**: `venus`'s `prov` says "the UI unchecks
   the wall box on selecting this profile", and `defaultWalls` has zero reads
   in v2.10, inside block 4 or out. All four are carried anyway, each with the
   note that killed it. `defaultWalls` is `Option<bool>` rather than `bool`, so
   "profile has no opinion" (`medieval`, key absent) stays distinguishable from
   "profile says no" (`venus`) for whatever eventually honours it; `waterway`,
   which *is* read and only ever as a truthiness test, is a plain `bool`.
6. **Nothing outside block 4 uses any of this milestone's exports.** The whole
   host app touches exactly three names on `UME` — `SITE_WM`, `SITE_HM` and
   `cityGen`. `CULTURE_PROFILES`, `resolveProfile`, `DEFAULT_RULES`,
   `resolveRules`, `cloneRules`, `applyWildness` and `applyPlotChaos` are
   exported for the reference's own headless tests, and consumed internally
   only by `generate()` at lines 30933-30934.
7. **`resolveProfile` has a prototype-chain hole, and the port hardens it.**
   `CULTURE_PROFILES[id]` indexes a plain object literal, so five
   `Object.prototype` names come back **truthy** and sail past the `||`
   fallback: `resolveProfile('toString')` returns a *function*,
   `resolveProfile('__proto__')` returns `Object.prototype`. `generate()` would
   then read `profile.planning` as `undefined`, take the organic branch, and
   crash at `profile.wallGates.scheme`. All five are captured as the
   reference's real behaviour and a golden asserts this port returns `medieval`
   for every one of them instead. A `match` has no prototype chain; reproducing
   the hazard would mean building one on purpose.
8. **`cloneRules` does not survive as a function, and is not quite a deep
   clone.** It is `JSON.parse(JSON.stringify(r))`, which `#[derive(Clone)]`
   already is on a well-formed rule set — the same call milestone 2 made about
   `gKey`. But a NaN round-trips to `null`, and the capture pins that the
   reference really does this. A typed `Rules` has no `null` to land on, so the
   port keeps the NaN. Unreachable inside the engine: `resolveRules` clones the
   all-finite `DEFAULT_RULES` and `Object.assign`s the caller's partial on
   *top* of the clone, so nothing a caller supplies is ever round-tripped.
9. **`subdivisionCap` stays an `f64`.** `applyPlotChaos` writes
   `Math.round(clamp(2*c,1,4))` into it, which is `NaN` for a `NaN` slider, and
   milestone 12 reads it only through `Math.min(P.subdivisionCap,
   Math.floor(age/3))` — where a `NaN` makes the whole expression `NaN` and the
   re-subdivision loop run zero times. Typing it `u32` would have to decide
   what `NaN` becomes, and every choice is a divergence. `Math.round` itself is
   safe as `f64::round` here (they differ only on negative halves, and the
   argument's domain is `[1,4]` plus NaN), and the goldens include the three
   `c` values that land it on `1.5`, `2.5` and `3.5` exactly.
10. **`resolveRules`' merge is per *field*, not per group**, and skips a falsy
    group wholesale (`if(partial[grp])`). Two structural divergences from
    `Object.assign`, both unobservable: the loop iterates `Object.keys(out)`,
    so an unknown *group* is ignored (a typed patch has none), and
    `Object.assign` does copy an unknown *field* inside a known group onto the
    result, where nothing reads it. Pinned by `resolveUnknownGroup` and
    `resolveFalsyGroups`.

#### Mutation testing: 120 mutations, 114 dead, 4 survivors, 2 killed by the compiler

Every numeric literal on a non-comment line was perturbed one at a time (84
mutations), plus 36 hand-written structural mutations covering both clamp
semantics, both `js_min`/`js_max` comparators, every `js_round` alternative,
the `deadEndBias` accumulation, the `2-w` inversions, both `meta` write-backs,
`resolveRules`' per-group and per-field merge, `resolveProfile`'s fallback and
arm order, and eleven profile-table values including the profile array's own
order.

**Two mutations are killed by the compiler rather than by a test**, which is
the strongest outcome available: `[CultureProfile; 2] → 3` and the flattening's
`[f64; 24] → 25`.

**Four genuine survivors, all reported with the invariant they rest on:**

| survivor | why it survives |
|---|---|
| `js_min`'s `b < a` → `b <= a` | the two branches return numerically identical values whenever `a == b`; the only case where *which operand* matters is `+0` vs `-0`, the documented unreachable divergence above |
| `js_max`'s `b > a` → `b >= a` | same |
| `clamp(2*c, **1.0**, 4.0)` → `1.01` | `subdivisionCap` is a **quantised output**: a rounded value cannot observe a change to its inputs smaller than half its own step. Shown by graded perturbation — `1.0 → 1.6` and `1.0 → 0.0` both **die**, `1.0 → 1.01` does not |
| `clamp(2*c, 1.0, **4.0**)` → `4.01` | same; `4.0 → 4.4` survives, `4.0 → 4.6` and `4.0 → 3.0` **die** |

The first mutation round had a **fifth** survivor — the `2` multiplier in
`clamp(2*c,1,4)` — for the same quantisation reason. That one *is* killable,
and three scenarios were added to kill it: `chaos_0p7475`, `chaos_1p2475` and
`chaos_1p7475`, sitting just *below* the rounding boundaries that
`chaos_0p75`/`chaos_1p25`/`chaos_1p75` sit exactly on. This is milestone 3's
lesson arriving from the other side — there, a **quantised input** was needed
before a tie-break could be observed at all; here, a **quantised output** hides
a constant unless some input sits within half a step of a boundary. Both are
the same underlying fact: *a golden can only test what its inputs let the
function express.*

#### A tooling trap worth more than the milestone: false survivors from a shared build

The first combined mutation run reported **34 survivors**. Re-running any one
of them by hand killed it immediately; re-running the structural block alone
killed 34 of 36; re-running the whole 120 killed 114. The switch flipped
mid-run and every mutation after it "survived", so the sweep was reporting a
stale binary's results, not the mutated code's.

The cause was **not** either of milestone 3's two (the mtime stamp and the
comment-anchored patterns were both already in place and both held). The most
likely cause is a sibling fork's concurrent `cargo` activity in the shared
`target/` directory during that window; it did not reproduce on replay. Two
things came out of it that later milestones should carry:

- **Re-run every survivor in isolation before reporting it.** That is what
  caught this, and it is the only check that catches a stale-binary survivor —
  a "did the tests actually run" gate does not, because a stale binary reports
  a perfectly healthy `test result: ok. N passed`.
- **Put an explicit output gate on the mutation runner anyway** (`maxBuffer`
  large enough for a full failure diff, and a parsed `N passed` count with a
  floor). It catches the adjacent failure mode — a filter that silently matches
  zero tests — which is the mutation-harness form of the silently-empty-output
  problem three subsystems in this project have already shipped.

#### Golden verification

All eight items are on `UME`'s **public** export rather than its `_test` one,
so this is the first milestone in the subsystem that needed no indirection at
all. 53 rule cases (defaults, the clone, ten `resolveRules` merge shapes,
fifteen `applyWildness` arguments including all three non-finite ones, four
repeat-application cases, seventeen `applyPlotChaos` arguments, and five
combined sequences), both profiles field by field including the two keys the
reference leaves off `medieval`, and fifteen `resolveProfile` ids. Rule sets
are flattened into one canonical field order and compared **bit for bit** via
`f64::to_bits`, so a NaN must be a NaN and a `-0` could not pass for a `+0`; no
tolerances anywhere.

The capture asserts the reference's own `DEFAULT_RULES` still carries exactly
that key set in exactly that order, so a rule added upstream cannot silently
drop out of the comparison; it asserts neither live profile defines
`deadEndBias`; and the emptiness / shape gate refuses to write unless there are
≥40 scenarios, every one is the right width and all-numeric, ≥30 of them differ
from the defaults, there are exactly two profiles with non-empty provenance
prose, and `applyWildness(NaN)` really did poison the rule set. **Every golden
matched on the first run** — which, per milestone 3, is why the mutation
testing above is the part that matters.

#### The slice assertions, re-run as a negative control

Same 28167-31103 contiguous slice plus line 2291, same balance scan with the
orphan-close counter, same four structural assertions including milestone 3's
tightened first-line form and the `mulberry32` negative control. Re-run
verbatim, with one row added:

| deliberately wrong slice | balance scan | structural asserts |
|---|---|---|
| ends inside a block comment | caught (depth 1) | caught |
| starts 3 lines into the header | caught (1 orphan `*/`) | caught |
| starts **1** line into the header | not caught | caught |
| starts at the `<script>` tag | not caught | caught |
| ends one line early | not caught | caught |
| **starts 7 lines early, swallowing the end of block 3** | **not caught** | **caught** |

Milestone 2's residual hole is confirmed for the third time, and confirmed
covered. The new row is the mirror image of it — a slice that begins *before*
block 4 rather than inside its header — and the balance scan misses that one
too, for the same reason. The first-line assertion is what pins the boundary in
every one of the four cases the balance scan cannot see.

#### Corrections to later milestones

1. **Verify each remaining stated range against the code before slicing.**
   Three for three now (milestone 2 over-claimed by one line, milestone 3 by
   nine, milestone 4 was wrong at *both* ends and by 13 lines at the start).
   Milestone 5's `28557-28742` start is confirmed correct as a side effect of
   this one; the rest are unverified.
2. **Milestone 7 must read `rules.street` through `resolveRules`, not
   `DEFAULT_RULES` directly** — except that `grow` itself writes
   `const rules = opts.rules || DEFAULT_RULES` (line 29446), i.e. it falls back
   to the **raw** defaults rather than a resolved partial. Reproduce that, do
   not "fix" it to call `resolveRules`.
3. **Milestone 11's `privatizeAlleys` gets a zero from the profile side** of
   `clamp((profile.deadEndBias||0)+…, 0, 0.40)` — see finding 4. Write the
   expression as the reference writes it; the port carries the field.
4. **Milestone 12 reads `subdivisionCap` as a float** (finding 9), and
   `buildParcels`' `Math.min(P.subdivisionCap, Math.floor(age/3))` must keep
   NaN-propagating semantics if it is ever restructured.
5. **Milestones 13-15 use `profile.id` as a lookup key** into `GAMES_SPEC`
   (line 29278) and `FARM_SPEC` (lines 30775, 30887), and milestone 16 surfaces
   `profile.name` as `cultureName`. Those two strings are load-bearing values,
   not labels — which is why `CultureProfile`'s fields are `&'static str`, the
   same call milestone 2 made about `Edge::cls`.
6. **Every milestone from here that rounds, floors, buckets or otherwise
   quantises an output** should expect the survivor pattern above: a constant
   inside a quantiser is invisible to any perturbation smaller than half a
   step, and the fixture that kills it is one whose input sits just below a
   boundary. Build one deliberately rather than discovering it in the survivor
   list.

### Milestone 5 — the site model: **done** (2026-08-18)

`shoreFromMask`, `buildSite`, `terrainSuitability` — reference lines
**28549-28741**. Module `cartalith-urban::site`; dependencies still
`cartalith-rng` only. 16 new tests (59 in the crate), 19 golden `shoreFromMask`
scenarios and 36 golden `buildSite` scenarios, each carrying **106 probes** of
the five field closures plus `terrainSuitability`.

**The stated range was one line long at the end, and its start understated the
milestone by eight.** `terrainSuitability` ends at **28741**; 28742 is blank.
28557 is right as the first line of *code*, but 28549-28556 are the site-model
archetype comment and `shoreFromMask`'s own v0.98 note — the block milestone 3
already identified as belonging here when it corrected its own range. So the
real range is **28549-28741**. Four ranges checked, four wrong; **check the
rest.**

#### `Math.exp` is the second V8 libm divergence, and it is far bigger than the first

Milestone 1 found `Math.hypot`. This milestone found `Math.exp`, and the two are
not comparable in scale:

| | disagreements with V8 |
|---|---|
| `f64::exp` (the platform libm) | **20,721 of 240,000** random arguments |
| `geom::js_exp` (this milestone) | **0 of 240,000** |

The very first golden run failed on it — `terrainSuitability` at one probe of
one site, one ulp out — exactly as milestone 1's first `dist_pt_seg` run failed
on `hypot`. V8 calls `base::ieee754::exp`, which is FDLIBM's `__ieee754_exp`.
It is *less* accurate than a modern libm (it promises under one ulp, not correct
rounding), and matching it rather than improving on it is the whole of
`cartalith-rust-conventions`' float rule. Ported as
`cartalith-urban::geom::js_exp` beside `js_hypot`, with the same
`assert_ne!`-style guard: eight golden arguments on which the platform `exp`
gives a different answer, and a test that fails if it ever stops doing so.

**One measured special case, reported rather than explained.** Across 244,000
arguments — 240,000 random, every half- and quarter-integer to ±20, and `1.0`
at ±1 and ±2 ulp — V8 and FDLIBM agree everywhere **except at exactly
`x == 1.0`**, where V8 returns the correctly-rounded `e` and FDLIBM returns one
ulp above it. Reproduced because it was measured, not because its cause is
known. Unreachable from the site model, whose `exp` arguments are all
`-(d²)/(2σ²)` and therefore never positive.

**This retro-fixes milestone 1.** `rng::logn` is
`median * Math.exp(sig * norm())` and had been on `f64::exp`. Its milestone-1
goldens passed, which means they happened to land on values the two libms agree
about — luck, not safety. It now goes through `js_exp`, and those goldens still
pass, which is the check. `logn` has **five call sites** in block 4 (29524 in
`grow`, 30242 and 30288 in `buildParcels`, 30523-30524 in `buildBuildings`), so
every frontage width, plot depth and building dimension in the town is drawn
through it. Milestone 12 would otherwise have found this the hard way, against a
far larger golden surface.

`Math.exp` appears once more after this milestone: `logisticRamp`'s
`1/(1+Math.exp(...))` at line 29392, **milestone 7**.

#### Findings

1. **`buildSite` is two sites wearing one name, and which one is live is decided
   per *field*, not per site.** A real water mask with no river centreline still
   runs the synthetic hills; a real heightfield with no water context still
   invents a synthetic channel. The port therefore carries `Option<WaterCtx>` /
   `Option<TerrainCtx>` rather than the one source enum this plan suggested — an
   enum would have to lie about the mixed cases the host actually produces. Four
   goldens are mixed on purpose.
2. **`kind` is not a closed vocabulary, and the difference is observable
   downstream.** `kind = kind || 'river'` defaults only the falsy case; every
   unrecognised string falls through to the **coastline** branch while still
   being returned verbatim — and milestone 9 compares `site.kind === 'coast'`
   directly (lines 29061, 29081). So an unknown kind and a real coast are
   different sites. `kind` stays a `String`, the same call milestone 2 made
   about `Edge::cls`. Pinned by `atoll`, which shares a seed with `coast` and
   produces a byte-identical shoreline under a different name.
3. **`!!W.riverPath` is truthy for a path too short to be a river.** A one-point
   or empty `riverPath` makes the site river-like (`rk`) — four route endpoints,
   no sea step in `height` — while the water geometry still comes from
   `shoreFromMask`. Goldens `pathOfOne` and `pathEmpty`.
4. **A bay draws one fewer number than a coast.** The coastline branch draws its
   harbour abscissa only when the site is *not* a bay (a bay reuses its own
   indent centre), so `bay` consumes 31 site-substream draws and `coast` 32, and
   their `routeEnds` diverge. Invisible to any fixture that does not pair the two
   on one seed; `bay` and `coast` share seed 5 on purpose, and a test asserts the
   whole draw budget (12 hills, then the branch's own, then 3 or 4 endpoints) by
   advancing a fresh stream by hand and rebuilding the endpoints.
5. **One mask, two different truthiness tests.** `shoreFromMask` takes any
   non-zero cell as water (JS truthiness); `isWater` tests `=== 1`. A cell
   holding `2` is water to the shoreline tracer and land to the water query.
   Reproduced rather than unified — golden `maskTwo`.
6. **`shoreFromMask`'s principal axis can collapse to `(0, 0)`, and then the
   sort is a no-op.** One water cell in a 5 × 5 land field leaves four shoreline
   points whose scatter matrix is perfectly isotropic: `sxy == 0`,
   `l1 - sxx == 0` **and** `l1 - syy == 0`, so the documented fallback
   eigenvector is degenerate too, the `|| 1` on the axis length fires, every
   projection is exactly zero and every comparison ties. The stable sort then
   returns the raster's own row-major order.
7. **The fallback eigenvector is not exotic — a plain horizontal shoreline takes
   it.** With `sxy == 0` and `sxx > syy`, `l1` is exactly `sxx`, so `(sxy,
   l1-sxx)` is `(0, 0)` and the fallback fires on every symmetric coast. It is
   still unobservable unless the shore has points in **two** rows, because
   sorting a row-major list by y is the identity; `twoRowShore` (water along the
   top edge *and* the bottom two rows) is the fixture that finally sees it.
8. **Out of bounds is `undefined`, not a panic, and it reaches three ways**: a
   `NaN` probe coordinate (the clamp propagates it and `arr[NaN]` is
   `undefined`), a `dt` array shorter than its mask, and a terrain raster with
   `mw < 2`. All three become `f64::NAN` here, all three are goldens
   (`shortDt`, `terrainShortGrid`, `terrainOneColumn`), and the port takes the
   deliberate divergence **the other way** from milestone 3's `astar` — loud
   there because the case cannot happen, quiet here because it can.
9. **`bankSide` never returns 0.** `Math.sign(x) || 1` sends a point exactly on
   the centreline, a `-0` cross product and a `NaN` one all to `+1`. `grow`'s
   bridgehead rule and `buildWall`'s far-bank test both read it, so the
   on-the-line case having a definite answer is load-bearing rather than
   incidental. Swept over every vertex of every golden site.
10. **The bridge index starts at `-1`, and `Math.max(0, bi)` is the only thing
    placing the bridge when no slope ever compares.** An all-`NaN` heightfield
    never satisfies `s < bs`, so `bi` survives the loop and the bridge lands on
    `river[0]`. Nothing with a finite height field can exercise that line;
    `terrainAllNaN` exists for it.
11. **The three analytic hills are drawn even when a real heightfield makes them
    dead.** Twelve draws nothing reads — but twelve *positions* in the site
    substream, so a port that skipped them on the real-terrain path would move
    every route endpoint.
12. **`waterPoly` is empty on two of the four paths** (landlocked, and coastal
    with real water) and **nothing inside block 4 ever reads it** — verified by
    grep across all 2,937 lines. Its only consumer is `generate()`'s return
    object at line 31081, i.e. the renderer. Milestone 10 must not treat it as
    the town's water.
13. **Six `||` defaults, of which only the `NaN` arm ever bites**:
    `riverWidthM || 20`, `riverOrder || 0`, `seaLakeCells || 0`, `hMax || 0`,
    `hMin || 0`, and `terrainSuitability`'s `site.riverW || 0`. A `0` width
    really does become 20 (`widthFallbackZero`) and a `NaN` Strahler order really
    does become 0 (`orderNaN`).

#### Golden verification

None of the three functions is on `UME`'s public export **or** its `_test` one —
the first milestone in this subsystem to reach neither. The capture therefore
adds them to the returned object with a **single anchored replacement** of the
`return {` line, asserted to match exactly once; the frozen reference file itself
is never touched, and the injected names are asserted to be functions before
anything is captured.

The `vm` handoff needed one thing worth writing down: `const UME = (() => {…})()`
is a **lexical binding, not a property of the vm context's global object**, so
`ctx.UME` is `undefined` however well the slice ran. This project has shipped
that exact bug before (it is one of the three silently-empty-output incidents the
verification convention lists); the capture appends an explicit
`globalThis.__UME = UME;` and asserts the result before proceeding.

Rasters are **emitted into the golden file** rather than rebuilt on the Rust
side, so both sides provably run on identical inputs. Everything is compared
bit for bit through `to_bits`, with no tolerances anywhere — including `height`
and `slope`, which run through `exp` and `js_hypot`.

The capture's emptiness / shape gate refuses to write unless: there are ≥19
shore and ≥30 site scenarios; at least three shorelines are `null` and at least
six are non-trivial; `plusShape` really came back in row-major order (i.e. the
tie fixture is actually tying); every site's river has ≥2 finite vertices, 3 or
4 route endpoints and 106 probes; height is not constant across a site's probes;
some probe is in water and some site is dry everywhere; `bankSide` took both
signs; `terrainSuitability` reached both 0 and >0.5; the NaN probe really
produced a NaN; `bay` and `coast` really drew different endpoints; `atoll` really
took the coast branch under its own name; both `riverWidthM` fallbacks landed on
20 while an explicit 26 survived; `orderNaN` really zeroed; the all-NaN slope
field really fell back to `river[0]`; `pathOfOne` really is river-like without
being the river; `landlocked` really has no harbour and no water polygon; the
short `dt` really produced a NaN; and a mask of 2s really read as land. The Rust
side mirrors the shape half of that gate as its own test, so a truncated
`golden.rs` cannot make the suite vacuously pass.

**Every golden matched on the first run except one probe of one site**, which is
what surfaced `Math.exp`. After `js_exp` landed, all 36 sites × 106 probes × 6
fields and all 19 shorelines matched exactly.

The slice harness is milestone 3's, verbatim: contiguous 28167-31103 plus line
2291, balance scan with milestone 2's orphan-close counter, and the four
structural assertions including milestone 3's tightened first-line form and the
`mulberry32` negative control. Re-run as a negative control, it reproduces
milestone 4's table row for row.

#### Mutation testing

Every numeric literal on a non-comment line of `site.rs` perturbed one at a time
(207), plus 64 hand-written structural mutations covering every `js_min`/
`js_max`/`js_hypot`/`js_exp` call site, every comparator and tie-break, both
Chaikin passes, the draw order and count, all six `||` defaults, the two mask
truthiness tests, the sort's stability, the fallback eigenvector, and the
bilinear term order. Patterns are validated to match **exactly once** before the
sweep starts, replacements are made by `(line, column)` rather than by first
occurrence, comment and string text is stripped before scanning, and **every
survivor is re-run in isolation**.

**Two rounds of fixture work came out of it, and the first round is the finding.**
The first sweep left **46 survivors**, and almost none of them were equivalent
mutants — they were fixture gaps of two specific shapes:

- **Every water raster was uniform along one axis.** `j >= 9 ? water : land` is
  the obvious hand-built mask, and it makes *every* mutation of `maskIdx`'s `i`
  clamp invisible, because column 0 and column 16 hold identical data. Fixed by
  giving each mask a per-column ripple so **no two adjacent columns agree**.
- **A fixed fractional probe grid never lands near anything.** The site's own
  interesting geometry is a 10-40 m wide band around a polyline; a
  `[0.1, 0.5, 0.9] × [0.1, 0.5, 0.9]` grid essentially never enters it, so the
  whole `riverW/2 + 2` water band, the shoreline half-plane test and both ends
  of `yAtX` were unexercised. Fixed by deriving most probes **from the site's own
  river**: offsets straddling the band boundary at three points along the
  centreline, and a ladder of points a quarter-metre either side of the real
  waterline at nine abscissae (`yAtX` is reimplemented in the *capture* for this
  — fixture code, not ported code).

Probe count went 16 → 65 → 79 → 97 → 106 and scenario count 13/31 → 19/36 across
those rounds. This generalises the lesson milestones 3 and 4 recorded: **a golden can
only test what its inputs let the function express**, and for a geometric
subsystem that means fixtures have to be built *from* the geometry, not sampled
on a grid that ignores it.

**271 mutations, 240 died (2 of them at the type level), 31 survived** — and all
31 were re-run in isolation, which milestone 4's stale-binary incident made
mandatory. This sweep reported no false survivors. The guard still earned its
place: an *earlier* sweep of this milestone was killed and restarted after a
hand-check showed a "survivor" dying immediately, which is exactly the failure
mode the rule exists for.

Two mutations are killed by the compiler rather than by a test — `[Hill; 3] → 4`
and the transposed raster index — the strongest outcome available.

**The 31 survivors, each with the invariant it rests on.** Nothing is hidden and
nothing here is a coverage gap a fixture could close; the ones that *were*
coverage gaps are in the round-by-round list below.

| class | n | why they survive |
|---|---|---|
| dead stores | 10 | `[Hill { x: 0.0, y: 0.0, amp: 0.0, rad: 0.0 }; 3]` — four initialiser fields, every one overwritten by the loop immediately below; `harbour_idx`'s declaration and its landlocked assignment (both overwritten, and a landlocked harbour is `{idx: -1}` regardless); the `Harbour { idx: 0, pt: None }` placeholder in the struct literal, replaced at the end of `build_site`; `bi = -1 → -2`, read only through `.max(0)`; and both `Vec::with_capacity(n + 1) → n + 2`, which is capacity, not length |
| equivalent by the surrounding arithmetic | 6 | `i0 + 1.0 → i0 + 1.5` on both bilinear axes — `i1` is used only as `i1 as usize` and `i0` is integral, so truncation erases the change, and the integral `js_min` bound cannot be crossed by half a step; `if s > 0.0 → s > 1.0` in `bank_side` — the true arm and the final `else` **both return `1.0`**, so any `s ∈ (0, 1]` is unaffected; and all three forms of `vl`'s `|| 1` (the `== 0.0` test, the `1.0` substitute, and removing it outright) — it fires only when the axis length is exactly zero, where `vx` and `vy` are both zero, and `0 / anything` — including `0 / 0`, whose `NaN` comparator differences map to `Ordering::Equal` — leaves every projection tied |
| boundary tests whose two branches compute the same value | 2 | `y_at_x`'s `x <= c[0].x → <` and `x <= c[i+1].x → <`. At a vertex abscissa exactly, the early return gives `c[i].y` and the interpolation gives `c[i].y + 0 · Δ`; at the far end it gives `c[i+1].y` and the next segment gives `c[i+1].y + 0 · Δ`. The branch taken changes; the number does not |
| guards against data the reference cannot produce | 6 | the `(c[i+1].x - c[i].x) \|\| 1` denominator, both halves — reachable only with two shoreline vertices at one `x`, and the abscissae are `i/26 · Wm` through two Chaikin passes, strictly increasing; the degenerate-axis test's `1e-9` — observable only if `hypot(sxy, l1-sxx)` lands in `[1e-9, 1.5e-9)`, where every fixture gives exactly `0` or more than `1e-8`; removing `max(0, ·)` from the eigenvalue — `tr²/4 - det` is algebraically `((sxx-syy)/2)² + sxy²` and so never negative, making the `max` purely defensive (**its constant is not** — `0.0 → 1.0` dies on `twoRowMicro`); `river[idx] \|\| river[0]` losing its fallback — `harbour_idx` is a valid index on every path; and the drift clamp switching to `f64::min`/`f64::max`, which differs from `js_min`/`js_max` only on `NaN`, and the drift is a sum of finite draws |
| need an exact tie a continuous field cannot produce | 4 | the coast harbour search's `<` → `<=` (two shoreline vertices exactly equidistant from a drawn abscissa); `isWater`'s channel band `<` → `<=` (a point-to-polyline distance exactly equal to `riverW/2 + 2`); and `js_hypot → f64::hypot` at two call sites, where a one-ulp difference can only flip a strict `<` if two candidates already agree to within one ulp. **Milestone 3's finding, recurring** — and unlike there it cannot be closed by a quantised raster, because these inputs are polyline distances, not cell costs |
| unobservable through Rust's stable sort | 3 | `else if d > 0.0 → d > 1.0` in the comparator, the same rewritten to compare projections rather than their difference, and `sort_by → sort_unstable_by`. The first was **checked rather than assumed**: Rust's stable sort reaches every ordering decision through the `Less` arm, so downgrading `Greater` to `Equal` still returns a fully sorted result (verified independently on a 16-element `f64` vector whose gaps sit below the mutated threshold). The second differs from the original only on `NaN`/`±∞` projections. `sort_unstable_by` survives because the only fixture with ties between *distinguishable* points is the fully-degenerate `plusShape`, whose four projections are all zero |

**Fifteen mutations that survived the first sweep were killed by fixture work
rather than argued away** — the survivor count went **46 → 35 → 31** across three
rounds, and each round's list was read one entry at a time to decide whether it
was equivalent or merely untested. The purpose-built fixtures, and the constant
each exists for:

| fixture | the constant it makes observable |
|---|---|
| a per-column ripple in every water mask | both of `mask_idx`'s `i`-axis clamps |
| probes derived from the site's own river | the whole `riverW/2 + 2` water band, and `bank_side` on both banks |
| a ±0.25 m ladder around `yAtX(x)` at nine abscissae, plus one probe exactly on it | the sea half-plane's `-1.0` offset and its `>` |
| `riverCeiling` / `throughCeiling`, found by **scanning** seeds | the channel drift's upper clamp — no hand-picked seed saturates it |
| `quayLadder`, 18.85 m per segment (five of them = 94.25 m) | the quay walk's 95 m stop and its accumulator's starting value |
| `twoRowShore` (water along the top edge *and* the bottom rows) | the fallback eigenvector, which a one-row shoreline cannot show because sorting a row-major list by *y* is the identity |
| `twoRowMicro`, the same cloud at 4 mm cells | the eigenvalue guard's own `0.0`, by pushing the discriminant below 1 |
| `vertShore`, a vertical real shoreline | the real-water harbour search's reference *y* |
| `exactlyTwo`, a mask producing exactly two shore points | `pts.len() < 2` |
| `northWater` / `westWater` | the north and west adjacency tests, individually |
| `twoPointPath` | `riverPath.length >= 2` |

#### Corrections to later milestones

1. **Milestone 5's own range was 28549-28741**, not 28557-28742. Four for four;
   verify milestones 6-16's stated ranges before slicing.
2. **Every milestone from here must use `geom::js_exp` for `Math.exp`**, exactly
   as it must use `js_hypot` for `Math.hypot` and `js_min`/`js_max` for
   `Math.min`/`Math.max`. The platform `exp` disagrees with V8 on 8.6% of
   arguments. Milestone 7's `logisticRamp` is the next direct call site;
   `rng::logn` (already fixed here) is the indirect one that milestones 12 and 13
   lean on hardest.
3. **Milestone 6's `placeAnchors` can reach its literal fallback.**
   `site.bridgePt || (site.harbour && site.harbour.pt) || {x: Wm*0.52, y: Hm*0.42}`
   — a landlocked site has `bridgePt = null` *and* `harbour.pt = null`, so the
   third arm is live, not defensive.
4. **Milestone 9 compares `site.kind === 'coast'` as a string** (lines 29061,
   29081). That is why `kind` is a `String` here and not an enum; an enum
   mapping unknown kinds to `Coast` would silently change those two branches.
5. **Milestone 10 must not read `site.waterPoly` as "the water".** It is empty on
   the landlocked and real-water-coastal paths, and nothing in block 4 reads it
   at all — it exists for the renderer. Use `isWater`/`riverDist`.
6. **Milestone 12 and 13 draw every parcel and building dimension through
   `rng::logn`**, which is now on `js_exp`. If either milestone ever sees a
   whole-town divergence that looks like noise, the libm is the first thing to
   check, not the last.
7. **Build fixtures out of the subsystem's own geometry.** Milestone 3 asked for
   quantised inputs and milestone 4 for just-below-a-boundary inputs; milestone 5
   adds that for anything with geometry, the probe set must be *derived from the
   geometry under test*. A grid of round fractions tests almost nothing in a
   subsystem whose thresholds are metres wide.

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
alongside its random ones. Report survivors rather than hiding them; all three
milestones that have done so found their survivors were genuinely dead branches
or provably unreachable divergences, which is itself worth knowing. Add an
explicit **emptiness / shape gate** to the capture script too (non-empty output,
right endpoints, expected `null`s really `null`) — three subsystems in this
project have shipped a harness that produced silently empty output and passed
every structural check.

Milestone 4 added two things to this convention, both from its own mutation run:

- **Quantisation hides constants in both directions.** Milestone 3 found that a
  quantised *input* is needed before a tie-break can be observed. Milestone 4
  found the mirror: a quantised *output* (anything rounded, floored or bucketed)
  cannot observe a change to its inputs smaller than half its own step, so a
  constant inside a quantiser survives every small perturbation. The fixture
  that kills it is one whose input sits **just below** a boundary — build those
  deliberately alongside the ones that sit exactly on it.
- **Re-run every mutation survivor in isolation before reporting it.** One
  milestone-4 sweep reported 34 survivors that all died individually; the
  combined run had been reporting a stale binary, most likely because a sibling
  fork was building in the same shared `target/` at the time. A "did the tests
  actually run" gate does **not** catch this (a stale binary reports a healthy
  `N passed`); only the isolated re-run does. Add the gate anyway — it catches
  the adjacent case of a test filter that silently matches nothing.

Milestone 5 added three more, all of which cost it a restart or a sweep:

- **Build the fixtures out of the subsystem's own geometry.** Milestone 3 asked
  for quantised inputs and milestone 4 for just-below-a-boundary inputs.
  Milestone 5's first sweep left **46 survivors** and almost none were
  equivalent mutants: every hand-built water raster was uniform along one axis
  (so no `i`-clamp mutation was visible), and a `[0.1, 0.5, 0.9]²` probe grid
  never once entered the 10-40 m band around the river where all the thresholds
  live. Deriving the probes from the site's own polyline — and giving each mask
  a per-column ripple — took the survivor count to 31 over three rounds.
  A grid of round fractions tests almost nothing in a geometric subsystem.
- **Validate every structural mutation pattern before the sweep runs.** A
  pattern matching zero times is otherwise silently counted as a kill — the
  mutation-harness form of the silently-empty-output problem. Milestone 5's
  runner refuses to start unless every pattern matches **exactly once**.
- **`const X = ...` in the reference is a lexical binding, not a property of the
  `vm` context's global object.** `ctx.UME` is `undefined` however well the
  slice ran, and the capture must append an explicit
  `globalThis.__UME = UME;` and assert the result. This is one of the three
  silently-empty-output incidents this project has already shipped, met again
  head-on; the fourth structural assertion is that the handoff produced a real
  object with a real `cityGen` on it.

**Where a milestone's functions are on neither `UME`'s public export nor its
`_test` one** — milestone 5 is the first — the capture may add them to the
returned object with a **single anchored replacement** of the `return {` line,
asserted to match exactly once, with the injected names checked to be functions
before anything is captured. The frozen reference file itself is never edited.

Where `_test` or the public export reaches a function, expected values are the
reference's own output. Where it does not (`polySelfIntersects` is the only
milestone-1 case), the test is a real unit test of the ported logic and is
labelled as such — the precedent territory, provinces and `cartalith-spatial`
all set.
