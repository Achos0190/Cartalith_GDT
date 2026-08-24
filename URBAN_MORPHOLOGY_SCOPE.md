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

### Milestone 6 — anchors and primary routes: **done** (2026-08-18)

`placeAnchors`, `buildPrimaries`, `buildPrimariesFromPaths` — reference lines
**28743-28833**. Module `cartalith-urban::routes`; dependencies still
`cartalith-rng` only. 10 new tests (69 in the crate), 38 golden scenarios each
carrying the market, its provenance string, every route polyline, the whole
resulting street graph and a hash of the spatial index.

The first milestone that produces a real street graph end to end, and therefore
the first whose golden is a whole-subsystem artefact rather than a function's
return value.

**The stated range over-claimed by ten lines at the end, and understated the
milestone by one at the start.** `buildPrimariesFromPaths` ends at **28833**;
28834 is blank and 28835-28843 is the *radial streets* header comment, which
belongs to milestone 8. 28744 is right as the first line of code, but 28743 is
the `/* ---------------- anchors ---------------- */` section header, which by
the convention milestones 4 and 5 settled belongs to the milestone it
introduces. **Five ranges checked, five wrong**; milestones 7-16 remain
unverified.

#### `Math.sin`, `Math.cos` and `Math.log` are the third, fourth and fifth V8 libm divergences

Milestone 1 found `Math.hypot`, milestone 5 found `Math.exp` — both **after** a
golden failed. This milestone measured first, which is why every one of its 35
scenarios matched on the first run.

| over 80,214 arguments spanning every reachable reduction branch | disagreements with V8 |
|---|---|
| `f64::sin` | **1,942** |
| `f64::cos` | **2,160** |
| `js_sin` / `js_cos` | **0** / **0** |

| over 60,009 arguments across the whole normal range | |
|---|---|
| `f64::ln` | **1,647** |
| `js_log` | **0** |

`Math.sin` and `Math.cos` are the **third and fourth most-used** functions in
block 4 (27 and 26 call sites, behind only `Math.min`/`Math.max`), and
`placeAnchors` calls both on every one of its 400 candidate points. V8 calls
`base::ieee754::sin`/`cos`/`log`, i.e. FDLIBM's `__ieee754_*`, ported into
`geom` beside `js_hypot` and `js_exp` as `js_sin`, `js_cos` and `js_log`.

**This retro-fixes milestone 1 a second time.** `rng::norm` is
`Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * PI * u2)` and had been on
`f64::ln` and `f64::cos`; milestone 1 asserted its goldens exactly and they
passed, which was luck exactly as `logn`'s were. `norm` is the single
highest-leverage function in the subsystem — `logn` sits on top of it and draws
every frontage width, plot depth and building dimension in the town. The
milestone-1 goldens still pass unchanged, which is the check. `Math.sqrt` needs
no such treatment: IEEE-754 mandates a correctly-rounded square root.

**One branch is deliberately not ported and says so.** For `|x| >= 2^19 * pi/2`
(about 8.2e5) FDLIBM switches to Payne-Hanek reduction — `__kernel_rem_pio2`, a
hundred-odd lines of multi-precision integer arithmetic over a 66-word table.
Every trig argument in this subsystem is an angle built from `range(-PI, 0)`,
`i/n * 2PI`, an `atan2` result or a bearing, none of which can leave
`[-4PI, 4PI]`, so that branch would be dead code with a real chance of being
silently wrong. `js_sin`/`js_cos` hand off to the platform libm above the
threshold, a test asserts they do, and the doc comment names it as the one input
class that is not reproduced.

**The rest of the libm bill, measured now so later milestones do not each
rediscover it.** `Math.atan2` disagrees with `f64::atan2` on **10,615 of
60,000** arguments — 17.7%, the worst yet, and it has 7 call sites starting at
milestone 8. `Math.log10` disagrees on 960/60,000 (milestone 15's
`computeMetrics`), `Math.acos` on 544/60,000 (milestone 10). `Math.pow(x, 2)`
was measured **bit-identical** to `x * x` on 60,000 arguments, so
`buildPrimaries`' one `Math.pow` needs nothing. `Math.sqrt`, `Math.abs`,
`Math.floor`, `Math.ceil` and `Math.sign` are exact by specification.

#### Findings

1. **Neither route builder draws a random number.** `buildPrimaries` and
   `buildPrimariesFromPaths` both take a `seed` and neither reads it — verified
   by grep over both bodies and asserted from the other side by a test that runs
   each with a wildly different seed and requires a byte-identical graph.
   `placeAnchors` is the only RNG consumer here and it draws exactly **800**
   times: two per candidate, **before** any of the four rejection tests, so the
   sequence is independent of the site's shape. Milestone 16 needs all of that
   when it reasons about `generate()`'s draw order.
2. **Both return values are dead.** `generate()` calls whichever builder applies
   for its effect on `g` and **discards** the routes (lines 31021-31022). They
   are returned here anyway — they are what the reference returns, and they make
   a far stricter golden than the graph alone.
3. **The two builders disagree about their own return shape, and nothing
   notices.** `buildPrimaries` pushes `{pts, i}`; `buildPrimariesFromPaths`
   pushes `{pts}` with no `i` at all. Carried as `Route { pts, i: Option<usize> }`
   rather than erased, because erasing it would be a silent decision about a
   field a later milestone might want.
4. **`riverthrough` shares `river`'s candidate band but not its preferred
   distance.** `dBand` tests `kind === 'river' || kind === 'riverthrough'` and
   widens to `[60, 240]`; the score's `Math.abs(d - (kind === 'river' ? 120 : 100))`
   tests `'river'` **alone**. So a bisecting river prefers a market at 100 m like
   a coastal town while still being allowed out to 240 m. Two fixtures share seed
   7 for this; a set with only one of the two kinds cannot see it.
5. **The market reference's third `||` arm is live, as milestone 5 predicted.**
   A landlocked site has no `bridgePt` *and* no `harbour.pt`, so the town centres
   on the literal `{Wm*0.52, Hm*0.42}`. Two fixtures reach it.
6. **`best === null` is reachable too, and only on a small box.** Every one of
   the 400 candidates can be rejected — the market then falls back to
   `{ref.x, ref.y - 120}`, which is the only place in the subsystem that can put
   the market **outside the site box** (at 150 x 150 it lands at y = -57).
7. **The 80 m margin is unobservable on the engine's own box, and that is not a
   fixture gap but a fact about the geometry.** On 1700 x 1250 the reference
   point sits at (884, 525) and candidates reach at most 240 m from it, so no
   candidate is ever within 400 m of the margin. It takes a ~520 m box to make
   the constant do anything at all. Recorded because milestone 7's `grow` reuses
   the same rejection idiom.
8. **`Math.max(0, rd - 260)` — the flood-band penalty — is dead on every site
   this engine can build.** The candidate is drawn at most 240 m from a
   reference point that lies *on* the water on every watered site, so `rd`
   cannot exceed the draw and the term is identically zero; and a landlocked
   site's river is a dummy segment at `(-1e4, -1e4)`, so the term is a ~81-unit
   *constant* that shifts every score equally and cannot move the argmax. A test
   asserts that invariant across all 35 fixtures rather than asserting the dead
   branch — which is what converts two mutation survivors into a statement.
9. **`buildPrimariesFromPaths`' final `sm.length < 2` guard cannot fire.** `pts`
   has at least 2 entries by the guard above it, `simplify` is the identity
   below three points and never drops an endpoint, and `chaikin` on an open
   2-point line returns 4. Reproduced as written, the same call milestone 3 made
   about `astar`'s dead `Infinity` check.
10. **Its `path.length < 2` guard is redundant too, but its `pts.length < 2` one
    is not.** A one-point path yields a one-point in-box run and is dropped by
    the second guard anyway; but a path whose *second* point is outside the box
    leaves exactly the market in `pts`, and without the second guard that
    survives as a degenerate two-identical-point street — which adds a **node**
    and no edge. `pathsOnlyMarket` is the fixture.
11. **A metre offset added to a metre coordinate cannot express a one-ulp
    boundary.** Both of this milestone's boundary fixtures needed rebuilding for
    the same reason: the host's paths are offsets from the market, and
    `(386.6 + 1.0000000000000002) - 386.6` is exactly `1.0` — the two extra ulps
    are absorbed at that magnitude. `> 1` is straddled with 1 m and 1.25 m; the
    6 m box tolerance with -5, -6 and -7. **Any boundary fixture built by
    offsetting a large coordinate has to clear that coordinate's own ulp, not the
    constant's.** This generalises milestone 4's just-below-a-boundary rule and
    every later milestone that takes host-supplied offsets will hit it.
