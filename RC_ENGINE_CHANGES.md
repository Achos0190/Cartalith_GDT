# HTML simulation & generation changes to carry into the native port

**What this is.** A porting specification for every change to the *simulation* and
*generation* layers of the HTML app (`Cartalith_RC`) made after the version this
repository froze as its reference. It names the function and constant each change
lives in, states why each number is the number it is, and says how the HTML side
verified it — so the Rust implementation can be checked against the same evidence
rather than against a paraphrase.

**What this is not.** It makes **no claim about what the port has already built.**
Port status lives in `cartalith-native/docs/STATUS.md` and, ultimately, in the
code — this repository's own `CLAUDE.md` is explicit that a document's claim about
itself is a claim, not evidence. Treat every row below as "this is what the HTML
does"; check separately whether the port already does it.

## Span, and a fork you must not miss

| | |
|---|---|
| Reference frozen here | `reference/Cartalith Gen1 v2.10.html` (plus `Cartalith Gen1 v2.11.html` at this repo's root) |
| Covered by this document | **v2.11 → v2.40** |

**The HTML source has two lines, and they diverged at v2.22.** This matters more
than anything else in this document:

- **Mainline** — `Cartalith Gen1 v2.22.html` is the newest mainline file. It carries
  everything up to and including v2.22.
- **DCC line** — `Cartalith v2.23 … v2.40 DCC test.html`. v2.23 duplicated v2.22 to
  carry the port's shell theme; **v2.24 onward exist only on this line.**

So every engine change from v2.25 on — the river carve rework, the blur path, the
routing cost model — **is present only in `Cartalith v2.34 DCC test.html`**, not in
any `Cartalith Gen1 v*.html`. The DCC files are deliberately named without `Gen1`
because `tests/run.sh` globs `Cartalith Gen1 v*.html` and takes the last by version
sort. If you diff against a `Gen1` file you will silently miss sections 2, 3, 4 and 6.

---

## 1. Generation pipeline: order and parameters

### 1.1 Erosion passes became generation parameters (v2.17)

`state.passes = { velocity:false, glacial:false, coastal:false, hillslope:false }`
(HTML line ~2842). Velocity erosion, glacial carving, coastal processes and
hillslope diffusion used to be buttons pressed *after* generation, so an eroded
world could not be reproduced from its seed. `generate()` now runs them.

- **The split is the whole design.** `*Pass()` mutates the field; the button wrapper
  is `*Pass()` **plus** the interactive tail `computeFlow(true); refreshClimate();
  renderNow()`. `generate()` calls the same `*Pass()` the button does. Do **not**
  give the generation path its own copy of a kernel's parameters — each pass reads
  its existing sliders (`state.velo`, `state.glacial`, `state.coastal`,
  `state.erosion`).
- **`eroSettle(pre)` is the physics half of `eroFinish(pre)`** — isostatic rebound,
  exhumation hardening, `enforceRiverChannels`. `eroFinish = eroSettle + tail`.
- **Fixed order: velocity → glacial → coastal → hillslope.** Fixed order is what
  makes a saved pass-set reproduce its world. Do not make it configurable without
  replacing that guarantee.
- **Staged after climate, before `carveRiverValleys()`** — velocity reads
  `rainField`, glacial reads `tempField`, coastal reads `flowField`.
  `runGenerationPasses()` returns whether anything ran, so `generate()` re-derives
  once in its own order (`refreshClimate()` then `computeFlow(true)`). The button
  path re-derives the other way round; that asymmetry is pre-existing and deliberate.
- **Porting trap:** `coastalPass()` swaps in a gravity-scaled copy of
  `state.coastal` for the GPU path and must restore it in a `finally`. At g = 1 a
  broken restore is invisible — **test at g ≠ 1.**

### 1.2 Physical crater model (v2.22)

`state.crater = { count:100, age:0.50, physical:false, ratePerMkm2Myr:2.60,
surfaceAgeMyr:500 }` (line ~2828). Default **off**. The legacy path picks an
absolute count and three hardcoded size buckets, so a 50 km region and a 40 000 km
world get the same hundred impacts.

| Constant | Value | Meaning |
|---|---|---|
| `CRATER_SFD_EXPONENT` | `1.8` | cumulative size-frequency `N(>D) ∝ D^−b` |
| `CRATER_TAU_MYR_PER_KM` | `20` | obliteration timescale `τ ∝ D` |
| `state.crater.ratePerMkm2Myr` | `2.60` | production rate per Mkm² per Myr |
| `state.crater.surfaceAgeMyr` | `500` | surface exposure age |

- Count = rate × real area × surface age. Size from a bounded Pareto by inverse
  transform. Wear from exposure time against `τ`.
- **The count SATURATES with age — do not "fix" that.** Production is linear in age
  while obliteration removes in proportion to the standing population, so an old
  surface settles at fewer, larger, more degraded craters. Measured 90 / 102 / 78 at
  500 / 3000 / 6000 Myr, median diameter 1.00 → 1.53 km. That is the terrestrial record.
- **Anchor the rate on the SURVIVING population, never on production.** A first cut
  anchored production at 100 and stamped only 19. The HTML's headless test pins the
  production identity (`rate × area × age`), not a tuned number, so re-anchoring
  cannot silently invalidate it.
- The stamping ceiling raises the smallest diameter kept (closed-form inverse of the
  size law) — never a random thinning.
- The checkbox **chooses which pair `stampCraters` reads**; `count`/`age` stay saved
  and are disabled, not cleared.

Verification: `tests/perf/probe_craters.js`.

---

## 2. Hydrology: the river carve (DCC line only)

This is the largest generation change in the span, and it lands in three steps.

### 2.1 Rivers are carved into terrain again (v2.29)

- **`state.viz.riverWays` is an EITHER/OR, not an addition.** The per-pixel branch is
  gated `!(state.viz && state.viz.riverWays)`, so the stroked vector line being ON
  means the terrain-blended raster river is OFF. It shipped `true` for fresh worlds.
  **Both now default `false`.** A save without the field has always loaded `false`,
  which is why an older project looked right and a new world did not.
- **`CARVE_STRENGTH_K = 8`** (line 9703) multiplies `state.stream.k` **inside
  `carveRiverValleys()` and nowhere else** — the manual Stream-power button,
  `evolveCoupled` and the erosion worker keep the raw slider. It is calibrated, not
  chosen: 8× is where the pass reaches **3.08×** the un-carved surface's Laplacian
  energy, against **3.16×** measured from the `elevation_foundation_v0.015` lineage.
  16× overshoots to 4.99× and costs 14% of the polylines.
- **Raising `P.iters` buys the same energy at ~2× the time.** Strength is the cheap lever.
- **Three structural hypotheses were measured and REFUTED — do not re-chase them:**
  frozen flow routing, MFD-vs-D8 drainage area, and restoring `stream.uplift`.
  Re-routing and D8 both come out *smoother*; uplift builds ridges (84 928 cells
  raised) while cutting the traced network from 899 polylines to 696.
- **Widening the polyline carve is refuted — the stamp is the inner CHANNEL.**
  `enforceChannelDescent` stamps a disc of radius `halfW` around every point of ~840
  polylines, so area grows as `halfW²`: ×1.5 carves 18% of the map, ×3 carves **96%**,
  and mean valley relief only moves +16% / +57%. The erosion pass is what broadens a
  valley.

### 2.2 The carve follows a river, not a receiver chain (v2.30)

`carveChannelPath(poly, maxOrder, slopeN, halfW, W, seed)` (line 9668) is now the one
place a receiver chain is converted into carvable geometry.

| Constant | Value |
|---|---|
| `CARVE_SINU_K` | `8` |
| `CARVE_RESAMPLE_MAX_STEP` | `0.5` |
| `CARVE_GRADIENT_K` | `2` |
| `CARVE_MEANDER_CTRL_PER_WAVE` | `8` |
| `CHANNEL_DROP_PER_CELL` | `0.0006` (line 9608) |
| `RIVER_SINU_SLOPE_K` | `0.4` (line 5347) |

- **`traceRiverPolylines` returns a receiver chain, and a chain is not drawable OR
  carvable geometry.** `drawRiverWays` already ran `rdpSimplify` →
  `catmullRomSample` → `riverSinuosity`; `carveRiverValleys` handed the raw chain
  straight to `enforceChannelDescent`.
- **`enforceChannelDescent` stamps a disc per point and NEVER interpolates.** So the
  resample must be finer than the channel. It must *not* be `drawRiverWays`' `GW/360`
  — at 2.8 cells the carve comes out as a dotted line of pits and measures *worse*
  than the raw chain (4 068 cells vs 6 960). **Assert the invariant directly: no gap
  between consecutive points wider than `halfW`.** Coverage of the drainage network
  by trench went **64.3% → 97.1%**.
- **`enforceChannelDescent`'s `drop` is PER POINT, so the resample step silently sets
  the channel gradient.** Halving the step doubled every river's enforced descent,
  costing 29.2% of the traced network against v2.29's 13.7%. Hence
  `CHANNEL_DROP_PER_CELL` is named, the carve scales it by its own step, and
  `CARVE_GRADIENT_K=2` then re-applies the steeper gradient **deliberately**. The
  Sculpt editor's hand-drawn River stamp keeps the plain default — a person places
  those points.
- **`riverSinuAmp` was a no-op and had been for many versions.** It divides by
  `1+6*slopeN` as if `slopeN` were a 0..1 grade, but `net.slope` is `hypot(grad)*W`,
  median 1.74 at GW=1024 — so median amplitude was **0.081 cells**. Fixed at source
  via `RIVER_SINU_SLOPE_K`, not by adding a second function.
- **`fbm` sampled at the carving step is jitter, not a meander.** Its top octaves vary
  fully between points 0.4 cells apart, separating neighbours by up to 1.74 cells and
  re-opening the gaps the resample exists to close. Perturb
  `CARVE_MEANDER_CTRL_PER_WAVE` control points per wavelength, then spline through
  them at the carving step.
- **`CARVE_SINU_K = 8` has a hard ceiling.** At 8× **86.5%** of the trench is still a
  real drainage line after the closing `computeFlow()`; at 20× that falls to **73.4%**
  and the trench wanders off the water. A real meander belt comes from lateral
  migration across a floodplain, not from displacing a drainage path.
- **The network thins, and that is understood, not a bug:** a flatter carved floor
  raises `channelThreshold`, so marginal headwaters leave the detected mask (−24.1% of
  traced extent). **Measure extent, never polyline count** — a carve that joins two
  runs into one changes the count without losing any river.
- **D8 is not the defect.** `buildRiverNetwork` already routes by the continuous
  aspect (a single-receiver projection of D∞); what remains is only that the receiver
  is one of eight lattice neighbours, which a spline removes. Velocity erosion
  (1.076 → 1.108 relief energy for ~1.8× generate time) and warp 0.45→1.00 (1.097,
  thinner network) were both measured and are poor value.

### 2.3 Channel width: symbol vs. real (v2.25)

- **`buildRiverNetwork` now RETURNS its channel half-width (`halfw`), and that is the
  only width model.** It always computed `halfW` — hydraulic geometry, real-km-aware
  since v2.07 — then threw it away after stamping `intensity`/`depth`/`omax`, so the
  renderer had nothing to compare its pen against. **Never re-derive `halfW` at a
  renderer.**
- **The stroke is `max(symbol, real)` — a FLOOR.** The cartographic symbol still wins
  at world scale: 0% of polylines floored at zoom 1, 1% at zk=8, 100% at zk=32.
- **A cell-granular water test against a sub-cell shoreline shatters the line.**
  `splitRiverPolylines`' lake predicate read `_waterBody[i]===2` per cell while lake
  shorelines draw sub-cell, and every run left under 2 points is dropped: **517
  polylines / 5 910 points → 393 / 3 931**. It now tests whether the pooled surface
  genuinely stands above the terrain under the point
  (`_lakeFill[i] − sampleArr(field,p.x,p.y) > 0.004`). Recovers 404 / 4 163.

---

## 3. `gaussBlur`: the CPU path is the fast path (v2.32, DCC line only)

`const GAUSS_BLUR_GPU = false;` (line 3105).

- **`gaussBlur` and `GPU.blurArr` are one algorithm, not two** — three separable
  box-blur passes at `pr = round(r/1.6)`, agreeing to 1.2e-7. They are **not one
  complexity**: `boxH`/`boxV` carry a running sum (O(N), radius-free — 36 ms at 1024
  whether `pr` is 4 or 40) while the shader scans the whole `2·pr+1` kernel (O(N·pr))
  and then blocks on a synchronous `readPixels`.
- Measured **2.1× – 21.7×** in the CPU's favour at every size and radius, **widening
  with radius**. That shape is algorithmic; more cores move the constant, not the
  exponent.
- **`generate()` 5 919 → 3 894 ms at 1024px (−34%)**; 2 057 → 1 631 at 512px;
  20 153 → 13 758 at 2048px. Flexure 599 → 49 ms. `readPixels` was **24.2%** of a
  1024px generate.
- **Only the five full-grid callers ever took the GPU route**, and they are the
  expensive ones: `stressField`, `shearField`, the flexural blur at `blurR*3`, `baseField`,
  and `isostaticRebound` inside `carveRiverValleys()`.
- **A startup timing calibration was considered and rejected.** The blur feeds
  `stressField` and flexure, so picking the path by measurement would make the terrain
  depend on how busy the machine was at load — one seed, two worlds, on one computer.
  It is a fixed flag.
- **The re-baseline is quantified:** with WebGL2 up, the world moves by max 2.31e-4 on
  `field` (mean 8.03e-8, below float32 resolution). Elevation scale is
  `metersPerUnit() = state.peakM/(1−seaLevel)` — **not** a fixed span; at defaults
  6 897 m/unit, so the worst cell moves ≈ 1.6 m.

**Port relevance.** If the Rust port routes any of those five blurs through `wgpu`,
this is the measurement that says re-check it. The HTML's conclusion is specific to a
kernel-scanning shader against a running-sum CPU pass — a compute shader with a
prefix-sum would not be the same comparison. **Do not "restore" a GPU route without
re-running the probe on the hardware you are claiming it for**
(`tests/perf/probe_blur.js`; it needs a real browser, since the headless suite has no
WebGL2 at all).

---

## 4. LOD tile passes must know how many pixels a coarse cell spans (v2.25)

`tileShadeExag(bounds, W)` (line 12682) scales hillshade exaggeration by
`(W−1)/bounds.w`, clamped at 1 so it never *reduces* exaggeration; **bounds omitted ⇒
the previous value exactly.**

- All three tile renderers hillshaded with a bare `state.exag` while the main map uses
  `state.exag/s`, and `renderBiomeTileRGBA` normalises its *material* slope by
  `cx`/`cy` one line later — the shading term was the lone un-normalised pass, so
  relief flattened exactly where the LOD viewer exists to show it. Shaded-pixel share
  at z=4: **22.4% → 36.1%**; z=0 identical.
- **The native port found the same gap independently at `lod_bridge.rs:420`** — worth
  checking whether that fix and this one agree in form as well as in effect.

**Two plausible LOD fixes were refuted by their own measurement — do not re-chase:**
raising `lodDetailFreqK` measures 3.7× more Laplacian energy, but the octaves land
**above the tile's Nyquist limit** from z=4 up (that is aliasing, not detail); and
scaling `burnChannels`' `widthK` costs **17× the time for a 0.3-point change in burned
area**, because `mag` is bilinearly interpolated coarse flow, so the `mag ≥ thresh`
band already scales with zoom on its own (1.35% burned at z=0 → 7.48% at z=8) and
`widthK` only feathers the rim.

---

## 5. Civilisation-layer simulation

### 5.1 Road connectivity had always answered "no" (v2.18)

`_civRoadComponents()` / `_civRoadConnected()` returned "not connected" for **every**
pair from v1.33 until v2.18, so long-range overland food import was never reachable.
Two stacked defects:

- **`state.ways` is never assigned anywhere in the file.** The live array is `civWays`.
- **Way points are not uniformly shaped**: `_civMstRoutes` / `_civHierarchicalNetwork`
  emit `[x,y]` **arrays**, other builders `{x,y}` **objects**. Reading `.x` gives
  `undefined` → `NaN` → every comparison false → a pass that silently does nothing.
- **Neither threw.** "Not connected" is an ordinary answer, which is why it survived
  versions. **Any predicate whose false branch is unremarkable needs a test that a TRUE
  case really is true** — here, that two settlements joined by a real road read as
  connected (`tests/perf/probe_roadconnect.js`).
- Sea lanes are excluded: this answers "is there a ROAD".

### 5.2 Military manpower (v2.19)

`civManpowerModel(inp)` (line 17382) is **pure and is the whole model**;
`_civFactionManpowerAll()` gathers the real inputs. It was ported from the owner's
specification, which this repository already carries verbatim in
`MILITARY_MANPOWER_SCOPE.md` §1 — **read that before touching any `MANPOWER_*`
constant.**

- **The two worked examples in that specification are the calibration target and both
  reproduce to the unit.** They are the only external check this model has. Do not
  retune a constant without re-running `tests/perf/probe_manpower.js`.
- **Four outputs, never one:** standing / field / emergency / duration. The
  force-by-duration ladder is what makes them comparable.
- **Technology is not the driver** — it sets the labour ratio only. The era is an
  *output* of the labour ratio split by state capacity, never a lookup on the ag-tech key.
- **Era bands are shares of the CITIZEN population.** Both bases are shown; nothing is
  clamped into a band. `MANPOWER_CITIZEN_MODERNISATION` is written as
  `CEILING − min(share)` on purpose, so editing the lowest row without editing it
  fails loudly.
- **Unknown government keys read as `chiefdom` in BOTH tables, for opposite reasons:**
  it denies an unclassifiable state an imperial treasury, and it gives it a high
  citizen fraction, which makes a share of that body smaller and so cannot flatter it
  into a band.
- **`MANPOWER_ROAD_REFERENCE = 10`, not the ~40 a Roman road inventory suggests** —
  this app's way network is inter-settlement trunk roads only, so it is not comparable
  to a road inventory.
- **Derived and stored nowhere** — nothing manpower-shaped reaches the save format.
- The state-capacity splits inside each labour-ratio band are **fitted** to the eight
  faction assignments the specification publishes, not stated by it.

### 5.3 Landmasses as named entities (v2.20)

`buildLandmassIndex` is pure; `currentLandmasses()` is the cached live accessor.
Detection was never the gap — `buildLandmassQuality` already labelled and ranked every
component. This adds identity: a name, a stable key, an extent.

- **`landmassKind` is a share of the world's own LAND, never km².** An archipelago
  world honestly has no continent; do not add a "promote the biggest" fallback.
- **`landmassKey` is POSITIONAL** (centroid quantised to 8 cells + the seed), never the
  component index — those renumber on any coastline change, so an index-keyed rename
  would reshuffle after a sculpt edit. `state.landmassNames` stores only user renames.
- **World mode needs the CIRCULAR centroid.** A landmass straddling the antimeridian
  has cells at x≈0 and x≈W−1; a plain mean puts its label in mid-ocean. The bbox stays
  raw and carries `wraps`.
- **No outline is traced** — the coastline is already what the renderer draws.
- Map layer `state.viz.landmassLabels`, default off, drawn beneath user labels and
  settlements.

### 5.4 Multi-good supply matching (v2.21)

`_civResourceSupply(p)` answers "where would each import actually come from". The
food-only version of this match already existed; `_civGoodReach` classified every other
export and nothing read it.

- **Reuses the existing machinery — do not add a second distance law.**
  `_civFoodMode` / `_civFoodConnected` / `_civFoodDeliverable` are the mode, the road
  gate and the curve.
- **The only addition is VALUE DENSITY:** `_civGoodDeliverable` divides the distance by
  a class multiplier (**bulk 1, general 3, luxury 20**) and calls the same curve — the
  same law with a longer half-distance. A bulk good is therefore bit-identical to the
  food answer; asserted.
- **An unlisted good falls to `general`**, never to bulk, so a new resource key behaves
  sensibly before anyone classifies it.
- **It depends on §5.1.** Before that fix `_civFoodConnected` refused every overland
  supplier past 50 km, so this match would have reported almost nothing and looked
  deliberate.
- `_civTradeNetwork()` is **one cached pass** — calling `_civPlaceTrade` per candidate
  per good is quadratic over a 235-settlement world.
- Display-only: it feeds no population or economy figure, so the acyclic rule (§7.4)
  is intact.

### 5.5 Corridor and travel-cost views (v2.13) — no new computation

`buildRouteCorridors` and `buildTravelCost` were **already computed and already
consumed** as internal scoring inputs — only never drawn. `currentTravelCost()` mirrors
`currentRouteCorridors()`'s existing cache idiom. **Before writing a new field, check
whether the one you want is already being computed as an internal scoring input.**

Debug-view normalisers here are **fixed, not per-world**: the same colour must mean the
same value across worlds — unlike the self-calibrating scales simulation uses (§7.1).

---

## 6. Land routing became a time model (DCC line only)

### 6.1 Land routing costs TIME (v2.33)

`_civTravelHours(dfld, W, H, sea, usageCount, opts)` (line 23916) is **hours per
routing cell on level ground**, and is now the one land model.

- **Three land cost functions were live**, and which you got depended on how the road
  was made: the auto network had the full terrain model, the Route tool a partial one,
  and **the Way tool, village tracks and the sea-lane MST's land branch had slope and
  nothing else.** `buildTravelCost` stays for the debug view and `_civAutoPolity` — a
  control flood is not a road.
- **`1 + 50·sl²` took `sl` in field units per cell**, so one hillside scored
  differently at 512 and 2048 and no number was ever comparable with the Journey
  Planner's hours.
- **Slope belongs on the EDGE, not the cell** — it is the only place a direction
  exists. `roadDijkstra` gained an optional trailing `edgeCost(i,j,dx,dy)` hook;
  omitting it takes the identical arithmetic path, bit-identical by construction.
  Tobler on **signed rise/run** (**not** degrees, **not** percent), **averaged over
  both directions**, because a road is bidirectional and a Prim MST is undirected.
- **The per-cell array stays the carrier, and that is load-bearing:** settlement
  gravity, the ×0.25 existing-way discount (`_CIV_EXISTING_WAY_DISCOUNT = 0.25`, line
  24740) and the usage-count reuse pass all multiply `cost[i]` in place and keep
  working untouched — a multiplier on hours is a speed multiplier. **Any future term
  must preserve that.**
- **Fords and bridges are additive HOURS**, not added cost. A crossing is a wait.
- **It changed almost nothing, and the reason is measured:** p50 grade on the routing
  grid is **1.20%**, p90 **5.77%**, p99 **14.66%** — Tobler gives ×1.001, ×1.043, ×1.42
  there, while the non-slope terms span ×0.55–×1.8. **Slope was never binding, for
  either formula.** Refuted along the way: the routing grid is *not* washing grades out
  (2.08 km cells reproduce the full-res distribution).
- **`'land'` vs `'mixed'` is not a fair model comparison** — mixed may cross water, so
  the domains differ. The honest comparison is same-mode across versions.

### 6.2 The land surface multiplier is a SPEED, from the Planner's own table (v2.34)

`_civSurfaceSpeed(terrIdx, biomeKey)` (line 23818) reads `JP_TERRAIN.land` through
`buildCartTerrain()`'s classification — the exact classifier `_jpDeriveStages` runs.

```
JP_TERRAIN.land = {
  "Paved Road":1.50, "Dirt Track":1.00, "Open Plains":0.95, "Forest Path":0.75,
  "Hills":0.70, "Rocky Terrain":0.50, "Mountain Pass":0.65, "Mountain Trails":0.45,
  "Swamp / Marsh":0.40, "Desert Hardpack":0.80, "Deep Sand":0.50,
  "Snow / Ice":0.55, "Ruins / Debris":0.50 }
```

- **A multiplier that barely varies cannot steer anything, and the one it replaced was
  worse than flat — it was INVERTED.** Measured over the routing grid, the old
  `_civBiomeFriction` charged **Open Plains 1.418 and Rocky Terrain 1.373**. It keys on
  climate vegetation, so a bare rocky mountain reads cheap and a forested plain reads
  dear. **Check the spread of any new cost term before trusting it.**
- **`CART_TERRAINS` and `JP_TERRAIN.land` are the same thirteen names**, so the mapping
  is 1:1 by name with nothing invented between. `hours = flatHours / speed` composes
  with the per-edge Tobler term exactly as `jpCalcLand` composes its own answer, so the
  router minimises literally the quantity the Planner reports.
- **The grade double-count is real, small, and measured:** Tobler charges ×1.001–×1.068
  on the slope-derived classes against the ×1.43–×2.22 the table asks. Grade is the rise
  BETWEEN two routing cells; the class is the roughness WITHIN one, and a routing cell
  aggregates several full-res cells.
- **`buildCartTerrain` can never emit four of its thirteen classes**, and `'Forest Path'`
  is the one *natural* omission — so the Planner calls flat woodland "Open Plains", 0.95,
  faster than Hills. A branch fixing that was built and **reverted**: `'Forest Path'` is
  in `JP_WHEEL_BLOCKED`, so emitting it hard-blocks every cart and wagon crossing
  woodland — a claim that set makes about a hand-painted narrow cut path, not ordinary
  forest.
- **So the router prices forest itself, from the same table's `Forest Path` entry, by
  `min` — the slowest surface binds, NEVER a product.** `0.50 × 0.75` is a number
  nothing supports, and a rocky forested slope is slow because of the rock. It moves
  only the 3.5–15.9% of land that is flat AND wooded.
- **Two tests for one question, again:** the old `dfld < sea+0.06 && flow > thresh*8`
  swamp proxy is gone — `'Swamp / Marsh'` is already a class, at 0.40.
- **Measure the objective you changed, not the one you changed last time.** Scored on
  the Planner's own composite, both sides against the identical unchanged terrain grid:
  **−3.8% / −4.2% / −13.9% / −21.0%** hours across seeds and modes, 61 pairs quicker to
  11 slower, **with path km and cumulative climb falling too**.

Verification: `tests/perf/probe_landsurface.js` (19 assertions).

---

## 6b. Tectonic boundary tracing and the orogenic belt (v2.35 / v2.36, DCC line only)

Both versions are reachable from terrain ONLY through `buildOrogenyField`, which `generate()` calls
solely under `state.tect.tectonicGraph` (default `false`); the other consumer is a debug overlay. So
both are bit-identical at defaults — but a port that implements `tectonicGraph` needs all of it.

### 6b.1 The junction predicate is the CROSSING NUMBER (v2.35)

`traceBoundaries` cut a chain wherever `deg !== 2`, with `deg` the **raw count of 8-neighbours**.
`thinMask` is Zhang-Suen, which yields an **8-connected** skeleton, so an ordinary diagonal staircase
gives an interior cell 3 or 4 neighbours in only 2 groups — and every such cell was treated as a
junction.

- The correct test is the **crossing number**: the count of `0→1` transitions around the 8-ring, i.e.
  the number of distinct neighbour GROUPS. Endpoint 1, interior 2, junction ≥3. On the staircase cell
  the ring reads `[0,0,0,1,1,0,1,1]` — raw count 4, crossing number 2.
- **`thinMask` already computes this quantity as its own `A`**, one function above, for its own
  thinning test. It was never a new idea, only an unused one.
- Measured at 512px / 14 plates: **1040 "junctions", only 77 of them at a real plate triple point**,
  against **23** for the crossing number — and a planar 14-plate graph has ~2n−4 ≈ 24. Longest
  collision margin 88.9 → 161.0 km (seed 12345), 91.5 → 288.0 (31337), 57.7 → 226.4 (4242).
- **The WALK must change with the predicate.** Under the crossing number an interior cell can still
  carry 4 neighbours in 2 groups, so picking `nbrs[0]` can step back into the group just left and
  ping-pong forever. Prefer a neighbour **not 8-adjacent to the previous cell**, orthogonal first.
- **And stepping past a same-group diagonal leaves it UNVISITED**, so a later pass walks a second
  polyline retracing the chain. v2.35 shipped that defect: 2.86 points per distinct skeleton cell
  against 1.45 before, with the loop pass emitting 81 of 121 polylines. **On a 1024 world it spun a
  walk to the safety cap and produced a 524 290-point polyline — `buildOrogenyField` took 93 885 ms,
  against 6 307 before and 1 878 after the fix.** Claim the skipped cell during the walk. Keep the
  cap proportionate (`2*(W+H)`, not `W*H`): a malformed skeleton should fail fast, not spin.

### 6b.2 The collision belt is a stack of thrust sheets (v2.36)

`buildOrogenyField`'s collision branch was one Gaussian ridge plus two satellites; it is now a stack
of 4 sheets across `1.55 * halfBelt`, each tapering toward the foreland.

- **Two things make it a BELT rather than four parallel lines or one merged hump**: ONE
  long-wavelength bend is shared by every sheet, so the belt curves as a unit; and each sheet's own
  deviation is a fraction of the **SPACING**, never of the belt width — which is what stops a sheet
  closing the gap to its neighbour.
- **Calibrate on a PER-STATION crest count, never a mean cross-section.** Each sheet's crest wanders
  independently, so a mean smears them back into one hump and undercounts ridges.
- **Whether a cross-section reads as separate ranges is set by sheet spacing IN CELLS**, and that is
  a resolution limit, not a tuning knob: 0.86–0.96 of stations carry the full stack at 35.3 cells,
  0.50–0.57 at 17.7, 0.07–0.18 at 8.8. **Five hypotheses were tested against it and refuted** — the
  post-blur, the fold-ripple period, the shared crest jitter (removing it is WORSE), the along-strike
  vigor term, and per-group walk starts (a measured no-op).
- **The fold ripple must stay incommensurate with the stack.** At ~one sheet sigma it manufactures
  false crests on each sheet's flank; `0.80 * spacing` preserves the original period-to-sigma ratio.
- **`orogenyWidthScaleK(mapWidthKm)`** gives the belt a fixed REAL width instead of a fixed fraction
  of the grid — the sixth sibling of the `terrainDetailK` family, keyed on `mapWidthKm` alone. It
  needs BOTH a cap (stamp cost grows as radius²) and a **FLOOR**: at 19.53 km/cell the belt fell to
  2.14 cells and structured orogeny rendered nothing. 8 cells is the floor.
- **ASPECT IS BOUND BY MARGIN LENGTH, NOT BELT WIDTH.** Anchoring the belt at a real orogen width
  (125 km half-width) fixes sheet resolution outright (0.57 → 0.93 of stations) and was **reverted**:
  it draws 550 km of belt against a 161 km margin — 69% of an 800 km map, aspect 0.29. At a default
  extent a Himalaya's proportions and its internal structure are not simultaneously available; a real
  Himalaya is 2400 km long, three times the map.

## 6c. The carve must never see a wrapped receiver chain (v2.37, DCC line only)

**This is the highest-value row in this document for a port, because the port will reimplement both
halves and can reintroduce it exactly.**

`buildRiverNetwork` picks receivers through `nx=((nx%W)+W)%W` in world mode, so a river crossing the
antimeridian has consecutive points at `x≈W−0.5` then `x≈0.5`. `splitRiverPolylines` exists for this
and was applied at the render and export sites. **The carve was exempted, in a comment**:

> *"The carve path is unaffected: enforceChannelDescent stamps a disc per POINT and never interpolates
> between them, which is why this only ever showed up as a rendering artifact."*

True when written. **v2.30 destroyed the premise** by inserting `carveChannelPath` between the trace
and the stamp, which resamples through `catmullRomSample` at `step = min(0.5, halfW*0.5)` and fills
the seam jump in. `enforceChannelDescent`'s monotone descent ladder then bottoms out at
`floorLim = sea − 0.06` and **holds below sea level for the rest of the traverse** — there is no
land/sea test. Rendered, that strip paints as water.

- Measured on a 20 000 km world: **28 of 11 802 chains wrapped, carrying 22.7% of the entire carve's
  points from 0.23% of the geometry**; 24 830 cells pinned at the floor in 30 horizontal runs of 100+
  cells, longest 355, against **zero** vertical runs of 100+.
- **The fix is one line**: pass the traced chain through `splitRiverPolylines` at the carve site too.
  No skip predicate — a lake reach is real hydrology the carve should cut.
- Channel cells **59 481 → 71 654 (+20%)**: the bogus trenches were drowning real rivers.
- **Region mode is byte-identical** (no receiver can wrap). World mode is a deliberate re-baseline,
  and not nil at the default extent.

**Three porting lessons, each of which cost this codebase real time:**

1. **A comment recording why something is safe is load-bearing, and it can expire.** When you add
   interpolation to a path, grep for every comment claiming that path does not interpolate.
2. **Severity is not the count of bad inputs, it is how much output each one produces.** 0.23% of the
   geometry produced 22.7% of the work. A count of wrapping polylines says "negligible".
3. **Match the detector to what the bug writes.** The carve writes a FLAT ABSOLUTE floor value;
   searching for cells *depressed relative to neighbours* cannot find a flat floor.

**And the reason it survived six versions: every harness ran in the one mode that cannot reproduce
it.** The region/world flag gates the wrap, the carve probe sets region, the hash battery never sets
the flag at all, and the battery seed has zero seam-crossing rivers even in world mode. **A port's
parity suite must include a world-mode scenario on a seed that genuinely wraps**, and should assert
the mechanism (no carve path spans more than half the map) as well as the symptom.

One adjacent bug found with it: `gridH()` reads the module global `state.world`, so anything
computing `GH` must assign the extent FIRST. The setup gate had the two lines the wrong way round and
built world maps with the region aspect.

## 6d. What the road-network build actually costs (v2.38 profile, DCC line only)

Not a change to port — a **measurement of the design in §6**, taken because the owner
reported Auto-populate as broken when it was merely silent. The port will implement
`_civHierarchicalNetwork` and should know the shape of its cost before copying it.

Profiled on a 1024 px World map, 20 000 km wide, 43 settlements:

| stage | time | calls |
|---|---|---|
| **`roadDijkstra`** | **8 593 ms (77%)** | **326** |
| `_civHierarchicalNetwork` (contains the above) | 8 773 ms | 4 |
| `currentSettlementSuitability` | 1 854 ms | 3 |
| `_civApplyFoodShedCeilings` | 217 ms | 1 |
| `findSettlementSeeds` | 24 ms | 2 |
| `_civNetworkMetrics` | 7 ms | 3 |
| **total (`_civIterativeAutoWorld(3)`)** | **11 190 ms** | |

Three facts worth carrying:

- **`_civHierarchicalNetwork` runs `2 x settlements` full-grid Dijkstras** — one per
  settlement to build the all-pairs matrix the Prim MST consumes, then a second full
  set for the minimum-degree pass over the reuse-discounted cost grid. It is rebuilt
  **from scratch four times**: the three `_civIterativeAutoWorld` passes plus the
  crossroads re-route. The two intermediate networks are discarded; only
  `_civNetworkMetrics` (7 ms) reads them, to promote/demote settlement tiers.
- **The cost does not scale with world resolution.** `_civRoutingGrid` is
  `Math.min(GW,384)`, so the routing grid is 384x192 at every world size — a 4K world
  costs what a 512 one does. **The driver is settlement count, not the map.** A port
  that instead routes on the full grid will be dramatically slower than the HTML at
  high resolution, for no gain the HTML is getting.
- **§6's per-edge `edgeCost` hook costs 44%** of each Dijkstra — 26 ms with it against
  18 ms with it null, same grid, same source. That is the price of the Tobler slope
  model being on the edge rather than the cell, and it is worth paying; but a port in
  a compiled language should inline it rather than reproduce the callback indirection.

**If the port wants this faster than the HTML**, the two openings are: derive the
all-pairs matrix from **one multi-source Voronoi Dijkstra** rather than n
per-settlement ones, and let the two intermediate passes settle tiers on a cheaper
distance proxy. Both **change the generated road network**, so they are a deliberate
re-baseline, not a free optimisation — the HTML declined to bundle them into a fix
about feedback, and the port should make the same choice consciously.

---

## 6e. The LOD colour path has no river, and that is structural (v2.39, DCC line only)

A porting constraint, not just a bug report: **the HTML's river water colour is produced in exactly
one place and the tiled-LOD path cannot reach it.**

- `waterShade` (Beer-Lambert depth water, `RIVER_KD`) has **one call site in the entire file**,
  inside `surfaceColor`, gated
  `state.showRivers && _riverNet && !(state.viz&&state.viz.riverWays)`.
- The LOD colorizer is `renderBiomeTileRGBA`, and it calls `landColorCore` **directly** — it never
  goes through the `surfaceColor` wrapper. Verified by nulling `_riverNet` and re-colorizing one
  tile: **byte-identical, FNV `262842011` both ways.**
- `renderHeightTileRGBA` (Relief tiles) and `bakePixel` (the PNG bake) are blind the same way —
  measured 0 of 5462 river cells changed. **So the reference's exported `map.png` has no river
  water colour either.**
- `burnChannels` does not compensate: it is off by default (`_lodBurnRivers`) and its only output is
  `tile[i]=Math.max(floor,tile[i]-burn[i])` — heightmap carving, not colour.

**What this means for the port.** If you implement the tile colorizer by porting
`renderBiomeTileRGBA`, you inherit a renderer with no river. The HTML papers over this with a
**vector overlay** in `drawLODView` (`drawRiverWays`, a reprojected Catmull-Rom spline), which is a
cartographic SYMBOL — sqrt-z-damped width with a real-half-width floor — and therefore renders
rivers in a **visibly different style** from the terrain-blended off-LOD map. A port that wants one
consistent look across zoom levels must put the water inside its tile colorizer rather than
reproduce the HTML's two-renderer split.

> **Correction, v2.40.** This paragraph previously advised sampling `_riverNet.intensity`/`depth` at
> world coords inside the tile colorizer. **Do not do that** — those are rasters, so sampling them is
> a bilinear smear of a one-cell stamp that gets blurrier at every pyramid level rather than sharper.
> v2.40 does it from the polyline instead; see **§6f**, which supersedes this advice.

**The v2.39 change itself** is one gate: that overlay was gated on `state.viz.riverWays`, which
v2.29 defaulted to false, so at defaults LOD drew no river at all (measured: river-vs-land contrast
28.27 off-LOD against 10.95 under LOD, the residual being only the carved valley). It is now
`((riverWays) || state.showRivers)`. **Do not port the v2.29 default flip without also porting
this** — off-LOD the flag is a real either/or preventing two renderers drawing one network; under
LOD there is no second renderer for it to select between.

---

## 6f. The river must be evaluated from its geometry, not sampled from its raster (v2.40, DCC line only)

This is the constraint §6e pointed at, now resolved in the HTML — and the resolution is **not** the one
§6e originally suggested. Port this shape, not that one.

**The key fact, and the reason a geometric evaluation is legal rather than invented.**
`buildRiverNetwork` stamps, per channel cell:

    halfW        = (0.6 + 3.0*mag² + 0.45*(order-1)) * slopeFac * widthK   clamped [0.5, 9*widthK]
    intensity[j] = amp * (1 - dist/halfW)
    depth[j]     = d01 * (1 - dist/halfW)      (max-combined across contributing cells)

That is a **linear falloff from the centreline** — a signed distance function that merely happened to be
evaluated on the coarse grid. `traceRiverPolylines` returns the same centreline as continuous geometry
and the network now also returns `halfw` (added v2.25), so the identical function can be evaluated at any
resolution. A tile colorizer that does so is not synthesising detail; it is re-evaluating the function the
coarse raster was a low-resolution *sample* of.

**`riverFieldTile(bounds, W, H, cx, cy)`** is the HTML's implementation: reject polylines by a
half-width-expanded bbox, then stamp each segment's AABB into a max-accumulator in TILE PIXELS, returning
`s`/`d` on exactly the scale the blend expects. Cost is the channel's own area, not the tile's.

**Read the peak values off the CENTRELINE CELL.** There `dist=0` so `t=1`, which makes `intensity[i]`
exactly `amp` and `depth[i]` exactly `d01`. No width formula is duplicated and there is no second model to
drift from `buildRiverNetwork`'s — the "two functions answering one question" failure this file documents
eight times.

**Width scaling is free, and must not be re-invented.** `halfW` is in GRID CELLS; the tile converts once
by tile-pixels-per-coarse-cell, which doubles every pyramid level. So the symbol→true-scale crossover
lands at a **different level for every river**, which is correct — production cartography derives that
switch from the rendering pixel (openstreetmap-carto: `WHERE way_area > 1*!pixel_width!*!pixel_height!`)
rather than picking a zoom. `RIVER_TILE_MIN_PX = 0.55` is the symbol floor beneath the crossover, applied
as a `max` so true width wins the instant it is wider.

**The verification a port should reproduce.** Hold the WORLD rect fixed and vary tile resolution. If the
pass is resolving known geometry, the channel's area in world units is the same number every time and only
its pixel count grows:

| tile | channel area | in world units |
|---|---|---|
| 64 px | 121 px² | 79.92 cells² |
| 128 px | 497 px² | 80.78 cells² |
| 256 px | 1 934 px² | 77.97 cells² |
| 512 px | 7 690 px² | 77.20 cells² |
| 1024 px | 30 898 px² | 77.40 cells² |

**A blue-contrast threshold is not a valid test.** Carved valleys plus `landColorCore`'s TWI wetness term
already make channel cells ~21 bluer than surrounding land with **no water drawn at all**. Key every
assertion to a delta against the same build's own suppressed baseline. The HTML's first attempt at this
used an absolute threshold and passed on the broken build.

**Gating — port this exactly.** `state.viz.riverWays` is an either/or (v1.14): on means the stroked spline
is the only renderer, so the raster blend must stand down or one network draws as two parallel rivers.
v2.39's widened gate (`riverWays || showRivers`) was correct only while LOD had no raster renderer; v2.40
reverts it, gives the tile pass `surfaceColor`'s condition character for character, and the two paths now
agree in both modes.

**Other pieces of the same change:** `applyRiverWater(c, s, d)` is the single blend, shared by
`surfaceColor`, `renderBiomeTileRGBA` and both bake loops; `riverLakeSkip()` is the shared "this point is
in open water" predicate; `_lodRenderKey()` gained a `state.showRivers` term because tile pixels now
depend on it. `bakePixel` is per-pixel with no tile bounds, so the bake evaluates the field once per strip
or tile and blends after it returns — **so the reference's exported `map.png` now does carry river water**,
correcting §6e's statement above for v2.40 onward.

**Still unbuilt in the HTML, and therefore still open for the port.** The channel is only as sharp as
`field` carries it: `carveRiverValleys`' groove is upsampled by `amplifyRegion` and then roughened by
`addZoomDetail`. Re-asserting the carve at tile resolution is the owner's LOD6 "banks and valley" rung.
The piece for it exists and is pointed at the wrong source — `burnChannels` already has the right
hydraulic width law (`W ∝ Q^0.5`) and the right quadratic cross-section, but reads bilinearly-interpolated
coarse `mag`, and its `widthK` is a radius in **tile pixels**, so it collapses from 6.0 coarse cells at
z=0 to 0.023 at z=8 — the same scale defect §6b/§2 record in four other subsystems. Also:
`renderHeightTileRGBA` is still river-blind, and no river geometry is persisted (`loadZip()` reads back
six entries and rebuilds the network from `flowField`).

---

## 7. Cross-cutting calibration rules

These are not features; they are the rules the HTML repeatedly re-learned the hard
way, and a fresh implementation can re-break every one of them.

### 7.1 Calibrate against the world's own measured distribution, never an assumed one
Sea level is re-anchored from the generated field's own height histogram; resource
density is normalised so the land-integrated ceiling is preserved; food surplus is
calibrated against the world's **measured median** soil. Assuming a distribution has
broken a calibration at least three separate times. Debug-view normalisers are the
deliberate exception — fixed, so a colour means the same thing across worlds.

### 7.2 A term that is not ~zero almost everywhere is reweighting the whole map
CORE terms exist at every land cell and must sum to 1.0 — they set where the sigmoid's
0.5 pivot falls. OPPORTUNITY terms (coast, river, lake, minerals) are zero for most of
the map and are ADDED on top. **Never spend core weight on a term that is usually
zero.** Check the land-mean of any new term before trusting it.

### 7.3 Two functions answering one question WILL drift
This has been paid for at least eight times in the HTML — two suitability scorers, two
coastal tests, two trade rules, a reader/writer pair, a selector/validator pair, five
place-pick sites, three land cost functions, a draw/reserve pair. When you find the
same question answered twice, collapse it to one function rather than syncing them.

### 7.4 The population chain must stay ACYCLIC
terrain → carrying capacity → rural population → surplus → urban ceiling. Nothing
downstream may feed back. The HTML asserts this with three dedicated checks (own
population cannot change own supply; growth cannot inflate a neighbour; the pass only
ever caps).

### 7.5 A distance threshold below one cell width is unsatisfiable, not strict
The chamfer DT and the river trace both quantise to whole cells, so the smallest
non-zero distance is one cell. Use a cells-based floor (the HTML's `_umWaterReachKm()`
is ≥1.5 cells), never a bare km literal.

### 7.6 Any per-tile pass with a spatial neighbourhood is a seam unless sampled from a world-wide field
Blur, SDF and AO all clamp at the array edge, so neighbours smooth a shared boundary
from opposite truncated neighbourhoods: identical height data in, different colour out.

### 7.7 Every verdict carries a basis string
A bare "none" cannot be told from a broken threshold — which is precisely how several
defects survived multiple versions.

---

## 8. Adjacent changes in the same span — not simulation, listed so nothing is silently dropped

| Version | Change | Why it may still matter to the port |
|---|---|---|
| v2.12 | Field-level redo, autosave snapshots, failed-open recovery | `loadZip()` reads back exactly **six** of `exportZip()`'s ~30 entries (`params.json`, `heightmap.f32`, `temperature.f32`, `rainfall.f32`, `volcanic_field.f32`, `impact_field.f32`) — everything else is write-only. That fact is what makes a snapshot cheap. |
| v2.14 | Dead-code sweep | One real bug fixed: `resource_index.json` was documented but never pushed into `exportZip()`'s entries. |
| v2.15 | Unified search + dismissible setup gate | The control index is **scraped from the DOM**, never hand-maintained; panel ownership is derived from `data-*` attributes. |
| v2.16 | CSS token aliases | Canonical palette is `--bg --panel --panel2 --line --ink --dim --faint --accent --accent2 --warn`. |
| v2.24 | The DCC editor frame | `_domain` is the ONE writable navigation variable; the finalize lock is `[data-genlock]`, never DOM containment. |
| v2.26 | `exportZip()` writes the project **tree** | Matches `SAVEFILE_COMPAT.md` §1 — readers accept both layouts, writers produce only the tree. `_treeWriteEntries()` is the exact inverse of `_treeRead`, member for member. §9.3's `from`/`to` index the settlements array; the app's `aIdx`/`bIdx` index `state.places`, which also holds POIs — `_twSettleIndex` is the remap. POIs ride in `reference.pois`. |
| v2.27–v2.28, v2.31 | Shell/CSS | Phone layout only. |
| v2.40 | The river moved INTO the tile colorizer and the PNG bake | Simulation-adjacent and load-bearing — see **§6f**. It also supersedes §6e's original porting advice: evaluate the river from its polyline geometry, never by sampling `intensity[]`/`depth[]`. |
| v2.39 | The tiled-LOD river overlay's gate widened | Small edit, load-bearing constraint — see **§6e**: the HTML's river water colour is unreachable from the LOD/bake colour path, so a port that reuses `renderBiomeTileRGBA`'s shape inherits a renderer with no river. |
| v2.38 | The three long civilisation buttons wrapped in `withBusy` | Not simulation. Worth knowing anyway: Auto-populate is **11.2 s of synchronous main-thread work** and gave the click no acknowledgement at all, which the owner reported as it not working. Any port that runs this on the UI thread inherits the same report. See **§6d** for where the time goes. |

---

## 9. Deliberately NOT in the HTML — do not port these

- **Multi-ridge orogenic belts ("the Himalaya problem").** The HTML's mountain ranges
  are a single symmetric ridge by construction: `stressField = gaussBlur(raw, blurR)`
  over a 1–2 cell boundary line, and the whole mountain term in `fillHeightRows` is one
  variable, `T = oro ? oro[i] + Math.min(sf,0) : sf`. A multi-sheet thrust-belt profile
  was prototyped and calibrated as a **runtime override only** — it is **not in any
  shipped file** and nothing was committed. Do not implement it from this note.
- **Causal landmark generation** — not started in the HTML. `LANDMARK_GENERATION_SCOPE.md`
  in this repository is the port's own design, not a port target.
- **Real-km-aware orogenic belt width.** `blurR` is in grid cells and never reads
  `mapWidthKm`, so belt width does not scale with map extent — the same defect shape
  that `terrainDetailK` / `riverCoarseEase` / `lodDetailFreqK` / `riverWidthScaleK`
  each fixed in their own subsystem. Known and open on the HTML side.

---

## 10. How to verify a port of any of the above

The HTML's own harnesses, and what each is good for:

| Harness | Covers |
|---|---|
| `tests/run.sh` | script block 1 (engine), CPU paths only. **Quote it as "0 failed", not as a count** — the total is not constant across runs. |
| `tests/run_um.sh` | script block 4 (urban morphology), pure and DOM-free |
| `tests/perf/hash_gen1.js A.html B.html` | FNV bit-identity battery over `field`/`temp`/`rain`/`flow`/`rgba` |
| `tests/perf/probe_craters.js` | §1.2 |
| `tests/perf/probe_carve.js` | §2.1, §2.2 |
| `tests/perf/probe_blur.js` | §3 — **needs a real browser**, the headless suite has no WebGL2 |
| `tests/perf/probe_roadconnect.js` | §5.1 |
| `tests/perf/probe_manpower.js` | §5.2 |
| `tests/perf/probe_landmass.js` | §5.3 |
| `tests/perf/probe_trade.js` | §5.4 |
| `tests/perf/probe_passes.js` | §1.1 — browser-only (checkboxes, `syncUI()`, `generate()`) |
| `tests/perf/probe_landsurface.js` | §6.2 |
| `tests/perf/probe_margins.js` | §6b.1 |
| `tests/perf/probe_orogeny.js` | §6b.2 |
| `tests/perf/probe_seamcarve.js` | §6c — **world mode, seed 21811**; the only harness here that can see the seam |

Two harness traps the HTML hit that apply to any parity suite:

- **`tests/stub_head.js` returns `null` for any `getContext` but `'2d'`** — the headless
  suite has no WebGL2 at all, so it never exercises a GPU route and cannot see a
  GPU/CPU divergence.
- **Two suite assertions decide on an unpinned ambient seed** and are known-flaky (an
  SST warm/cold-cell check and a world-seam delta). If either goes red, re-run before
  believing it. The prescription on the HTML side is an aggregate over pinned seeds.