12. **`toCell`'s clamp absorbs the `Math.round` question.** JS rounds halves
    toward `+Infinity` and `f64::round` rounds them away from zero, so they
    differ on negative halves — and a negative cell index clamps to `1`
    regardless, so the divergence cannot be observed. `geom::js_round` is written
    correctly anyway (and `rules`' private copy now routes through it, provably
    identical on its own `[1, 4]` domain), because the next caller may not clamp.
13. **The reinforcement's `Set` iteration order cannot matter.** Each distinct
    cell is multiplied by `0.45` exactly once per route and the indices are
    disjoint, so a `HashSet` reproduces a JS `Set` here without a claim about
    ordering. What *is* order-dependent is the route sequence itself: a test
    reverses `site.routeEnds` and requires the town to change, so the `0.45` can
    never be quietly neutralised.

#### Golden verification

Same slice harness as milestones 3-5, verbatim: contiguous 28167-31103 plus line
2291, the balance scan with milestone 2's orphan-close counter, and the four
structural assertions including milestone 3's tightened first-line form and the
`mulberry32` negative control. None of the three functions is on `UME`'s public
export or its `_test` one, so the capture adds them — with `buildSite` and
`makeGraph`, which the fixtures need — by milestone 5's single anchored
replacement of the `return {` line, asserted to match exactly once, with the
explicit `globalThis.__UME` handoff and its assertion. The frozen reference file
is never touched.

Everything is compared **bit for bit** through `to_bits`, with no tolerances
anywhere. The spatial index is pinned by the reference's **own** `fnv1a` over
its own canonical grid dump rather than cell by cell: milestone 2 golden-tested
the index itself, and restating 400-odd cells per scenario would have added
40,000 lines of golden for no extra strength.

The capture's emptiness / shape gate refuses to write unless: there are ≥37
scenarios, ≥18 driving `buildPrimaries` and ≥12 driving
`buildPrimariesFromPaths`; every market is finite and every provenance string
non-empty; `nextN`/`nextE` still equal the array lengths; every edge is a
7 m-wide epoch-0 `'primary'`; the 80 m margin **rejects >20 and admits >20**
candidates on the two mid-box fixtures and rejects **zero** on the full-size one;
`lastCandidateWins` really wins on candidate 399; `shortDtWater` really admits
>100 candidates and then scores every one of them `NaN`; `tinyBox` really takes
the `best === null` fallback and `landlocked3` really does not; `bay` and
`coast` really diverge on one seed while `atoll` and `coast` really coincide;
`nanCost` really produced no routes at all; `_fromPaths` agrees with the route
count on every paths scenario and is false on at least one; the 1 m unshift
boundary is straddled in both directions; the box-edge triple really keeps 3 of
its 4 points; `bendPath`'s Chaikin corners really separate `simplify(1.2)` from
`simplify(1.3)`; and the whole capture carries ≥400 edges and ≥550 route points.
The Rust side mirrors the shape half of that gate as its own test, because `zip`
stops at the shorter side and a truncated `golden.rs` would otherwise pass.

**Every golden matched on the first run** — all 38 scenarios, every round of
fixture work included. That is the payoff for measuring `sin`/`cos`/`log` before
trusting them rather than after a failure, and it is why the mutation results
below are the part that matters.

#### Mutation testing

Every numeric literal on a non-comment line of `routes.rs` and of `geom.rs`'s new
FDLIBM block, perturbed one at a time (231), plus **69 hand-written structural
mutations** covering every draw and its order, every comparator and tie-break,
both `||` fallback chains, the cost field's three terms, `toCell`'s clamp order
and rounding, the reinforcement's factor and its accumulation across routes, the
`astar` endpoint order, both smoothing pipelines, the street class and width,
`_fromPaths`, the in-box break, the market unshift, and every branch of
`kernel_sin`/`kernel_cos`/`rem_pio2`/`js_log`/`js_round`. Patterns are validated
to match **exactly once in real code** before the sweep starts, replacements are
made by `(line, column)`, comment and string text is stripped before scanning,
and **every survivor is re-run in isolation**.

**Five sweeps: 300 mutations / 98 survivors, then 300 / 79, then 306 / 73, then
306 / 74, and finally 306 mutations, 233 died, 73 survived.** Every survivor was
re-run in isolation and **not one false survivor appeared in any round** —
milestone 4's stale-binary problem, solved by giving the sweep its own
`CARGO_TARGET_DIR` instead of sharing one with the other forks.

**Six of the 306 are deliberate graded perturbations** — milestone 4's device for
a constant whose small change is absorbed — and **all six die**: the sea cost
`240 → 5`, both second-simplify tolerances `1.2 → 4.0`, `toCell`'s lower clamp
`1 → 3`, the margin `80 → 200` on all four sides at once, and the flood-band
penalty `260 → 20000`. Each says *this constant is tested; a 37% nudge is simply
below what the fixture can express*.

**One thing the round-4 sweep taught that no earlier milestone had hit: fixture
coverage is not monotonic when you *replace* a fixture rather than add one.**
Round 4 swapped a trig band whose reduced remainder was ~1e-9 for one whose
remainder is ~1e-13, gaining the third correction round and *losing* the kernels'
own `|x| < 2^-27` shortcut — two mutants a previous round had killed came back.
The survivor count went 73 → 74 on a round that was strictly meant to improve
things. Both bands are now present, which is the final 73.

##### The 19 survivors in `routes.rs`

| class | n | why it survives |
|---|---|---|
| the 80 m margin, three of its four sides | 3 | `marginWinner` is a scanned site whose *winning* candidate sits 80-110 m from **one** edge, so only that side's constant is observable; the other three would each need their own scanned site. The graded `80 → 200`, which moves all four at once, **dies** |
| the flood-band penalty's `0` and `260` | 2 | proven dead rather than argued: a candidate is drawn at most 240 m from a reference point that lies *on* the water, so `rd − 260` is never positive on a watered site; and a landlocked site's dummy river at `(−1e4, −1e4)` makes the term a ~81-unit constant that shifts every score equally. A test asserts that invariant across all 38 fixtures, and the graded `260 → 20000` **dies** |
| the `240` sea cost | 1 | a **barrier, not a cost**: any value large enough to make a water cell non-optimal produces the same path, so `240 → 328.93` cannot move one. The graded `240 → 5` **dies** |
| `toCell`'s two `1.0` lower clamps | 2 | the clamp's result is immediately `as usize`, so a change smaller than one whole cell truncates away — milestone 4's quantised-output pattern, third appearance. The graded `1 → 3` **dies** |
| five comparators that need an exact tie | 5 | the margin's `<` → `<=`, the flood band's `<` → `<=`, the score's `>` → `>=`, the bridge window's `<` → `<=`, the bank band's `<` → `<=`. Every one of those inputs is a continuous distance or score; **milestone 3's finding recurring**, and unlike there it cannot be closed by a quantised raster, because these are polyline distances and sums of RNG draws |
| `bs = −∞ → −1e308` | 1 | no reachable score is below `−1e308`; the initial value's only job is to lose to the first accepted candidate |
| `toCell`'s clamp **order** | 1 | `max(1, min(W−2, ·))` and `min(W−2, max(1, ·))` differ only when `W < 3`, i.e. a site box under 24 m |
| `js_round → f64::round` | 1 | they differ only on negative halves, and a negative cell index clamps to `1` either way. `js_round` is written correctly anyway, because the next caller may not clamp |
| `fromPaths`' `path.len() < 2` → `is_empty()` | 1 | a one-point path yields a one-point in-box run, which the *next* guard drops. That next guard is **not** redundant, and `pathsOnlyMarket` is the fixture that shows it |
| `rem_pio2`'s two round triggers, `16 → 17` and `49 → 50` | 2 | see below — both rounds are load-bearing and tested; what no fixture produces is an argument whose exponent gap is *exactly* 17 or *exactly* 50 |

##### The 54 survivors in the FDLIBM block

| class | n | why they survive |
|---|---|---|
| dead in **this port's** call path | 11 | `js_sin`/`js_cos` filter `|x| ≤ π/4` and Inf/NaN *before* calling `rem_pio2`, so its own early return and its own Inf/NaN branch are unreachable through the public API (8 mutants); `HUGE_ARG_HI` only decides where the platform hand-off starts (1); and the `ix == 0x3ff921fb` sub-branch needs `|x|` inside a 2.3e-8-wide window at π/2 (2) |
| `iy` is a flag, not a value | 5 | `kernel_sin`'s third argument is only ever tested `== 0`, so `1 → 2` is the same call at all four sites; and on the `|x| ≤ π/4` short path `y` is unused, so `0.0 → 1.0` is too |
| ±1-ulp threshold constants | 18 | every `0x…` comparison bound — the four `0x7fff_ffff` absolute-value masks, `0x3e40_0000`, `0x3fd3_3333`, `0x3fe9_0000`, `0x3fe9_21fb`, `0x4002_d97c`, `0x4139_21fb`, `0x7ff0_0000`, `0x0010_0000`, `0x6147a`, `0x6b851`. One ulp of a **high word** only changes behaviour for an argument sitting in that one-ulp window; 54,000 uniform draws never land in one |
| provably equivalent arithmetic | 13 | `0x95f64` is **even**, so the bit its mask can add to `i0` is one `hx` already carries and the `\|` is a no-op — checked by hand after the runner flagged it, because it looks like it should be catastrophic; `qx`'s `0x0020_0000` cancels in `a − (hz − …)`, which is exactly why FDLIBM may pick `0.28125` arbitrarily; `(x as i32) == 0 → == 1` never takes the tiny-x shortcut and the polynomial returns `x` (or `1.0`) anyway; `hx > 0 → > 1` and `hx < 0 → < 1` sit where `hx ∈ {0, 1}` is unreachable; and `js_log`'s five branch selectors pick between two algebraically identical final formulas |
| the staged reduction refines the **tail**, not the returned double | 7 | the four `y[0] → y[1]` index mutations in the medium branch, both `0x7ff` exponent masks, and one more trigger form. **Evidence, not assertion**: never running the second round (`i > 100000`) **dies**, always running the third (`i > −1`) **dies**, and always running the second (`i > −1`) **survives**. So both rounds are load-bearing and both are tested; FDLIBM's first round is already "good to 85 bit" against a 53-bit result, so running one round more than needed is free and running one fewer is not |

#### Two tooling incidents, both worth carrying forward

**A dozen hand-picked rows cannot test a bit-twiddling port.** The first sweep
left **63 survivors inside `js_sin`/`js_cos`/`js_log` alone** — every reduction
threshold, every `y[0]`/`y[1]` slot, both correction-round triggers and the whole
`kernel_cos` `qx` split were untested, by a golden table built exactly the way
`js_exp`'s and `js_hypot`'s were. Twelve rows cover twelve paths through a
branchy function, not its branches. The fix is four lines of golden: an FNV-1a
**hash** of every result over 24,000 sin arguments, 24,000 cos and 30,000 log,
with the arguments drawn by the reference's own `mulberry32` so both sides
provably evaluate the same points, and the bands chosen to enter each reduction
branch on purpose. It matched V8 on the first run and it kills essentially all
63. **Any later milestone that ports a libm function should start there.**

**Two mutation runners on one target directory left a live mutation in the
source.** Round 2 was started twice by accident; the first run was killed
mid-mutation, the second read the already-mutated file as its "original" and
faithfully restored it to that, and `routes.rs` shipped `-(s * 5.61)` where the
reference has `-(s * 4)`. Nothing but the suite failing afterwards said so — the
per-edit `finally` restore is not enough, because it restores to whatever it
read. The runner now takes a **pristine snapshot before it writes anything**,
restores from that snapshot at the end, re-runs the suite as a post-sweep
baseline and refuses to start while a lock file exists. Milestone 4's stale-
binary incident produced the "re-run every survivor in isolation" rule; this is
its sibling, and it is the more dangerous of the two because it corrupts the
*source* rather than the *report*.

(The private `CARGO_TARGET_DIR` this milestone's runner uses did work as
intended for the original problem: **zero false survivors** across 600
mutations, where milestone 4's shared-directory run reported 34.)

#### Corrections to later milestones

1. **Milestone 6's own range was 28743-28833**, not 28744-28843. Five for five;
   verify milestones 7-16's stated ranges before slicing. Milestone 8's range
   should start at **28835** (the radial-streets header comment), not 28844.
2. **Every milestone from here must use `geom::js_sin`/`js_cos` for
   `Math.sin`/`Math.cos`**, exactly as it must use `js_exp`, `js_hypot` and
   `js_min`/`js_max`. Milestone 8's `buildRadialStreets` is the next call site
   and it is trig-saturated.
3. **Milestone 8 needs a `js_atan2`, and cannot borrow the one that now
   exists.** `Math.atan2` is the worst divergence measured — **17.7%** of
   arguments here, 20.4% over the audit's wider range — and it has 7 call sites
   in block 4. A sibling fork landed `cartalith-hydrology::jsmath::js_atan2` on
   the same day (`JS_SEMANTICS_AUDIT.md` §2.3), but `cartalith-urban` depends on
   `cartalith-rng` **only** and must keep doing so, so milestone 8 either copies
   it into `geom` beside `js_sin`/`js_cos`/`js_log`/`js_exp`/`js_hypot` or the
   `cartalith-jsmath` leaf crate the audit recommends finally gets built.
   Milestone 10 needs `js_acos` (0.9%) and milestone 15 `js_log10` (1.6%) on the
   same terms. All are FDLIBM functions; port them against a **bulk hash**
   golden, not a dozen rows — and note the audit's own measurement of
   `js_atan2`'s trap, the `m &= 1` correction V8 carries and the 1993 fdlibm
   source does not.
4. **`Graph::from_paths` exists now** and milestone 10's `builtMassHull` must
   read it (`g._fromPaths && alive.length < 3 && every cls === 'primary'`), or
   the enceinte over-encloses along arterials exactly as the reference's own
   v1.01 note describes.
5. **Milestone 16 gets its primaries for free but must not expect a draw.**
   Neither builder touches the RNG and both return values are discarded by
   `generate()`, so the only things milestone 16 inherits from this one are the
   graph and `placeAnchors`' 800-draw `'anchors'` substream.
6. **A host-supplied metre offset cannot express a one-ulp boundary.** Milestone
   17's adapter produces exactly these offsets (`_umPrimaryPaths`,
   `_umRouteEnds`), so any boundary fixture built on them has to clear the
   *market coordinate's* ulp — roughly 1e-13 m at the engine's box size — not
   the constant's.
7. **The market can land outside the site box.** `best === null` sends it to
   `{ref.x, ref.y - 120}` with no clamp at all. Milestones 7 and 10 measure
   everything from `anchors.market`; on a small or fully-rejecting site that
   origin is not guaranteed to be inside the town, or even inside the box.
8. **`extractFaces` still sorts half-edges with `f64::atan2`, and should not.**
   Milestone 2 wrote `(b.y - a.y).atan2(b.x - a.x)` before anyone had measured
   `Math.atan2`, which is now the largest known divergence in the workspace.
   Its goldens pass and the sort only cares about *order*, so a one-ulp angle
   change bites only when two half-edges at one node point within an ulp of the
   same direction — but that is the same argument that was made for `hypot`
   before milestone 2 proved `hypot` changes graph topology. **Not changed
   here**, because milestone 6's scope is the three route functions and the
   `js_atan2` that landed the same day lives in `cartalith-hydrology`, which
   this crate must not depend on; recorded so that whichever milestone brings
   `js_atan2` into `geom` sweeps this call site too, re-runs milestone 2's 19
   graph scenarios, and reports the result.
9. **Fixture coverage is not monotonic when a fixture is *replaced* rather than
   added.** Round 4's sweep count went *up* (73 → 74) on a round intended to
   improve things, because swapping one trig band for a better one silently gave
   up the branch the old band reached. Add; do not substitute — and re-run the
   full sweep after every fixture change rather than assuming the direction.

### Milestone 7 — organic growth: **done** (2026-08-18)

`logisticRamp`, `estimateCarryingCapacity`, `wallOccupancy`, `grow`,
`supersedeWall` — reference lines **29384-29630**. Module
`cartalith-urban::growth`; dependencies unchanged (`cartalith-jsmath` +
`cartalith-rng`). 15 new tests (84 in the crate), 60 golden scenarios each
carrying the total street length placed, a per-epoch trace of the whole graph,
every node and edge, a hash of every provenance string, the spatial index,
every `buildWall` call and every supersession record.

The scope note said to expect this to be the hardest milestone to land, and to
expect its golden to be a per-epoch graph hash rather than a single end-state
hash. Both held. **Every golden matched on the first run** — the first 48, and the 12
the mutation sweep's second round added.

**The stated range understated the milestone by six lines at the start; the end
was right.** `logisticRamp`'s body starts at 29390, but 29384-29389 is its own
six-line doc comment — the one that flags `k = 6.5` as tuned rather than
measured — which by the convention milestones 4, 5 and 6 settled belongs to the
milestone it introduces. 29630 is exactly `supersedeWall`'s closing brace and
29631 is `ringCrossings`, milestone 10's first function. **Six ranges checked,
six adjusted**, though this is the mildest of the six and the first whose *end*
was correct.

#### Three functions this milestone had to port that belong to later ones

`grow` calls `buildWall` (line 29748, milestone 10), `ringCrossings` (line
29631, milestone 10's first function) and `distToLine` (line 28971, milestone
9's first line). The last two are six and three lines and are ported here as
`ring_crossings` and `dist_to_line`; **milestones 9 and 10 should read them from
`growth` rather than porting them again.**

`buildWall` is 190 lines and is not portable here. It arrives as a
`WallBuilder` trait object — and that is not a design flourish, it is what
made the rest testable. The golden capture **stubs the reference's own
`buildWall`** by a single anchored insertion into the sliced text (the frozen
file is never written to), so the reference side and the Rust side run the same
no-op recorder and every branch that *leads* to a wall — the fire epoch, the
M-GRW-2b age gate, the M-GRW-2a occupancy gate, the generation cap, the
supersession itself — is golden-verified now instead of in three milestones'
time.

**What the stub changes, said plainly, because it is the one place this
milestone's goldens are not the whole engine's behaviour.** A stubbed
`buildWall` never writes `wallState.ring` and never advances `wallState.epoch`.
So (a) a run that starts with `ring: null` can never reach the supersession
branch, which is why the supersession fixtures **preset** a ring; and (b) the
age gate is measured from the initial epoch for every generation instead of
being re-armed by each new circuit, which is why `genSupersede` supersedes twice
in successive epochs where the real builder would make the second wait out
another `wallGenerationMinAgeGap`. Both are identical on both sides and
therefore parity-neutral. **Milestone 10 should re-run this milestone's 60
scenarios with the real builder** and expect the wall-bearing ones to move.

#### `WallState` carries only what milestone 7 touches — milestone 10 must extend it

`generate()` initialises `{ring: null, gates: [], epoch: 0}` (line 31003) and
`buildWall` fills in `waterWalls`, `spurs`, `spansWater`, `style`, `prov`,
`fort`, `centroid`, `terrainDeflected` and `_waterClosure`. `supersedeWall`
copies the first six of those into its history record. **None of them is
modelled here**, exactly as milestone 2 left `Graph::_fromPaths` out until
milestone 6 became the milestone that set it: guessing the shape of `fort` from
a function this milestone does not port is the running-ahead this port avoids,
and leaving a documented hole is not. **Milestone 10 must add those fields to
`WallState` and to `WallGeneration`'s copy list in the same pass.**

#### Findings

1. **`kept` is dead.** `grow` pushes `made[0].id` into a local array that is
   never read, returned or exported. Omitted rather than reproduced — there is
   nothing for it to be equal to. The per-epoch graph hash is the stronger
   instrument the scope note asked for, and it does not need it.
2. **The wet-crossing walk takes six samples, not five, and the last is the
   segment's own endpoint.** `for (let t = 0.15; t <= 1; t += 0.17)` gives
   `0.15`, `0.32`, `0.49`, `0.66`, `0.8300000000000001` and exactly `1.0`. Every
   one of those was read out of `node`; the reasoned answer — that the
   accumulation drifts and the sixth sample is `1.0000000000000002`, so the walk
   stops at five — was **wrong twice over**, and a test now states the measured
   version. Third confirmation of the standing rule that expectations come from
   running `node`, not from reasoning about decimals.
3. **And the accumulation is not load-bearing at these three constants.** `0.15
   + k * 0.17` for `k` in `0..6` is bit-identical to the accumulated walk on all
   six values, and the value past the end is `1.17` either way — measured, not
   assumed, and recorded as a measurement so a later milestone that changes the
   step knows to re-measure rather than inheriting either belief.
4. **A `NaN` slope does not reject.** `NaN > 0.34` is false, so an all-`NaN`
   heightfield stops nothing in `grow`'s legalisation. What it *does* poison is
   `estimateCarryingCapacity`, whose ring average becomes `NaN`, `clamp` returns
   `NaN`, and `maxR` is then `NaN` for the whole run — which makes every
   `dM > maxR` test false, i.e. **removes the reach limit entirely** rather than
   stopping growth. `nanSlopeTown` and `genCcNanTerrain` are the two fixtures.
5. **`opts.rules || DEFAULT_RULES` is the raw table, confirmed by golden rather
   than by reading.** Milestone 4 wrote this forward and it was not "fixed": the
   capture asserts, before writing anything, that a run passing no `opts.rules`
   produces a byte-identical town to one passing an explicit copy of
   `DEFAULT_RULES`, and the Rust shape gate re-asserts it on the two `graph_hash`
   values.
6. **`primEdges` is captured once per epoch, before any street is placed.** So
   streets laid this epoch cannot anchor this epoch's ribbon suburbs — a real
   ordering decision, not an optimisation, and one a "hoist the filter" refactor
   would silently invert.
7. **`wallState.generation || 1` reads a stored `0` as `1`.** Reachable, and the
   `genGenerationZero` fixture reaches it: a preset generation of `0` supersedes
   like a first circuit, while a preset `3` hits `maxWallGenerations` and blocks.
8. **`Math.max(3, Math.floor(epochs * 0.6))` needs three fixtures, not two.** At
   2 epochs the wall never fires at all (the floor is 3 and the run ends at 2);
   at 3 and at 5 it fires at epoch 3 — the `max` arm and the `floor` arm
   respectively — and at 8 it fires at 4. A pair cannot separate the `max` from
   the `floor`.
9. **`buildPrimariesFromPaths` with an empty path list is the empty-graph
   fixture.** `grow` on a graph with no nodes and no edges runs
   `g.nodes[r.int(0, -1)]`, which is `undefined` in JS and `None` here; the loop
   spends its 2,600 tries per epoch and places nothing, without touching the
   RNG budget differently on the two sides.
10. **A harbour with a one-point quay is still a harbour.** The reference tests
    the *object* for truthiness and then indexes `.quay`; `distToLine` over
    fewer than two points is `Infinity`, so `Math.min(dM, Infinity + 35)` is
    just `dM` and the town is the no-harbour town. `harbourEmptyQuay`'s graph
    hash equals `coastTown`'s, asserted.
11. **`estimateCarryingCapacity` is a declared placeholder and is ported as
    one.** Its own header pins the integration contract — same signature, one
    number in ~`[0.3, 1.0]`, never a hard zero, every consumer already treats it
    as "whatever this returns", so replacing this one body is the entire port.
    Replacing it is a Cartalith decision, not a porting one; and the goldens
    have to compare against what the reference actually computes.
12. **The carrying-capacity ring is not clipped to the site box**, and milestone
    6 wrote forward that `anchors.market` is not guaranteed to be inside it
    either. Probes outside the box are not an error — the site model answers for
    any point — but on a raster-backed site they can return `NaN`, which is
    finding 4's path.

#### Golden verification

Same slice harness as milestones 3-6, verbatim: contiguous 28167-31103 plus
line 2291, the balance scan with milestone 2's orphan-close counter, and the
four structural assertions including milestone 3's tightened first-line form
and the `mulberry32` negative control. Three anchored text edits, each asserted
to match **exactly once**: the `return {` replacement that exposes the five
functions plus the builders the fixtures need; the `buildWall` stub; and the
per-epoch observer inside `grow`'s loop. The frozen reference file is never
touched.

Everything is compared **bit for bit** through `to_bits`, with no tolerances
anywhere. `graph_hash` is the reference's own `fnv1a` over its own canonical
dump of every node and every edge with each double written as its exact 64
bits, which is a bit-for-bit statement about the whole graph and not a tolerance
in disguise; the explicit node/edge dump is redundant strictness kept only for
the scenarios under 170 edges, so that a failure is readable — the same trade
milestone 6 made for the spatial index, one scale up. It took the golden file
from 785 KB to 244 KB. `prov_hash` is a second `fnv1a` over every edge's
provenance string, which pins the Exploration/Densification split, the epoch
stamp, and — on the supersession fixtures — the ring-road string's interpolated
`Math.round(fillFraction * 100)`.

The capture's emptiness / shape gate refuses to write unless: there are ≥40
scenarios, ≥30 of which actually grew a street and ≥3,000 edges in total; ≥8
called `buildWall` and ≥3 superseded a circuit; at least one laid a ring road
and `genSupersedeNoArc`/`genSupersedeShortArc` laid **none**;
`genAgeGapBlocks`/`genCapBlocks`/`genOccupancyBlocks` really blocked and
`genAgeGapDelays` really superseded exactly once; `genGenerationZero` really
read its stored `0` as `1`; the four fire-epoch fixtures fired at exactly
`[]`/`[3]`/`[3]`/`[4]` and the preset-ring one did not fire at all;
`emptyGraph` really stayed empty; `seedShortOnly`'s **first** grown street was
an exploration one; `nanSlopeTown` really grew; `genCcNanTerrain` really
produced a `NaN` carrying capacity; the two harbour fixtures really diverge; the
two ring fixtures really diverge; the four rules variants really produce four
distinct towns; the raw-`DEFAULT_RULES` fallback really equals the explicit one;
and every per-epoch trace has exactly one record per epoch. The Rust side
mirrors the whole of it as its own test, because `zip` stops at the shorter side
and a truncated `golden.rs` would otherwise pass.

#### Two rounds of fixtures lost to the same lesson, in two different disguises

Milestone 5's rule — *build the fixtures out of the geometry under test* — cost
this milestone two restarts.

**Round 1: the terrain rasters were in metres.** `site.height` reads
`opts.terrain.grid` **raw** and `site.slope` multiplies a per-metre central
difference by **900**, so a grid holding 40-95 m of elevation produces slopes of
2 to 204 and `grow`'s `slope > 0.34` rejected **every candidate on every
raster-backed site**. Fifteen fixtures grew nothing at all and the two that
worked were the two with no terrain raster. A realistic normalised grid varies
by ~0.1 across the whole box; `TERRAIN_RIDGE` then exists specifically so the
0.34 rejection *does* fire, because a smooth bowl never reaches it.

**Round 2: a hand-drawn ring can never be 80% full.** The M-GRW-2a gate needs
`fillFraction >= 0.8` **and** `exteriorCount >= max(10, interior * 0.15)` — both
halves, which is the whole point of the metric. Ellipses centred on the market
topped out at 0.44; scaling them about the market swept 0.30-0.80 and never got
past 0.58, because a convex hull of a real town's interior nodes does not fill
an ellipse. The first hull-derived attempt then failed the *other* half: the
hull of the **whole** built mass at epoch 3 reaches the box edges along the
primaries, so inflating it 8% enclosed the finished town completely and left
`exteriorCount` at **zero**. What works is the hull of the built mass at epoch 3
**restricted to 260 m of the market** and inflated 6% — which is, not
coincidentally, roughly what `buildWall` itself constructs. A sweep over that
radius shows the gate opening between 180 m and 220 m and staying open.

#### Round 2: twelve fixtures, and seven survivors turned into assertions

Milestone 6's rule — *add, do not substitute* — applied to a survivor list. The
first sweep left 51 survivors; the twelve scenarios below were built to close
the ones that were closable, and every one of them **also matched the reference
on the first run**:

| fixture | the constant it exists for |
|---|---|
| `seedExact38` | a closed square of four **exactly-38 m** edges with no degree-1 node, so neither the mid-edge tap (`dist < 38`) nor the dead-end continuation can fire. `<` and `<=` are different towns |
| `smallBox`, `smallBoxRiver` | 520 × 420 and 560 × 460 boxes, where all four `40 m` box margins actually reject. On the engine's own 1700 × 1250 they never bind |
| `harbourClose` | a quay **40 m** off the market, so `distToLine(quay) + 35` really is the smaller term and both the `35` and the `Math.min` become observable |
| `genAgeGapExact` | 160 years over 8 epochs is 20 a year and `120 / 20` is **exactly 6.0**, so `>=` and `>` differ by one epoch. The only integer-vs-integer boundary in the whole function |
| `genNoAgeRing` | `settlementAge` absent, with the rule gap set to `262.5` so that `262.5 / (300/8)` is **exactly 7.0** — which is what makes the `300` default observable at all |
| `genZeroAgeRing` | `settlementAge: 0` is *falsy*, so it must produce the byte-identical town to an absent one. Without it, dropping `js_truthy_num` is invisible |
| `genTinyAgeRing` | `settlementAge: 0.5` with a 1-year gap: the only setting where `Math.max(1, …)`'s floor decides the answer inside 8 epochs |
| `genExtramuralHigh` | `share = 0.8`, so `interior · share` exceeds the exterior count and the test blocks — which is what says it multiplies the **interior** count |
| `genExtramuralFloor` | **scanned**: `share = 0` pins `max(10, …)` to its floor, and the ring radius (592 m) was searched for the one whose first supersession happens with an exterior count of **exactly 10** |
| `genRingReversed` | the same circuit wound the other way: same interior, opposite signed area, which is all `Math.abs(polyArea(ring))` is for |
| `genSupersedeTwoArc` | a **two-point** `landArc`, between the one-point arc that lays no road and the long one that does |

Seven more were dealt with the other way. A proof does not *kill* a mutant —
a test asserting that a constant cannot matter still passes when the constant
changes — so these are still counted as survivors below. What changed is that
each one now rests on an **executable** statement instead of a paragraph:

- `estimateCarryingCapacity`'s clamp bounds are dead by construction —
  `terrainSuitability` is a product of two `[0, 1]` factors, so `0.3 + 0.7·mean`
  is already inside `[0.3, 1.0]`. Asserted over 720 probes across every site the
  golden file builds. Same shape as milestone 6's flood-band penalty.
- `wallOccupancy`'s `alive` filter cannot bite inside milestone 7: `rawEdge` is
  the only writer of `adj` and `splitEdge` removes the id when it kills an edge,
  so no node ever holds a dead edge. Asserted over all 60 scenarios. Milestone
  11's `_killEdge` is what will make the filter load-bearing.
- the junction-angle double wrap, `abs(((a−b) % π + π) % π)`, is undone by the
  `min(dd, π − dd)` that follows it at both call sites. Measured over 200,000
  arguments, because the mutation that drops it survived and the reason had to
  be established rather than asserted.
- `estimateCarryingCapacity`'s twelve ring angles are `2π·i/12`, and V8's FDLIBM
  and the platform libm agree on **all twelve** — which is why swapping `js_cos`
  for `f64::cos` survives *here*. The test asserts both halves: agreement on the
  twelve, and >100 disagreements in 40,000 arbitrary angles, so the survivor
  cannot be read as a licence anywhere else.
- a zero-area ring cannot contain a node, so `wallArea > 0` sits beside an
  `interior.length >= 8` that can never hold with it.
- `convexHull`'s winding never varies, so `Math.abs(polyArea(hull))`'s `abs` is
  a no-op — while the `abs` on the **ring** is not, which `genRingReversed`
  shows.
- `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0` are only assigned when
  `wallGenerations` is off and only read when it is on. Asserted from the other
  side: with it off, neither the carrying-capacity weight nor the settlement age
  can move the town.

#### Mutation testing

Every numeric literal on a non-comment, non-string line of `growth.rs` (96),
plus **118 hand-written structural mutations** covering every draw and its
order, every comparator and tie-break, both `||` fallbacks, the epoch loop's
two origin branches, the reach and bank tests, the ribbon-suburb rule, the
demand gradient, every legalisation guard, the wet walk, the wall-permeability
loop, the parallel-spacing loop, the street class and width, the provenance
strings, all four arms of the wall episode, every field of the supersession
record, and both helpers borrowed forward. Patterns are validated to match
**exactly once in real code** before the sweep starts, numeric replacements are
made by `(line, column)`, comment and string text is stripped before scanning,
the runner takes a **pristine snapshot before it writes anything** and restores
from that, holds a lock file, runs on a **private `CARGO_TARGET_DIR`**, and
re-runs the suite as a post-sweep baseline.

**Two sweeps: 214 mutations / 51 survivors, then — after twelve new fixtures
and seven new assertions — 214 mutations, 176 died, 38 survived.** Every
survivor was re-run in isolation and **not one false survivor appeared in either
round**, the third milestone running for which the private target directory has
held.

**Eleven of the 214 are deliberate graded perturbations** — milestone 4's device
for a constant whose small change is absorbed — and **all eleven die**: `k`
`6.5 → 30`, the mid-edge minimum `38 → 300`, the junction minimum `18 → 400`,
the slope limit `0.34 → 0.001`, the gate radius `20 → 4000`, the tapped-frontage
skip `1.5 → 500`, the parallel-angle limit `0.5 → 3.2`, the exploration band
`+140 → +5`, the ribbon-suburb radius `90 → 2`, the interior-node floor
`8 → 400`, and the try budget `2600 → 12`. Each says *this constant is tested; a
37% nudge is simply below what the fixture can express.*

##### The 38 survivors, by the invariant each rests on

| class | n | why they survive |
|---|---|---|
| **an exact tie on a continuous value** | 13 | `len < budget`, the bridgehead distance and probability, the 90 m ribbon radius, `h.u > 1e-3`, `h.t > 0.03`, `h.t < hitT`, the 18 m junction minimum, the junction-angle limit, the 0.34 slope limit, the 20 m gate radius, the parallel spacing, and `fillFraction >= 0.8`. **Milestone 3's finding recurring**, and here it cannot be closed the way milestone 3 closed it: every one of these inputs is a polyline distance, an angle, a hull-area ratio or a raw `mulberry32` draw, none of which a quantised raster can pin. Where the boundary *was* integer arithmetic — the age gate, the extramural floor, the 38 m minimum — round 2 built the fixture and the mutant died |
| **proved dead or a no-op, with an executable assertion** | 11 | both carrying-capacity clamp bounds; `wallArea > 0` twice (a zero-area ring contains no node, so the `interior >= 8` beside it can never hold); `ccFactor`'s `: 1` and `yearsPerEpoch`'s `: 0` (assigned only when `wallGenerations` is off, read only when it is on); the probe-ring rotation `i → i+1` (twelve evenly spaced angles are the same twelve points); `js_cos → f64::cos` (V8 and the platform agree on all twelve of *these* angles, asserted together with >100 disagreements over arbitrary ones so it cannot be read as a licence elsewhere); the `alive` filter on `adj` (no node ever holds a dead edge until milestone 11's `_killEdge`); `abs` on the hull area (`convexHull`'s winding never varies — the `abs` on the **ring** does matter, and `genRingReversed` shows it); and the junction-angle double wrap (undone by the `min(dd, π − dd)` that follows it, measured over 200,000 arguments) |
| **an exact integer count no town produced** | 4 | `interior.len() >= 8` in both directions and `hull.len() >= 3` in both. Quantised and therefore closable in principle — it needs a circuit containing *exactly* eight built interior nodes, or one whose interior hull has *exactly* three vertices, while still passing the fill and extramural gates. None of the 60 towns lands there, and unlike the 38 m edge these cannot be constructed by hand: the counts are outputs of the growth loop, not inputs to it |
| **a bound no reachable value approaches** | 5 | `tries < 2600` → 2601 (a 2,601st attempt after 2,600 failures still places nothing); `h.u < 1 − 1e-3` widened twice (`segInt` only ever returns `u ∈ [0, 1]`, so raising the ceiling admits nothing); the wet walk's start `0.15 → 0.3155` (no fixture has a segment wet *only* in that opening slice); and `fmt_js_int`'s `n > 0` sign test, which needs an infinite `fillFraction` |
| **three of the four 40 m box margins** | 2 | the small-box fixtures made growth bind against one edge and killed that side; the other two need their own site whose *growth* is bounded by that specific edge. **Milestone 6's 80 m margin finding recurring exactly** — a margin is invisible until the candidates it removes were going to be kept |
| **provably equivalent rewrites** | 3 | the tapped-frontage skip `1.5 → 2.165` (the frontage sits at ~0 and every other edge is far past either value); `edgesNear(midp, midp) → edgesNear(O, B)` (a superset of cells, but the `d < 24` test measured from `midp` rejects every extra one); and `arc.length > 1 → > 0` (a one-point polyline yields no consecutive pair, so `addPolylineStreet` lays nothing either way) |

#### Corrections to later milestones

1. **Milestone 7's own range was 29384-29630**, not 29390-29630: the six-line
   `logisticRamp` doc comment belongs to it. Six for six. Milestones 8-16 are
   still unverified apart from milestone 8's start, which milestone 6 already
   moved to 28835.
2. **Milestone 14's stated end overlapped this milestone by seven lines.**
   29160-29389 runs past `buildGames`' close at 29382 and into `logisticRamp`'s
   doc comment; it should end at **29382**. Adjusted in place above.
3. **Milestone 9 should not port `distToLine` again** — it is `growth::dist_to_line`,
   and milestone 9's stated range should start at **28967** (the
   `/* ---------------- harbour: quay, piers, mole ---------------- */` header),
   not 28971, by the same convention that moved this milestone's start.
4. **Milestone 10 should not port `ringCrossings` again** — it is
   `growth::ring_crossings`. Its stated range 29631-30037 starts correctly at
   `ringCrossings`, but note that 29638 is the `wall + gates` section header, so
   the milestone contains two sections rather than one.
5. **Milestone 10 must extend `WallState` and `WallGeneration` together.**
   `buildWall` writes nine fields this milestone does not model and
   `supersedeWall` copies six of them into the history record. Adding them to
   `WallState` without adding them to `WallGeneration`'s copy list would produce
   a silently lossy history that every structural test still passes.
6. **Milestone 10 should re-run this milestone's 60 golden scenarios with the
   real `buildWall`.** Twenty-six of them exercise a wall path against a stub; the
   stub is faithful on both sides, but it is not the engine. Expect the
   fire-epoch fixtures to start producing a ring, and expect `genSupersede`'s
   two-in-two-epochs supersession to become one, because the real builder sets
   `wallState.epoch = ep` and re-arms the age gate.
7. **`grow` always enters with `ring: null` from `generate()`, and always with a
   resolved rule set.** Checked rather than assumed, because the first draft of
   this note said the opposite: `generate()`'s only pre-`grow` `buildWall` (line
   31017) is inside the **radial** branch, and that branch does not call `grow`
   at all (lines 31011-31028 are an `if/else`). So the `ep === fireEpoch` arm is
   always live in production, the preset-ring fixtures here are a **superset** of
   what `generate()` can reach, and `opts.rules || DEFAULT_RULES`' fallback arm
   is likewise reachable only by a direct call — `generate()` always passes the
   resolved `rules`. Milestone 16 inherits all three facts.
8. **`grow`'s `opts` object is `generate()`'s literal at line 31027**, and three
   of its ten fields (`wallStyle`, `fortified`, `pop`) are read only by
   `buildWall`. They are on `GrowOpts` for milestone 10 to read; `pop` is read
   by nothing at all in the whole subsystem and may be removable once milestone
   16 confirms it.
9. **A raster-backed fixture in *any* later milestone must use a normalised
   heightfield.** `site.height` returns the grid value untransformed and
   `site.slope` scales by 900; a grid in metres makes every slope test in the
   engine reject. This will hit milestones 10 (`buildWall`'s terrain
   deflection), 13 (`terrainAware` parcels) and 15 (`computeMetrics`).

### Milestone 17a — the adapter and the first consumer: **done** (2026-08-23)

Out of dependency order on purpose, and the reason is recorded rather than
assumed away. `PARITY_AUDIT.md` §3.4 found what this document's own "Out of
scope" section below had prescribed: 4,516 lines of golden-tested engine
across milestones 1-7, with **zero consumers** — no `Cargo.toml` in the
workspace naming `cartalith-urban` but its own, and one disclosure comment
under `godot-project/`. The standing "don't wire in what nothing calls" rule
had held for so long that the largest unported subsystem was also the least
*visible* one; `GUI_GAP_REGISTER.md` had no row for it at all until the same
audit added §6.16.

**What landed.**

- **`cartalith-civ::urban_adapter`** (new module, this document's own named
  home for milestone 17: *"it should live outside `cartalith-urban` (in
  `cartalith-civ`, …) so the engine crate stays dependency-light"*). 13 of the
  28 block-2 `_um*` functions: `_umSiteBoxKm`, `_umWaterNearKm`,
  `_umWaterReachKm`, `_umSiteKindFromTerrain`, `_umInferAge`, `_umRayBoxExit`,
  `_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`, `_umTerrainOrient`,
  `_umWaterCtx`, `_umTerrainCtx`, `_umPlaceContext` — chosen by one rule: a
  function is ported when milestones 1-7 can consume its output. Plus
  `run_layout`, the prefix of `generate()` (line 30931) those seven supply:
  the scalar derivations, `buildSite`, the `routeEnds` override, `placeAnchors`,
  the real-water market pin, `buildPrimaries`/`buildPrimariesFromPaths` and
  `grow`.
- **`cartalith-godot::urban_bridge`** — one batched `#[func]`,
  `urban_layouts(indices)`.
- **The GUI**: `shell/urban_layout_draw.gd` (`_umDrawLayout`/
  `_umDrawLayoutPreview`, which are one drawing twice), `shell/
  city_viewer_window.gd` (`cityViewerModal` — canvas, wheel-zoom, drag-pan,
  legend, info panel), `map_overlay.gd`'s "Urban layouts" block
  (`civUrbanLayoutsChk`), and `right_dock.gd`'s Settlement ▸ Actions ▸ City
  layout as the launcher.

**What is deliberately not ported, by category.**

| Function | Why absent |
|---|---|
| `_umWallSpec`, `_umInferWalls` | the whole fortification pipeline is milestone 10; `walls` is passed `false`, because with no `WallBuilder` in existence a wall spec is a value nothing can build or draw |
| `_umHarbourScale` | consumed only by `buildHarbour`, milestone 9 |
| `_umSiteProfile` | its consumers are the wall spec (10), harbour/bridge validity (9), economic districts (13) and a Settlement Inspector — none exist |
| `_umOreBearing` | feeds `economy.oreBearing`, read only by 13/15; and this port's settlements carry no `specialisation`, exactly the gap milestone 17's own note below predicted |
| `_umPt` | a JS `[x,y]`-vs-`{x,y}` normaliser; `Way::pts` is typed |
| `_umCacheKey`, `_umCacheEvict`, `_umScheduleGenStep`, `_umModelFor`, `_umModelForNow` | "Out of scope for every milestone" below, verbatim |
| `_umDrawLayout`, `_umDrawLayoutPreview`, `_umLayoutAlpha` | likewise — Godot's job, and the GUI files above are that job |

**Two honest deviations, both recorded rather than absorbed.**

1. **`traceRiverPolylines` is hoisted out of `_umWaterCtx`.** The reference
   calls it per settlement — a full-grid walk — and pays for that with the LRU
   this document rules out. The bridge traces once per *batch* instead. The
   call and its result are unchanged; only where it is made.
2. **The map layer's reveal gate is not `_umLayoutAlpha`.** Its 24 km → 10 km
   viewport-span crossfade cannot fire on this port: `ViewportHost.ZOOM_MAX`
   is 8.0, so the default 800 km world's closest reachable span is ~100 km. A
   ported constant that never once fires is a silently-empty surface, which is
   this project's own most-repeated failure mode. The gate is the town's
   1.7 km site box measured in screen pixels instead — a stated rendering
   choice, not a ported one.

**Not golden-verified, and this is the one caveat that matters.** The
verification convention below slices block **4**; the `_um*` functions live in
block 2 and run inside the host's full civ scope (`field`, `flowField`,
`civWays`, `state`, `_riverNet`, `currentWaterBodies`). There is no block-2
fixture and building one is a real harness effort. Every function is ported by
reading the reference line by line with its constants carried verbatim and
cited, and covered by 11 ordinary unit tests over synthetic fields — including
the two that would catch the failure this project keeps rediscovering: that a
real settlement produces a *non-empty* street graph, and that no street class
milestones 8+ own has leaked in. Closing this properly is milestone 17's
remaining half.

### Milestone 12 — blocks and parcels: **done** (2026-08-24)

Out of dependency order, like 17a before it, and for a comparable reason. The
City Viewer (§17a's own first consumer) drew a wire diagram, because a street
graph has nothing discrete in it to fill: no shape smaller than the whole
town. Parcels are the **smallest stage that produces one**, and every
primitive `buildBlocks`/`buildParcels` need had already been built and
golden-tested at milestones 1-2 — `ensureCCW`, `insetPoly`, `polyCentroid`,
`pointInPoly`, `polyArea`, `polySelfIntersects`, `segInt`, `edgeBetween`,
`extractFaces`, and the `logn`/`chance`/`range` draws. Two functions, no new
kernel. It was a smaller change than inventing a Voronoi or straight-skeleton
subdivision to fake the same shapes, and unlike one it is the reference's
own algorithm.

**What landed.** `cartalith-urban::blocks` — `build_blocks` and
`build_parcels`, ported from reference lines 30193-30344 (the range this
document already gave, verified against the file before slicing and correct at
both ends). `UrbanLayout` gained `blocks`/`parcels`, `urban_bridge` emits
them, and `urban_layout_draw.gd` draws them.

**One field is this port's own and is marked as such:** `Parcel::tone`, a
stable 0..1 scalar a renderer varies a rooftop's brightness and saturation
with. It is drawn from a **separate** RNG substream (`'roof-tone'`), never
from the per-block `'parcels/…'` stream the geometry comes out of — one extra
draw from that stream would shift every subsequent frontage width and the
parcels would stop matching the reference's.

**Verification.** Golden, on milestones 2 and 7's terms: the reference's own
`buildBlocks`/`buildParcels` run under `vm.runInContext` over the frozen
file's block 4, with both slice boundaries and the comment balance asserted
and the two functions exposed by one anchored replacement asserted to match
exactly once. Five scenarios, ~5,400 parcels, compared by a hash over the
complete state (both polygons, face ids, edge distances, and every parcel
field) plus written-out anchors. **All five passed unmodified on the first
run.**

**What the mutation sweep found, which is the part worth reading.** Every
constant was mutated by one unit and the suite re-run. Ten were caught. Three
survivors were real coverage holes and two new scenarios closed them:

1. **The 2000 m probe ray survived** three scenarios, because their blocks are
   far deeper than the 14-46 m plot depth, so `min(t_min*0.42,
   depthTarget*1.35)` is always won by the depth term and the ray-cast caps
   never bind at all. `narrow_rows` (~30 m rows) fixed it.
2. **`depthTarget*1.35`, the 120 m² floor and the 0.97 area-conservation trim
   survived** because rectangular faces produce no acute vertices, no tiny
   slivers and no over-filled block. `wedges` (diagonal cuts) fixed the first.
3. **The 120 m² floor cannot be reached at all**, and this is the finding to
   carry forward: `attach_point`'s `SNAP` is 11 m, so any two nodes closer
   than that merge, and an ~11 m cell — the only rectangular shape with an
   area near 120 m² — collapses before `extract_faces` ever sees it. Measured,
   not assumed. The floor guards the degenerate slivers `splitEdge` and
   crossing-resolution can produce, not anything a clean street lay can make.
   **Milestone 11's `lanePass` is the first stage that could produce one**,
   and is where this is worth revisiting.

The 140,000 m² ceiling is pinned by its own boundary test. The 7 m minimum
frontage, 4 m minimum depth, `riverW/2 + 1` wet margin and the 0.97 trim are
pinned by the hash for every value the fixtures produce, but not at their own
boundaries — `blocks/tests.rs` says so in its header rather than leaving it
implied.

**Three upstream stages are missing, and milestone 12 runs without them.**
This is the honest cost of taking it out of order, and it is a property of the
*input*, not of this port:

- ~~**`buildPlaza` (milestone 8) runs on the organic branch too**~~ —
  **closed the same day**, see the milestone-8a record below. It was the most
  visible gap and the smallest change that closed one.
- **`removeWaterCrossings` (milestone 11)** does not run, so streets may still
  cross the channel. Milestone 12's own guards absorb most of it: a block whose
  inset centroid is wet is dropped, and a lot with *any* corner in the water is
  rejected (the reference's footprint test, not a centroid test).
- **`lanePass` (milestone 11)** does not run, so faces are coarser than the
  reference's would be from the same seed.

Milestones 8 and 11 will change what comes out of here without changing a line
of `blocks.rs` — and are the moment to re-run this milestone's mutation sweep
rather than trusting it. **Milestone 8a did both**, the same day: `blocks.rs`
is unchanged apart from its doc comments, and the plaza golden re-runs
`build_blocks` on every one of its own post-plaza graphs.

### Milestone 8a — the plaza: **done** (2026-08-24)

`buildPlaza` alone, reference lines **28941-28965**. Module
`cartalith-urban::plaza`; dependencies still `cartalith-rng` only. 12 tests,
17 golden scenarios, and the first mutation sweep in this subsystem to close
with **zero survivors**.

Taken out of milestone 8 rather than with it because the milestone's other two
functions (`buildRadialStreets`, `buildWaterway`) serve the *radial* planning
mode only, and `buildPlaza` runs on **both** branches of `generate()` (lines
31018 and 31024). Milestone 12 named it the highest-value remaining change and
it was: 60 lines of Rust, and it is the difference between a town with an open
market square and a town with a block platted over its own anchor.

**The stated range over-claimed by five lines at the end.** 28835-28970 runs
past `buildPlaza`'s close at **28965** and into the four-line harbour section
comment at 28967-28970, which milestone 7's correction had already assigned to
milestone 9. Milestone 8's range is **28835-28965**. **Seven ranges checked,
seven wrong** — and this one is the failure mode the rule was written for: an
end that is too *late* silently pulls in the next milestone's header.

#### Where it runs is part of the port

`generate()` calls it **between `buildPrimaries` and `grow`** on the organic
branch, not after growth. The three streets it lays are in the graph before the
epoch loop starts, so the town accretes *around* the square. Putting it after
`grow` would still produce a plaza and would produce a different town;
`cartalith_civ::urban_adapter::run_layout` calls it in the reference's place
and its module header says why.

#### Nothing new was built for it

The reuse milestone 12 found repeats exactly. `distPtSeg`, `V.norm`/`lerp`/
`rot90`, `polyCentroid` and `addStreet` were all built and golden-tested at
milestones 1-2; `stream`/`range` at milestone 1; `site.riverDist` at milestone
5. No new kernel, no new libm, and no new RNG semantics — `stream(seed,
'plaza')` is its own labelled substream taking exactly two draws, so adding
this stage **cannot** perturb any other milestone's sequence. Only the graph
changes, which is the point of it.

#### The mutation sweep, and why five survivors were closable here

20 mutations, 20 killed. Five survived the first pass, every one of them
milestone 7's *"exact tie on a continuous value"* class:

| survivor | closed by |
|---|---|
| side probe `20 → 21` | a river centreline laid **parallel to the street under test** |
| side probe `-20 → -21` | the same fixture at a 0.25 m offset |
| `>` → `>=` in the side ternary | the same fixture at an exact tie |
| `rot90()` → `-rot90()` | the same exact tie — see below |
| `d < bd` → `d <= bd` | two primaries exactly equidistant from the market |

Milestone 7 could not close its thirteen because every one rested on a
polyline distance or a raw `mulberry32` draw. These rested on **distance to a
centreline**, and `site.river` is a plain field this port may overwrite on a
real `build_site` site — so the probe gap becomes an *input*. Parallel is what
makes it a razor: along the edge normal the distance to a parallel line changes
metre for metre, so `c = 0` gives an exact tie and `c = 0.25` gives a 0.5 m gap,
which is inside the window a one-metre mutation of either probe moves the
answer through. **The general lesson: a survivor that rests on a continuous
comparison is closable exactly when one side of that comparison is a field the
fixture can set, rather than an output of an earlier stage.**

**Negating the edge normal is not the no-op it looks like.** `nl` is read twice
— to build the two probe points and as `nl * (side * wd)` — and away from a tie
the two negations cancel bit-exactly, which is why the mutation survived all 15
real towns. At an exact tie they do not: both arms of the ternary yield the
*same* `side`, so the product flips and the square opens the other way. Only
the tie fixture sees it, and it would have been recorded as a proved-dead
survivor without one.

#### Findings

1. **`buildPlaza` mutates `g` before its return value exists, and the two do
   not agree.** The three streets go in through `addStreet`, whose 11 m
   `attachPoint` snap binds a plaza corner to an existing node rather than
   creating one — up to **6.1 m** of movement across the fixture set. The
   reference builds `plaza.poly` and `plaza.center` from the **pre-snap**
   points regardless. That is why `buildBlocks` tests a *point* against each
   face rather than comparing polygons, and why a consumer must not assume the
   returned quad is the face the graph holds.
2. **The plaza's fourth side is not laid.** Three `addStreet` calls, not four:
   `p1 → p2` is the primary being widened and is already there. A port that
   lays four produces the same picture and a different graph.
3. **"Away from the river" is a statement about 20 m, not about the square.**
   The probe is a fixed 20 m either side of the street's midpoint and the
   square is up to 40 m wide, so on a curving channel the finished square's far
   edge can end up *nearer* the water than the rejected side's would have been
   — 0.05 m on the `river7` fixture. Reference behaviour, captured, and
   asserted, so that the next person to measure the square instead of the probe
   does not read it as a port bug.
4. **A landlocked site still resolves the ternary.** `riverDist` answers from
   the synthetic dummy centreline, so the branch is live rather than
   degenerate; three landlocked scenarios are in the golden for that reason.
5. **Every scenario produced exactly one flagged block.** Asserted as a
   property, not just as a golden count — it is the whole point of the
   milestone, and a change upstream that split the widened band into two faces
   would show up here first.

#### Corrections to later milestones

1. **Milestone 8's remaining range is 28835-28939** (`buildRadialStreets` at
   28844 and `buildWaterway` at 28928, plus the radial header comment at
   28835). `buildPlaza`'s 28941-28965 is done.
2. **Milestone 9's start of 28967 is right** — milestone 7 moved it there and
   this milestone confirms it: 28967 is the first line of the harbour block
   comment, 28966 is blank, and 28965 is `buildPlaza`'s close.
3. **Milestone 12's mutation sweep should be re-run again after milestone 11**,
   not treated as re-run by this one. This milestone changed `blocks.rs`'s
   *input*, and its own golden re-runs `build_blocks` on 17 post-plaza graphs
   — but `lanePass` and `removeWaterCrossings` will change it again.
4. **Milestone 16 must call `buildPlaza` between the primaries and `grow`** on
   the organic branch, and **after** `buildWall` on the radial one (line
   31018). The two positions differ and both are in `generate()`.
5. **`Plaza` is `blocks`' input and `plaza`'s output.** It is defined in
   `plaza.rs` and re-exported from `blocks.rs`; `build_blocks` takes
   `Option<&Plaza>` rather than `Option<Plaza>` now that it carries a polygon.

### Milestone 8 — radial (Venus) streets, waterway (lines 28835-28939, 2 functions)

`buildRadialStreets`, `buildWaterway`. The second planning mode, independent of
`grow`. Separable from milestone 7 and cheaper. **`buildPlaza` is done** —
milestone 8a above.

### Milestone 9 — water infrastructure (lines 28967-29159, 4 functions)

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

### Milestone 12 — blocks and parcels: **done**, recorded above

Built out of order on 2026-08-24 — see the full record in the completed-
milestone run above. The plan this stub carried was right on every count: the
line range 30193-30344 was correct at both ends, `buildBlocks` does skip
`extractFaces`' outer face on milestone 2's first-index-wins tie-break, and
`hashModel` was indeed unavailable, so the golden dumps state directly exactly
as milestone 2 did.

### Milestone 13 — districts and buildings (lines 30345-30710, 7 functions)

`assignDistricts`, `bmap`, `rectPoly`, `buildBuildings`, `_rectPts`,
`_peristyle`, `buildFaithSites`. Building grammars (burgage, venus-mixed),
the terrain-suitability building gate, churches and temples.

### Milestone 14 — amenities (lines 29160-29382, 5 functions)

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
  (the source repo's `urban-morphology/docs/07-culture-architecture.md` §3.10 —
  one of nine UME design documents under `Cartalith_RC`'s `urban-morphology/docs/`
  that are **not vendored in this repository**, unlike `docs/`) after a
  post-launch pass found them visually indistinguishable. Only `medieval` and
  `venus` are live; only those get ported.
- **`buildGridStreets` and the palimpsest planning mode** — likewise removed
  upstream, with no live caller.
- ~~**Wiring into `compute_civilisation()`, `cartalith-godot`, or the GUI.**~~
  **Superseded 2026-08-23** — see milestone 17a above. The rule was right for
  as long as there was nothing to render; it became wrong once seven
  milestones of engine had accumulated with no way for anyone to see, use or
  regression-check any of it, which is exactly what `PARITY_AUDIT.md` §3.4
  found. Note the boundary that *did* hold: the wiring is a bridge and a
  renderer, and `compute_civilisation()` still does not call this subsystem —
  a town is generated on demand, per settlement, never as a generation stage.

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

Milestone 7 added three, all of which cost it a restart, a sweep or a wrong
sentence:

- **A raster fixture must be in the units the code reads, and the code is the
  only place that says what those are.** Milestone 7's first fifteen
  raster-backed fixtures grew **nothing at all**, because `site.height` reads
  `opts.terrain.grid` raw and `site.slope` scales a per-metre central
  difference by 900, so a heightfield in metres of elevation makes every slope
  test in the engine reject. Read the consumer before building the input.
- **A threshold with two halves needs a fixture that satisfies both, and an
  invented shape usually satisfies neither.** The M-GRW-2a gate wants an
  interior hull filling 80% of the circuit *and* growth spilled outside it.
  Hand-drawn ellipses could not pass the first half at any size, and the
  obvious fix — the town's own convex hull — failed the second by enclosing
  everything. What works is the town's own hull *at an earlier epoch and
  restricted to a radius*, i.e. approximately what the function that would
  normally build it constructs.
- **A survivor is a claim you have not made yet.** Milestone 7's first sweep
  left 51; a second round split them into fixtures that close them (12 new
  scenarios, one of them **scanned** for an exterior count of exactly 10) and
  seven *provable* equivalences written as assertions — the clamp that cannot
  bind, the adjacency that cannot hold a dead edge, the angle wrap that its own
  following fold undoes, the twelve trig angles V8 and the platform agree on,
  the zero-area ring that cannot contain a node, the hull whose winding never
  varies, and the two fallbacks that are only assigned when they are not read.
  "Survived" and "cannot matter" are different reports; the second one is a
  test.

Milestone 6 added two more, both from its own sweep:

- **A dozen hand-picked rows cannot test a bit-twiddling port.** Its first sweep
  left **63 survivors inside `js_sin`/`js_cos`/`js_log` alone**, by a golden
  table built exactly the way `js_exp`'s and `js_hypot`'s were — twelve rows
  cover twelve paths through a branchy function, not its branches. The fix is
  four lines of golden: an FNV-1a **hash** over every result across tens of
  thousands of arguments, drawn by the reference's own `mulberry32` so both
  sides provably evaluate the same points, with the bands chosen to enter each
  branch on purpose (two of milestone 6's six trig bands exist only to reach
  `rem_pio2`'s second and third correction rounds, which no uniform band
  reaches). Milestones 8, 10 and 15 each need one of these; start there rather
  than with a table.
- **Take a pristine snapshot before the sweep writes anything.** Two of
  milestone 6's runners overlapped on one target directory; the first was killed
  mid-mutation, the second read the already-mutated file as its "original" and
  faithfully restored it to that, and the source shipped a live mutation that
  only the suite failing afterwards revealed. A per-edit `finally` restore is
  not enough, because it restores to whatever it read. Snapshot first, restore
  from the snapshot, re-run the suite as a **post-sweep baseline**, and refuse
  to start while a lock file exists. Milestone 4's stale-binary incident
  corrupts the *report*; this one corrupts the *source*.

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
