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
| Covered by this document | **v2.11 → v2.55** |

**The HTML source has two lines, and they diverged at v2.22.** This matters more
than anything else in this document:

- **Mainline** — `Cartalith Gen1 v2.22.html` is the newest mainline file. It carries
  everything up to and including v2.22.
- **DCC line** — `Cartalith v2.23 … v2.55 DCC test.html`. v2.23 duplicated v2.22 to
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
- **Read the v2.49 row before implementing any of this.** The width model above is
  correct and was receiving a CONSTANT on any map past ~12 800 km, for two stacked
  reasons: `riverWidthScaleK` shared its lower clamp with the whole `terrainDetailK`
  family and saturated there, and `buildRiverNetwork`'s 0.5-cell raster floor then
  swallowed every remaining difference before `halfw[]` was returned. A port that
  implements v2.25's `max(symbol, real)` crossover on top of that inherits a river
  network of one uniform width at every Strahler order.

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

## 6g. Flow must be routed over filled depressions, or there are no trunk rivers (v2.41, DCC line only)

**This is the most consequential simulation finding in this whole document, and a port that copies
`computeFlow` as written inherits it.** It is not a river-rendering issue; it silently distorts every
consumer of river order — settlement placement, water access, navigability, food-shed mode, the
Journey Planner.

**The defect, measured on the shipped engine.** `computeFlow` accumulates on the RAW `field` with no
depression filling. Every local pit terminates accumulation. At 512 px / seed 12345:

| | share of land |
|---|---|
| drains to an interior pit | **66.5%** |
| drains to the sea | 29.6% |
| drains off a map edge | 3.9% |

At 1024 px the single largest channel cell carries flow 14 072 and its receiver is a pit **~391 m
above sea level**; five of the six biggest do the same; the largest flow reaching the sea anywhere is
2 355, six times smaller. The drainage is 732 separate basins, median 3 channel cells. Max Strahler
order is 2–3 world-wide and there was **not one order-3 outlet** across five seed/resolution/extent
combinations tested. The carve makes it worse: land pits 334 → 550 per `carveRiverValleys()` call.

**The fix, and the part that is easy to get wrong.** `buildRoutingSurface` is a Barnes priority-flood
seeded from every sub-sea cell and every map edge (x edges only when not wrapping) — **with an epsilon
tilt**. A plain flood leaves a filled basin perfectly flat, and the receiver search requires
`drop > 0` strictly, so a flat basin terminates accumulation *exactly as the pit did*. Each cell is
raised to `max(own height, neighbour + eps)`. **A fill without the tilt is not a fix.**

**It must never modify the heightmap.** The terrain keeps its pits; only routing sees them filled.
That is the standard hydrological-correction distinction, and it is what keeps lakes lakes — the
water-body classifier still reads the real surface, and a river correctly flows *through* a lake to
its outflow instead of stopping dead in it.

**Both trees need the same surface.** `buildRiverNetwork` constructs its OWN receiver tree (the R3b
aspect-projected single receiver), still on the raw field. Filling `computeFlow` alone moves the
accumulation and leaves the TRACED network unchanged — the two then describe genuinely different
objects, which is itself a defect worth not porting. In the HTML this is `opts.routeOn`, threaded at
every live call site. Note the one deliberate exception: the **slope** field keeps the REAL gradient,
because it feeds the channel-initiation threshold, which is a statement about ground steepness, not
about where water goes.

**Do not verify this with a flow ratio.** "Max flow reaching the sea ÷ max flow on land" reads 28.5%
before the fix and 14.4% after — which looks like a regression and is not. In region mode the largest
basin often exits via a **map edge**, a perfectly legitimate outlet, so that ratio measures the crop.
Walk every land cell's receiver chain to its terminus and classify it instead: **pit 66.5% → 0.0%**,
sea-draining land ×1.47, outlet Strahler 2 → 3, land fraction unchanged to four decimal places.

### Deltas, which this unblocks

Two independent reasons the reference could not build one, both worth checking in a port.

1. **The sediment router's sub-sea branch is an asymptote.** `dep = min(load, (sea − h) × 0.5)` with
   each cell visited exactly once closes at most half its own depth, so it can never cross into land
   however much sediment it is given. Measured with the real carve supply: 45 612 sub-sea cells
   raised, 53 crossed sea level, and **all 53 were sinks pooling, not progradation**.
2. **The one river-mouth process runs the wrong way.** The coastal pass's estuary branch SUBTRACTS
   height where a major river meets the coast (712 of 720 gate-matching cells lowered, mean −3.59 m
   per pass) — it builds drowned valleys. It is also default-off, so the reference has **no** mouth
   process at all in either direction. Land fraction around the 40 biggest outlets vs ordinary coast:
   +0.113 at r=4. The river just stops at the shore.

**Calibration is the entire difficulty, and it runs the opposite way to intuition.** Sediment is
**over-supplied by roughly 60×** — median water depth just offshore of the biggest mouths is 13.9 m
against 1 133.8 units of eroded column. Routing all of it builds ~11 900 km² of new land, about 160
Mississippi deltas. Three controls, each load-bearing:

- A delivery ratio (0.10, inside the literature's 0.05–0.30 basin sediment-delivery band). Isostatic
  rebound already consumes that same eroded column as broad uplift, so routing 100% double-counts.
- **A swell floor on the wave term.** A naive onshore-wind projection gives **63% of mouths zero wave
  energy** — on a fixed wind field most coastline is a lee shore — which would make 63% of mouths
  river-dominated, the exact over-correction above. Real oceans get far-field swell regardless of
  local wind; the floor is the physical term, not a fudge.
- **`R = Qr / Qs,max` pivoted on the world's own R distribution, never an absolute cutoff.** The pivot
  is R at the target quantile, so the same ~10% of mouths land river-dominated whatever the world's
  absolute discharge scale. Sources: Nienhuis, Ashton & Giosan, *Geology* 43:511 (2015) for R; Nienhuis
  et al., *Nature* 577:514 (2020) for the ~80% wave / 10% tide / 10% river global split that is the
  calibration target. **Most mouths must NOT become bird's-feet.** (Both figures come from secondary
  summaries — the primary PDFs were unreachable from this environment.)

Measured result: 177 mouths, 13 river-dominated (7.3%), 330 km² of new land, 78%+ of it within 14
cells of a mouth.

**Do not try to give the flow model multiple receivers.** In the HTML the receiver array is a single
`Int32Array` with **nine read sites** across both script blocks, every one walking it as a tree
(Strahler solver, polyline tracer, the carve, both renderers, the GeoJSON export, the civ layer's own
river cache). A second receiver needs a different structure and breaks all nine; a mouth-only
exception has no principled bound either. Distributaries are a **post-hoc pass over the deposited
lobe** that emits extra polylines and touches the receiver tree not at all.

**Gating.** Both halves are opt-in and default off in the HTML, so its hash battery is ALL IDENTICAL
and the headless suite is untouched. Deltas require integrated drainage — a delta is a trunk-river
landform. Whether the port ships them on by default is its own call, but the drainage half is a
correctness fix and the rest of this document's simulation numbers were all measured against a world
that had it OFF.

**Still open, and stated rather than implied**: distributary branch *count* follows R, but the branch
*geometry* is a fan over the real lobe, not Edmonds & Slingerland's (2007) depth-over-bar bifurcation
(that needs mouth-bar bathymetry); no tide axis, so Galloway's third leg is absent; the coastal pass's
wrong-direction estuary branch is untouched, as is its own hardcoded flow gate at 2.5× the canonical
river threshold — an eighth instance of this codebase's "two functions answering one question" shape,
sitting in the exact function a future estuary/delta discriminator would have to touch.

---

## 6h. The deep-zoom plain has THREE floors, and only one of them is storage (v2.53 / v2.55, DCC line only)

Owner, on a LOD-7 tile of ordinary lowland: it reads as one flat colour however far
you zoom. Three independent mechanisms were each destroying the sub-cell signal, and
**fixing any one alone leaves the plain looking exactly as flat** — which is why they
are one section. Measured on one real LOD-7 plain tile (512x328 px, 6.25 km across,
seed 12345, region, 800 km, 1024 px, `metersPerUnit()` 6897):

| Floor | Mechanism | Measured cost | Fixed in |
|---|---|---|---|
| Storage | the baked height word was 16-bit | 12 524 distinct source heights -> **58** | v2.53 |
| Generation | the relief gate multiplies every synthetic octave by `min(1, hypot(gx,gy)*8)` | **100.0%** of the added octaves removed on coarse-flat ground | v2.55 A |
| Display | the Height view's hypsometric ramp is GLOBAL | **1 colour across 512 px** | v2.55 B |

**A port that fixes storage alone will reproduce the HTML's own v2.53 disclosure:
"storage stops destroying data; it does not create data or make it visible."**

### 6h.1 The stored height word is 24-bit, in a byte that was already there (v2.53)

`packHeight16` wrote `out[i*4+2]=0; out[i*4+3]=255;` — four bytes per pixel, two
carrying height, one a constant zero — and `atlasEncodeChunk` hands that `Uint8Array`
straight to IndexedDB by structured clone. **There is no PNG and no compression
anywhere in the height path** (the PNG beside it is the biome visual), so a baked
1024^2 chunk already spent 4.19 MB and threw half of it away.

- **24 is where the container stops being the limit and the SOURCE becomes it.**
  `field` is a `Float32Array`; f32 carries a 24-bit mantissa. Measured on the plain
  tile: 16-bit recovers 58 levels, 24-bit recovers **all 12 524**, 32-bit recovers
  **12 524 — not one more**. The f32 ULP just below 1.0 is 4.110665157e-4 m against
  the 24-bit step's 4.110665402e-4 m: seven figures, the same 24 bits. **Do not widen
  past the precision the source actually carries.** If a port stores height as f64 or
  as a different normalisation this argument must be re-derived, not copied.
- **Alpha is deliberately NOT a data channel.** Byte 3 is equally free and equally
  unused, and it is the one premultiplication corrupts through any canvas. 24 bits
  suffices, so the hazard stays off the table rather than becoming a comment someone
  must keep honouring.
- **Encoding is decided by FIELD PRESENCE (`hgt24` vs `rg16`), never inferred from the
  bytes.** `atlasChunkHeight(rec)` is the one place that decides. Byte inspection would
  be wrong on exactly the tiles this fix exists for — **a genuinely flat tile has a
  constant low byte**, so content cannot discriminate 16 from 24.
- **Format surface a port must match**: the atlas-ZIP manifest gained a per-chunk
  `enc` (**absent => 16**, so a pre-v2.53 archive still imports), and
  `exportRegionTiles` now writes `tiles/refined_{r}_{c}_rgb24.bin` with
  `heightEncoding:'rgb24'`. A downstream reader honouring that manifest field keeps
  working; one ignoring it **fails loudly on the filename** instead of silently
  misreading. `packHeight16`/`unpackHeight16` **stay** — they read every existing chunk
  and any legacy `heightmap_rg16.bin`, and a reader/writer pair must not be
  half-deleted. The save fallback `heightmap_rg16.bin` stays 16-bit on purpose: it sits
  beside the full-precision `heightmap.f32`, and `SAVEFILE_COMPAT.md` §15.2 forbids it
  in a tree anyway.
- **This retires the per-chunk scale+offset design** the HTML's own research had
  recommended. A port that already implemented per-chunk scale+offset is not wrong, but
  it is carrying a complexity the source has dropped.

### 6h.2 A relief gate needs a real-metre floor (v2.55, part A)

`amplifyRegion` and `addZoomDetail` both taper their synthetic detail by
`relief = min(1, hypot(gx,gy)*8)` — the coarse gradient at the sample point. On a
genuinely flat coarse cell that is ~1e-4, so the refinement pass **adds essentially
nothing** and the tile is a smooth upsample of five identical numbers. Measured on the
plain tile: the gate removed **100.0%** of every octave the ladder offered.

- **The floor is a REAL-METRE quantity, converted once.**
  `SUBCELL_RELIEF_M = 3.0` m is the metre-scale relief an ordinary land surface carries
  below one coarse cell; `subcellReliefFloor(detailAmp, metersPerUnit())` converts it to
  the gate's normalised units. **Keying it on grid cells or on normalised height would
  be the exact defect the source project has now paid for five times** (its own v1.60,
  v2.05, v2.07, v2.49, v2.51) — a physical quantity keyed on the grid stops meaning the
  same thing when the map's extent changes. A port must do the conversion, not copy the
  normalised constant: measured `reliefFloor` is **7.250e-3** at these settings and is
  not a portable number.
- **The floor is applied INSIDE the `max`, above the underwater fade** — so it is
  land-only by construction rather than by a second test, and deep ocean measures
  **max delta 0 m**.
- **It crosses the worker boundary as a SCALAR**, `opts.reliefFloor`. The HTML's tile
  worker pool stringifies a NAMED FUNCTION LIST, and a helper reached from
  `amplifyRegion` but absent from that list is a `ReferenceError` inside the worker
  that per-tile isolation turns into a **silently skipped tile**. Passing the already-
  converted number adds no name to that list. **Any port with an equivalent worker
  boundary should prefer the scalar for the same reason.**
- **Measured**: plain-tile span **5.92 m -> 10.58 m**, distinct heights
  **12 524 -> 18 638**, worst pixel delta 3.20 m. On a steep control tile the span is
  unchanged at 2160.2 m and the worst pixel moves **0.056% of the tile's own span** —
  the floor binds where the coarse grid resolves nothing and is inert where it does.
  Coldest flat land gains at most 2.98 m, driest flat land 2.82 m.

### 6h.3 The Height view's ramp is rebased on LOCAL relief (v2.55, part B)

`renderHeightTileRGBA` coloured every pixel by `r = (h - seaLevel) / (1 - seaLevel)` —
a GLOBAL hypsometric ramp. A 10 m tile occupies ~0.15% of that ramp, so the plain
paints one colour no matter how much height the tile now carries.

- **`buildLocalReliefField` is a world-wide min/max field, NOT a per-tile one, and
  that is the whole correctness argument.** A per-tile min/max ramp was built,
  measured and **rejected**: it puts a hard discontinuity at every tile boundary
  (measured **142.4/255** at the shared column). This is §7.6 — any per-tile pass with
  a spatial neighbourhood is a seam unless it is sampled from a world-wide field. The
  shipped field is a separable two-pass min/max over a radius in coarse cells, cached
  on `[GW,GH,_fieldGen,state.world,radius]`, wrapping in X only in world mode.
  Measured seam: the contrast factor at the shared column differs by **exactly 0**.
- **The TINT stays absolute; only the SHADING is rebased.** `hypso(v)` still reads the
  true elevation, so a colour still means an elevation. The local term is added to the
  hillshade value before the band mapping:
  `shL = clamp01(sh + k*(t - 0.5))`, `t = (v - lo)/(hi - lo)`.
- **It MUST be additive in shading space. A multiplicative form is a real defect, and
  it was shipped and caught.** `s *= 1 + k*(t - 0.5)` reaches `s = 1.35`, and
  `c[ch] * s` then CLAMPS in the `Uint8ClampedArray` — which is **a hue shift, not a
  brightness one**, because the channels clamp at different points. The additive form
  bounds `s` to the band's own `[0.4, 1]` so no channel can ever clamp. Measured: the
  multiplicative version fails a direct "no channel exceeds its own unshaded `hypso`
  value" test immediately; the additive one passes everywhere, with a least-squares
  scalar residual of 1.155/255.
- **`localContrastK(spanM)` fades the effect out as the tile's own local relief grows**
  (`LOCAL_CONTRAST_K = 0.7`, full below `LOCAL_RELIEF_FULL_M = 40`, zero above
  `LOCAL_RELIEF_NONE_M = 400`), so an already-expressive tile is untouched: the steep
  control measures **k = 0.000** and is byte-identical.
- **Deliberately NOT given to the Biome view.** Every consumer of `r` in the biome
  path was audited and all but one are ABSOLUTE thresholds — `materialWeights`' rock
  `smoothstep(0.7,0.95,r)` and mangrove `smoothstep(0.08,0,r)`, `rockCol`'s
  `r > 0.82` scree, `landColorCore`'s geology `smoothstep(0.5,0.8,r)`, the strata
  `sin(r*90)`/`sin(r*160)` bands, and three `r > 0` land guards. Rebasing `r` there
  would move rock onto a lowland plain. The Biome view gets part A's benefit through
  hillshade instead (measured: the plain's longest same-colour run 35 px -> 20 px with
  no colour change).
- **Measured, Height view, plain tile**: part B alone 6 -> 133 colours, longest run
  22 -> 6 px; the full ladder from bare 16-bit (**1 colour, 512 px run**) to A+B
  (**133 colours, 6 px run**).

### 6h.4 What this re-baselines

`hash_gen1.js` v2.54 -> v2.55 is **ALL IDENTICAL** in every scenario: `field` is never
touched, and the battery never enables tiled LOD. What does move is **LOD tiles and
baked atlas chunks** — a stale IndexedDB atlas keeps decoding but holds the old
pixels, so it wants a re-bake. The flat `map.png` bake reads the coarse field per
pixel through `bakePixel` and is unchanged.

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

### 7.8 A clamp shared across a family of scale factors saturates, and says nothing
`terrainDetailK`, `riverCoarseEase`, `lodDetailFreqK`, `riverWidthScaleK` and
`orogenyWidthScaleK` all read `800/mapWidthKm` and all shared `TERRAIN_DETAIL_MAX_K`
as their bound. Past mapWidthKm 12 800 the reciprocal bound binds and the factor stops
responding to real km at all — silently, since a clamp is exactly what a clamp is
supposed to do. **The shared cap is asymmetric in its harm**: on the small-map side it
prevents exaggeration, on the large-map side it holds the feature too LARGE, which is
the opposite of bounding. Check each sibling's saturation point against the extents you
intend to support, and give the one that needs it its own constant rather than widening
the shared cap for everybody (v2.49 did this for rivers and deliberately left orogeny
alone). Related: **a value that also sets a sampling rate is not a scale factor only** —
the same `halfW` feeds `carveChannelPath`'s resample step, so changing it moves `field`
even where its own raster stamp is bit-identical.

### 7.9 A floor that inflates is not a bound
The mirror of 7.8, and it cost two subsystems their scale. `stampOneCrater` and
`stampOneVolcano` floor their DRAWN radius so the stamp loop always touches a cell and
`t = d/R` is never `0/0` — load-bearing, and it must stay. But a floor that widens the
**footprint** while leaving the **amplitude** alone does not bound a sub-cell feature, it
**manufactures** one: measured at 40 000 km, 98 of 100 craters inflated and **96.9% of the
crater material the engine wrote was the floor's**. **Scale the amplitude by the AREA ratio
`(r/floor)^2`**, which conserves the integrated material exactly for any profile that
integrates to amplitude x R^2 — and area-weight the marker rasters in the same edit, or the
fix reads as done while half of it is missing. The general form: **a clamp is only a bound in
the direction it was reasoned about**; check the other direction before trusting a comment
that says it is handled (v1.60's did, and was wrong for ninety versions).

### 7.10 Refinement removes the coarse contribution before it draws the real one
The other half of 7.9, and the rule any port's LOD layer needs in writing. Once 7.9 has
established exactly what the coarse grid holds for a sub-cell feature — the profile at the
draw floor with the amplitude scaled by `(r/floor)^2` — a tile can reconstruct the feature at
its own resolution. **The correctness question is not resolution, it is double-counting**: the
coarse field already carries that smear and the refinement pass upsamples it, so the tile must
**remove** the smear before drawing the feature at its own radius. Do it with the **same
function and a negated amplitude**, never a second formula, and the removal is exact rather
than approximate.

That design also buys the property a hand-tuned one would have to fake: as `r -> floor` the
scale goes to 1, the remove and the draw become the same profile with opposite signs, and the
pass fades to **exactly nothing** at the radius where the coarse grid starts resolving the
feature. **If a port needs a fade constant at that crossover, the design is wrong.**

Three failure modes to design against, all measured rather than argued:
**(a) Two profile definitions will drift**, so let the stamp and the tile call one — and, in a
language where accumulation order is observable, make it WRITE term by term rather than return
a sum, or the refactor silently re-baselines the coarse field.
**(b) A missing name at the thread boundary is SILENT**, because per-tile error isolation turns
it into a skipped tile rather than an error; generate any layout constants that cross rather
than retyping them, since a retyped copy mis-reads every record instead of failing.
**(c) Invention is the thing to guard against, and the test is a measurement**: hold the WORLD
rect fixed, vary tile resolution, and require the refinement's area in world units to HOLD
while its area in pixels grows (measured 3 -> 702 px at a constant ~11.0 coarse cells²). A
synthesizing pass drifts. And whatever a model PRODUCES but never gave positions to must not be
drawn at all — that is inventing, not refining.

---

## 8. The rest of the span

**This section's heading used to read *"Adjacent changes in the same span — not
simulation, listed so nothing is silently dropped"*, and by v2.52 it was false
for half its rows.** Sessions kept appending newest-first without re-reading the
heading, so **v2.48, v2.50 and v2.51 — three deliberate re-baselines of the
height field — sat under a heading telling a porter they were not simulation and
could be skimmed.** That is the same expired-claim defect the source project
documents three times over (its own v2.37, v2.44 and v2.50 entries), reached here
by exactly the same route: a claim true when written, invalidated by later work,
never re-read. **Re-read a section heading before appending to it.** The split
below is what the heading always promised.

### 8.1 These change generated output — port them like any other simulation change

Each row moves `field`, or the tiles drawn from it. **Four are deliberate
re-baselines** (v2.48, v2.50, v2.51, and v2.49 above 12 800 km): a world
generated from the same seed does not come back the same, which is a decision to
carry across deliberately, not a regression to chase. **Three already own full
sections above** and appear here only so the span reads complete — go to the
section, not the row: **v2.39 → §6e, v2.40 → §6f, v2.41 → §6g.**

| Version | Change | Why it may still matter to the port |
|---|---|---|
| v2.55 | The deep-zoom plain stops being flat: the relief gate gains a real-metre floor, and the Height view's ramp is rebased on local relief | **Core LOD/refinement contract — see §6h.2 and §6h.3.** A deliberate re-baseline of **LOD tiles and baked atlas chunks** (never `field`), so a stale atlas wants a re-bake. Two constraints a port must not lose: the floor is a REAL-METRE quantity converted through `metersPerUnit()`, not a normalised constant to copy; and the local-relief field is **world-wide, not per-tile** (a per-tile ramp measured a 142.4/255 seam). The contrast stretch must be **additive in shading space** — the multiplicative form clamps in an 8-bit buffer and shifts HUE. |
| v2.53 | The baked height word widened 16 → 24 bits, in a byte the format already allocated | **Format change every port that bakes an atlas must match — see §6h.1.** There is no PNG and no compression in the HTML's height path, so the widening is free. Encoding is decided by FIELD PRESENCE (`hgt24` vs `rg16`), **never inferred from the bytes** — a flat tile has a constant low byte. Per-chunk `enc` in the atlas manifest (absent ⇒ 16); `tiles/refined_{r}_{c}_rgb24.bin` + `heightEncoding:'rgb24'`. **Retires the per-chunk scale+offset design the HTML's own research had recommended.** |
| v2.52 | A sub-cell feature RESOLVES in the tile; it is not upscaled from the smear | **Core LOD/refinement contract for any port that draws point features onto a coarse grid.** This is the other half of v2.50 and is legal under §6f's rule for the same reason the river pass was: **the sub-cell truth is DATA, not a guess.** A crater is a continuous profile in `t = d/R` with a known centre, radius and amplitude — the coarse grid merely SAMPLED it, and below the draw floor v2.50 established exactly what it holds instead (that profile at `R = floor`, amplitude scaled by `(r/floor)^2`). **Port rules.** (1) **The correctness question is DOUBLE-COUNTING, not resolution.** The coarse field already carries the smear and the refinement pass upsamples it, so a tile must **remove** the smear before drawing the feature at its own radius. Adding without removing gives a feature roughly 1.87x too deep near the crossover (measured at `r = 1.4` cells, where the coarse residue is 87% of the refined peak; at `r = 0.076` it is 0.26%, i.e. the subtraction is nearly free exactly where the refinement matters most). (2) **Removal must be the SAME function with a negated amplitude**, never a second formula. That is what makes the pass exact rather than approximate. (3) **The crossover must be silent, and this design makes it so for free**: as `r -> floor`, `sc -> 1`, so the remove and the draw become the same profile with opposite signs and the pass fades to EXACTLY nothing at the radius where the coarse grid starts resolving the feature. **No blend, no threshold, nothing to tune** — a port that needs a fade constant here has the design wrong. Above the floor there is no registry entry at all: the coarse field already carries a correctly-shaped feature. (4) **ONE profile definition, shared by the stamp and the tile**, or the two drift (the shape the HTML has paid for nine times). The HTML's helpers WRITE into the array term by term in the caller's own order rather than returning a value, because `arr[i]+=a; arr[i]+=b` is not `arr[i]+=(a+b)` in IEEE arithmetic — returning a sum would have silently re-baselined the coarse field. A port in Rust faces the identical hazard and should pin it the same way (bit-identity of the coarse field across the refactor). (5) **Measure resolution-vs-invention by holding the WORLD rect fixed and varying tile resolution** (§6f's own technique). Measured 64 -> 1024 px on one 40 000 km tile: changed pixels **3 -> 702** while the changed area in WORLD units held at **11.20 / 11.09 / 10.97 / 10.99 coarse cells²**, with the peak delta converging 1.4e-2 -> 8.3e-2 on the feature's own true depth. **Constant world area IS the proof**; a synthesizing pass drifts. (6) **The worker/thread boundary is where this fails SILENTLY.** The HTML's tile pool rebuilds its kernel by stringifying a NAMED LIST of functions, so the new tile function and both profile writers had to cross it or the pooled path throws `ReferenceError` inside the Worker — which v1.61's per-tile isolation turns into a **skipped tile, not an error anyone sees**. The record layout crosses too and is **generated** from the module constants rather than retyped, because a retyped copy mis-reads every stamp instead of failing. A port with real threads has the analogous hazard in whatever it sends across a channel. Note also the HTML's pool-eligibility test is a **whitelist** of five opt-in extras, so adding the registry to the tile options correctly keeps the batch pooled — which is exactly why the names were needed. (7) **Seam-free by construction, not by blending** (§ the v1.29 rule): the pass has no spatial neighbourhood — a pixel depends only on its own world coordinate and the world-wide registry — and that coordinate is computed with the existing detail pass's literal expression, so a shared edge lands on the identical value from either neighbour. Measured Δ **0.00e+0** between real adjacent tiles. (8) **Carry the registry as a FLAT buffer**, not a list of objects: ten slots per record (kind, position, true radius, drawn radius, the coarse amplitude pair actually written, the true amplitude pair, flags) so a thread boundary clones one allocation. Cap it, and when the cap binds keep the **largest**, since those are the ones a tile resolves first. Measured 117 records at 40 000 km and **3 at the reference extent**, where the worst tile moves 3.1e-3 of the height range — the pass earns its place on large maps and is nearly inert on small ones, which is the correct shape. (9) **What must NOT be drawn**: the physical crater model PRODUCES ~1 040 000 craters at 40 000 km and stamps 3 000. The rest were never given positions, so drawing them at tile resolution is **inventing, not refining** — §6f's rule from the other side. Only what was stamped is refined. (10) **Disclosed**: a project loaded from a save has no registry (the loader restores the height field rather than re-stamping), so its tiles refine nothing — the correct degradation, since subtracting a smear from a field that may not contain it is worse than leaving it; the registry is never serialised. The smear is subtracted as WRITTEN while the coarse field has since been eroded and carved, so the residual is bounded at both ends (~1e-5 of the range where it matters, cancelled at the crossover) but is not zero. Everything taking a TILE gets the refinement, but a per-pixel bake that reads the coarse field directly does not. `hash_gen1.js` vs v2.51 **ALL IDENTICAL**. Verified by `tests/perf/probe_cratertile.js` (16 assertions; 2 fail immediately on v2.51) plus 15 headless. |
| v2.51 | Crater depth belongs to the crater's real DIAMETER, not to its radius in grid cells | **Core simulation, and a deliberate re-baseline of every world generated from a seed.** The HTML's law was `depth = min(0.4, 0.02 + radCells*0.004)` — a normalised-height depth keyed on how many cells the crater spans — so the SAME crater got shallower every time the map got wider: one 10 km crater measured **1550 m at 200 km, 491 m at 800 km, 194 m at 5 000 km, 145 m at 40 000 km** (fixed 1024px). That is the v1.60 / v2.05 / v2.07 / v2.49 real-km defect once more, in the depth law rather than in the radius, and it also put craters at 0.07x–0.25x of the ~1:5 depth-to-diameter every simple crater has. **Port rules.** (1) **Use Pike 1977's relation, both branches**: simple `d = 0.196 D^1.010` km (essentially a flat **1:5** in D), complex shallowing as `D^0.301` (**1:11** at 10 km, **1:34** at 50 km, **1:91** at 200 km). (2) **The published complex branch does NOT meet the simple one, and a generator cannot have that.** At the transition it reads 383 m against the simple branch's 635 m — a **1.66x step**, so two craters of near-identical size come out two-thirds different in depth. It is an artefact of fitting two different populations, not a physical discontinuity. Keep the complex **exponent** (the part carrying the physics) and **re-anchor** the branch on the simple branch's own value at the transition; continuity measured at 634.3 m vs 634.6 m across it. (3) **The transition diameter is fixed by a relation, not chosen: it scales as 1/g** (Melosh 1989). Moon 19.4 km, Earth 3.2 km, and `g_earth/g_moon = 6.05` against `19.4/3.2 = 6.06`. Any port that already carries a gravity parameter gets the wider simple-crater regime on a low-gravity world for free, and the number stays checkable. (4) **Pike's `d` is rim crest to floor — measure what the stamp DRAWS, not what the formula says.** The HTML's profile puts its floor at `-depth` and its rim crest at `+0.25*depth`, so it draws `1.25*depth` crest-to-floor and would have overshot the published relation by a quarter. Divide by that factor (a named constant) and assert it by measuring a real stamped crater: **D 3.1 km -> 613 m drawn against 620 m wanted; D 31.3 km -> 1260 m against 1260 m.** A port whose profile has a different rim fraction needs its own factor — do not copy 1.25 blindly. (5) **State the size of the re-baseline.** Mean crater depth at the reference extent moves **194 m -> 558 m (x2.87, worst x3.2)**; at 40 000 km, x3.98. Rim height and the central peak are fractions of `depth` and scale with it; the `0.4` normalised ceiling is untouched and still essentially never binds. **Isolate it**: `hash_gen1.js` vs v2.50 diverges in every scenario, and with craters + volcanoes OFF on both sides it is ALL IDENTICAL — with `impactField` and `volcanicField` byte-identical either way, which is what shows the change moves only the height field. (6) **A moved coastline exposes identity bugs, and one surfaced here**: the HTML's landmass key quantises a centroid to an 8-cell block (so a name survives a small sculpt edit), and **two landmasses in one block therefore shared a key, a generated NAME and an entry in the user-rename map — renaming one renamed the other.** Measured: two distinct islets, 68 km² and 20 km², both `lm:8,19,777`. Fix without losing the stability the quantisation buys: a **colliding** entry refines to a finer block until it stands alone, and the **largest** member of a colliding group keeps the coarse key, so the dominant landmass's identity and any rename already saved against it are untouched; a world with no collision keeps every key exactly. **Any positional identity a port derives by quantisation needs this collision pass.** (7) **Disclosed, not fixed**: the crater rim height is still a flat 0.25 of depth rather than the ~4% of diameter the literature gives for simple craters (159 m against ~120 m at D=3 km — the right order, not calibrated); the `large`/`basin` flags are still absolute km rather than the g-scaled transition diameter; the volcano stamp was already real-metre keyed and needed no analogous change. Verified by `tests/perf/probe_craterdepth.js` (15 assertions; 3 fail immediately on v2.50) plus 12 headless. |
| v2.50 | A drawn-radius floor must scale the AMPLITUDE too, or it inflates every sub-cell crater and volcano | **Core simulation for any port that stamps point features onto a grid at more than one map extent.** `stampOneCrater` floors its drawn radius at 1.5 cells and `stampOneVolcano` at 2, so the stamp loop always touches a cell and `t = d/R` is never `0/0`. Both then widen the **footprint** and leave the **amplitude** alone, so a feature smaller than one cell is drawn at the floor's full depth or height. **A floor that inflates is the opposite of a bound** — the mirror of v2.49's saturating shared ceiling. Measured, legacy path, seed 12345, 100 craters: floored **3 / 16 / 43 / 94 / 98** of 100 at 800 km/2048px, 800/1024, 800/512, 5 000/1024, 40 000/1024, with the depth budget kept **x0.9987 / x0.9542 / x0.8125 / x0.2823 / x0.0312** — so at world extent **96.9% of the crater material the engine wrote was manufactured by the floor**, and a single 6 km crater was drawn 58.6 km wide, removing **381.5x** its own material. The volcano half is starker because nothing damped it: its height is `(heightM/state.peakM)*0.9*(1-age*0.5)`, keyed on real metres and therefore completely independent of how many cells the cone spans, so at 40 000 km **99.8%** are floored and the p50 one is drawn **78.1 km across against a true 8 km at full real height**. **Port rules.** (1) **Keep the floor and correct the amplitude** — the floor is load-bearing (`R = 0` is `0/0`, and the loop must touch a cell or a sub-cell feature writes nothing at all), so check what a guard is guarding before removing it. (2) **The conserving factor is the AREA ratio `(r/floor)^2`, not the radius ratio.** Both profiles integrate to amplitude x R^2 — the crater's `depth*(1-t^2)` gives `depth*pi*R^2/2`, the volcano's `H*(1-t)^p` is the same family — so the area ratio makes a sub-cell feature contribute exactly the material its own real radius accounts for. (3) **Assert conservation on the real stamp, not on the formula.** A volcano's height is r-independent, so its integrated material must come out exactly proportional to `r^2` with **no step where the floor takes over** (measured 0.267774 / 0.267782 / 0.267783 / 0.267783 across r = 0.1…1.0). A crater's own depth law carries an `r` term (`0.02 + 0.004r`), so the invariant there is `vol / (depth(r) * r^2)` (1.44015 / 1.44080 / 1.44082 / 1.44081) — stating it that way is what shows the conservation is exact and the whole residual belongs to the depth law. (4) **Area-weight the MARKER rasters too.** `impactField` and `volcanicField` are `max(existing, (1-t)*…)`; left unscaled, a crater covering 0.6% of one cell still claims full impact intensity over nine, and both feed resource potentials. A half-fixed stamp is worse than an unfixed one because it reads as done. (5) **Isolate a re-baseline or the claim is unfalsifiable** — the divergence is total at 512px (43/100 floored), and only turning **both** stamps off on both sides and getting byte-identical output proves it is this change and nothing else; at the reference extent the whole crater depth budget moves **0.13%**. (6) **Disclosed, measured, deliberately not fixed in the HTML**: `depth = min(0.4, 0.02 + radCells*0.004)` keys crater depth on radius **in CELLS**, so one 10 km crater is **141 m deep at 40 000 km against 491 m at 200 km** and 0.07x–0.25x of a simple crater's ~1:5 depth-to-diameter — the same real-km defect as v1.60 / v2.07 / v2.49, in the depth law rather than the radius. A port adopting a real d/D relation would make craters **4–14x deeper at every extent**, so it is a look decision, not a scale fix. (7) Also disclosed: the physical crater model produces **1 040 000** craters at 40 000 km and stamps **3 000**, while **16.4%** of the full produced population survives wear — so ~167 000 real craters are dropped by the stamping ceiling rather than "already lost to erosion" as its own note implies. They were never given positions, so drawing them at tile resolution would be **inventing, not refining** (§6f's rule). Verified by `tests/perf/probe_craterscale.js` (15 assertions; 2 fail immediately on v2.49) plus 9 headless. |
| v2.49 | River channel width follows discharge again: the real-km scale factor gets its own floor, and the raster's 0.5-cell floor stops reaching the renderers | **Core simulation for any port that renders rivers at more than one map extent.** Two clamps, stacked, and the symptom (uniform-width rivers at every zoom on a 40 000 km world) is neither an LOD nor a renderer defect — the generator never produced varying widths to refine. (1) `riverWidthScaleK(mapWidthKm) = clamp(800/mapWidthKm)` shared `1/TERRAIN_DETAIL_MAX_K` (1/16) as its lower bound with the rest of the `terrainDetailK` family, so it **stopped responding to real km at mapWidthKm 12 800**: at 40 000 km it wants 0.02 and returns 0.0625, 3.1x past its own ceiling. The shared cap is **asymmetric in its harm** — on the small-map side it correctly stops a channel being exaggerated into a band of cells; on the large-map side it holds the channel too WIDE, i.e. the opposite of bounding. `RIVER_WIDTH_MIN_K = 1/128` is now its own constant. (2) `buildRiverNetwork` floors the per-cell `halfW` at 0.5 cells, and with `widthK` pinned at 0.0625 the largest value the width formula `(0.6 + 3.0·mag² + 0.45·(order−1))·slopeFac·widthK` can produce is **0.3656** — so every channel of every order clamped and the returned `halfw[]` was uniformly 0.5, i.e. a 39.06 km band for the trunk and the trickle alike (four times the Amazon). Measured after: 0.079 / 0.158 / 0.251 / 0.265 km by Strahler order; widest point 39.06 → 1.62 km. **Port rules.** (1) **A shared clamp constant across a family of scale factors is a bug waiting on a large enough map** — check each sibling's saturation point against the extents you intend to support, and give one its own bound rather than widening the shared cap (`orogenyWidthScaleK` saturates the same way and was deliberately left alone: different subsystem, own v2.36 calibration). (2) **The floor cannot be zero and a port must keep one.** Both consumers divide by the width — `buildRiverNetwork`'s `t = 1 − d/halfW` and `enforceChannelDescent`'s `t = d/halfW` — so `halfW = 0` is `0/0` and a NaN in the height field. (3) **The floor is dead weight on the raster stamp**, which is what makes the split safe: `r = ceil(halfW)` is 1 for any `halfW` in (0,1), so the only distances in reach are 0, 1 and √2, only `d = 0` passes the `d > halfW` test, and there `t = 1` regardless of the value. Verified by reproducing the loop at 0.04 / 0.12 / 0.366 / 0.5 / 0.9 — one cell, `t = 1`, every time. So the stamp keeps the floored value and the **returned** array carries the unfloored (cap-clipped) width. A port that raster-stamps and returns one number must make the same split, or its renderers receive a constant. (4) **A width parameter that also sets a sampling rate is not a width parameter only.** This change is bit-identical at the reference extent but `field` does move above 12 800 km, because v2.30's `carveChannelPath` derives its resample step from `halfW` (`step = min(0.5, halfW·0.5)`), changing the point count and hence the per-point enforced drop. Grep every consumer of a constant before claiming bit-identity. (5) **Observed, not tuned**: `slopeFac = 1/(1 + 5·|grad|·W)` suppresses the final width ~11x on that world, which is why the trunk lands at 1.6 km rather than an Amazon's 5 km — physically the right direction (mountain streams narrow, lowland rivers wide), so it was left alone rather than tuned to make one number look better. Verified by `tests/perf/probe_riverwidth.js` (11 assertions; 4 fail on v2.48) plus 3 headless. |
| v2.48 | The plate-age distance transform is exact and wrap-aware (was a 3x3 chamfer) | **Core simulation, and a deliberate re-baseline of every generated world** — port this before matching any terrain golden from v2.47 or earlier. `distanceToBoundary()` was a two-pass 3x3 chamfer with weights (1, 1.4142). Its error is **directional**: measured against true Euclidean from a single source, **0.00% at 0°, 45° and 90°, +8.15% at 22.5° and 67.5°**. Its level sets are therefore **octagons with flat facets**, not circles. That field becomes `ageField`, which the height formula spends as the **noise amplitude** (`rug = exp(-age*(1+ageInf*6))`, then `B*N*(0.25+0.75*rug)`) and which also feeds `resistanceField` — so terrain roughness and erosion resistance both inherited the octagons, and the generated world grew flat facets whose edges can only run at 0/45/90/135°. Replaced by an exact Euclidean transform (Felzenszwalb & Huttenlocher 2012 — separable, O(n), the same cost class, ~2x the constant). **Port rules.** (1) A distance transform that feeds terrain must be EXACT; a chamfer's error is systematic and directional, so a mean-error check will pass while the field is visibly octagonal. (2) Measure a transform's error **by angle**, never as an aggregate. (3) On a wrapping world the transform must wrap: the HTML's did not, and its seam column read **251 against a true 5**, holding 63 of that world's 173 straight coastline cells. A region must NOT wrap — it has real edges. (4) The artefact was present at every scale and legible only at one: that world's sea level put `metersPerUnit` at 34 770 against the default 5 000, ~7x amplification, so 'not reproducible at the defaults' did not mean 'not a bug'. (5) `chamferDist()` and the civ layer's coast/ocean distance fields carry the same anisotropy in the HTML and were left alone, since they feed placement rather than height — a port should decide deliberately rather than inherit that split by accident. |
| v2.47 | LOD height refinement: a C¹ interpolating upsample, and detail band-limited to the tile's own sampling rate | **Simulation-adjacent and directly relevant to any port that refines terrain per tile.** Two defects. (1) The coarse height was reconstructed **bilinearly**, which is C⁰ — exactly linear inside a coarse cell and kinked across every boundary. Because the tile renderers hillshade from finite differences, they read that kink directly and the surface becomes a mesh of flat facets with a crease between each pair. Measured second difference **3040× higher on a cell boundary than inside a cell**, the interior reading float noise because bilinear has no curvature there at all. Fixed with Catmull-Rom, which is **interpolating** — it reproduces the source exactly at source nodes (asserted, max |Δ| = 0), which is what makes it reconstruction rather than a different surface. It overshoots the local cell range on ~1% of samples, worst ~6 m at the reference scale. **Port rule: any renderer that differentiates an upsampled height field needs a C¹ reconstruction; bilinear guarantees a crease on every source-cell boundary.** (2) The detail noise is fBm with six internal octaves at lacunarity 2, so its content reached far above the tile's own sampling rate, where it can only alias — and aliased noise re-randomises whenever the sampling grid moves, which is a surface that boils as the camera pans. Each octave is now faded out as its wavelength approaches two tile pixels, **toward the octave's own mean and never toward zero** (fading to zero leaves a DC shift and the terrain sinks). Measured half-pixel shift stability −52.8% on a large map, and unchanged at the reference scale, which is correct: nothing there is unrepresentable. (3) The octave count was `min(6, z − zBase)`, so the same world point had a different height at different pyramid levels — adding terms, not resolution. It is fixed now and gated on resolvability instead. **Port rule: gate refinement on the tile's sample spacing, never on the level index.** Two invariants held throughout and are worth copying: both samplers stay pure functions of the world coordinate reading the full source, so shared tile edges agree **exactly** (0, by construction, not by blending); and the helpers had to be added to the worker pool's stringified function list, since a missing name is a ReferenceError inside the worker that per-tile error isolation turns into a silently skipped tile. |
| v2.41 | Flow routed over filled depressions; river deltas | **Core simulation, and the highest-priority row in this table** — see **§6g**. A port that copies `computeFlow` as written inherits a world where 66.5% of land drains into an interior pit and no order-3 river reaches the sea. |
| v2.40 | The river moved INTO the tile colorizer and the PNG bake | Simulation-adjacent and load-bearing — see **§6f**. It also supersedes §6e's original porting advice: evaluate the river from its polyline geometry, never by sampling `intensity[]`/`depth[]`. |
| v2.39 | The tiled-LOD river overlay's gate widened | Small edit, load-bearing constraint — see **§6e**: the HTML's river water colour is unreachable from the LOD/bake colour path, so a port that reuses `renderBiomeTileRGBA`'s shape inherits a renderer with no river. |

### 8.2 Genuinely adjacent — shell, save format, tooling

None of these move a generated value. They are here because the port still has to
answer for the save format, the control surface and the tooling debt they carry.

| Version | Change | Why it may still matter to the port |
|---|---|---|
| v2.54 | The generation-parameter dump reads back IN (`⤓ Apply pasted settings`) | **Not simulation, but it fixes a claim the port may have inherited.** Measured against live state, the dump whose caption promised *"for reproducing this exact world"* was missing **27 generation-affecting values** — `passes` and `hydro` entirely, 16 of `climate`'s 22 fields, and five scalars that lived only in prose, where `seaLevel` was rounded to a whole percent (0.4235 → 0.42, enough to move the coastline). The fix is **ONE list with TWO consumers** (`GEN_PARAM_BLOCKS`/`GEN_PARAM_SCALARS`), not a longer hand-list. A paste is an **untrusted-input boundary**: type-matched, finite-checked, everything refused reported **by path**, and recursion bounded by the REFERENCE rather than the input. |
| v2.45 | A resolution/extent change also carries the per-cell rasters (territory, timeline history) and the faction metadata | **Not simulation; it is a data-model rule, and it corrects v2.44's row.** v2.44 rescaled the vectors and recorded the rasters as an accepted loss. Measured, the loss was wider: a 512→1024 change took painted territory from 96 659 cells to **0**, both timeline years to **0**, and every faction's culture, religion, government and **agricultural technology** back to its default — the last of which drives `foodSurplusRatio` in the HTML, so a resolution change silently reverted that lever (the HTML's own v1.54 measured the Early-Industrial-vs-Traditional gap at 2.66× on a settlement's food shed). Cause: the restore rebuilt `state.civ` from an **explicit field list**, and a reader that falls back to a default for every field the list omits turns an omission into a plausible wrong value rather than an error. **Port rules.** (1) Round-trip the serialiser's whole output, never a hand-written subset — the HTML now snapshots `_civSyncToState()`'s entire result. (2) A per-cell raster keyed by cell index is grid-relative; resample it **inversely** (walk the destination grid, read the source cell beneath each destination cell), because the forward direction leaves holes as soon as the grid grows — at 512→1024 a solid faction border returns at quarter density. (3) **Nearest-neighbour only**: a faction id is a label, so interpolating two of them invents a third along every border. (4) A history/timeline entry is a whole frozen world — its own entity positions need the same rescale as the live ones, or its diff overlay draws in the wrong place. (5) The invariant across an extent change is the **fraction of the map**, not the cell count: region→world took territory from 95 482 to 74 525 cells at an unchanged 0.5686 share. A port storing normalised coordinates and rasters as textures gets (2)–(5) for free, which is the argument for doing so. (6) **A key built out of a coordinate is a coordinate.** The HTML keys a journey's planned rest days by `name|kind|x.toFixed(1),y.toFixed(1)`, so rescaling the settlement orphaned every one of them — the stop stayed on the route, the days stayed stored, and nothing joined them, which reads as *no layover* rather than as an error. Prefer a stable id over a positional key; where one exists, remap it with the same function that built it. (7) **A cache keyed on something the restore puts back unchanged does not self-invalidate** — the HTML's year-diff cache is keyed on the year cursor and holds references to the history entries the restore replaced, so the ghost overlay drew the previous grid's entities. |
| v2.44 | The remaining four world-construction paths reset world-scoped state; a resolution/extent change rescales content instead of abandoning it | **Not simulation; supersedes v2.43's row on one point and adds a porting rule.** v2.43's note said the HTML had three world-construction paths — **it has five** (`generate()`, `loadZip()`, `loadImage()`, the region extract, `resSeg`/`extentSeg`), and three of them had each grown a different partial reset. The rule for a port: **enumerate construction sites by where the grid is (re)allocated, and give them ONE shared reset**, not a copy each. Two specific hazards the HTML measured: (a) coordinates held in GRID units silently move when the grid changes — 512→1024 took a settlement from 68.8% across the map to 34.4% and destroyed all 90 roads, so a port storing entity positions in cell units needs an explicit rescale (per axis: an extent switch changes the aspect too) or normalised coordinates from the start; (b) **session state living outside the serialised state object can never be reset by a state reset** — the HTML's sculpt draft and setup-gate flag survived every load for that reason. |
| v2.43 | Extract-as-new-world no longer inherits `finalized`; the flat project reader refuses a heightmap-less archive | **Not simulation, but TWO constraints this port's own code must satisfy.** (1) **World-scoped state must be reset at EVERY world-construction site.** The HTML had three — `generate()`, `loadZip()` and "Extract as new world" — and the third reset none of it, so a new world inherited the parent's `finalized` lock, its `worldKey` atlas association and its undo stack. A port that grows a second way to build a world inherits the same hazard; the reset list is `finalized`, the worldKey/atlas clear, undo history, and a flow pass (the extracted world had `flowSum 0` until an unrelated gate was committed, while already being rendered). (2) **`SAVEFILE_COMPAT.md` §6.1 makes `rasters/heightmap.f32` mandatory and §6.4 says refuse — and that refusal must live on EVERY reader path, not just the tree one.** The HTML's tree reader threw; its flat `params.json` reader did not, so an archive with parameters and no raster loaded "successfully" into an all-ocean world with no warning. If the port keeps a second, flatter ingest path for interoperability, it needs the same guard. |
| v2.42 | The Seasons checkbox also opens the Season (render) blend | **Not simulation — nothing to port, listed so the span's own claim stays true.** It does record one fact a port must not mistake for a bug: `computeSeasons()` writes only `tempJul`/`tempJan`/`rainJul`/`rainJan`/`koppenField` and **restores the annual `rainField`**, so enabling seasons is bit-identical on the annual path by design (v0.93). The HTML proved it inside one build — annual FNV `3576384877`, seasons-on-at-annual FNV `3576384877`. A port that wires a seasons flag and sees no change to its annual fields is matching the reference, not failing to. |
| v2.38 | The three long civilisation buttons wrapped in `withBusy` | Not simulation. Worth knowing anyway: Auto-populate is **11.2 s of synchronous main-thread work** and gave the click no acknowledgement at all, which the owner reported as it not working. Any port that runs this on the UI thread inherits the same report. See **§6d** for where the time goes. |
| v2.26 | `exportZip()` writes the project **tree** | Matches `SAVEFILE_COMPAT.md` §1 — readers accept both layouts, writers produce only the tree. `_treeWriteEntries()` is the exact inverse of `_treeRead`, member for member. §9.3's `from`/`to` index the settlements array; the app's `aIdx`/`bIdx` index `state.places`, which also holds POIs — `_twSettleIndex` is the remap. POIs ride in `reference.pois`. |
| v2.24 | The DCC editor frame | `_domain` is the ONE writable navigation variable; the finalize lock is `[data-genlock]`, never DOM containment. |
| v2.16 | CSS token aliases | Canonical palette is `--bg --panel --panel2 --line --ink --dim --faint --accent --accent2 --warn`. |
| v2.15 | Unified search + dismissible setup gate | The control index is **scraped from the DOM**, never hand-maintained; panel ownership is derived from `data-*` attributes. |
| v2.14 | Dead-code sweep | One real bug fixed: `resource_index.json` was documented but never pushed into `exportZip()`'s entries. |
| v2.12 | Field-level redo, autosave snapshots, failed-open recovery | `loadZip()` reads back exactly **six** of `exportZip()`'s ~30 entries (`params.json`, `heightmap.f32`, `temperature.f32`, `rainfall.f32`, `volcanic_field.f32`, `impact_field.f32`) — everything else is write-only. That fact is what makes a snapshot cheap. |

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
| `tests/perf/probe_lodrivers.js` | §6e |
| `tests/perf/probe_rivertile.js` | §6f |
| `tests/perf/probe_deltas.js` | §6g |
| `tests/perf/probe_lodsurface.js` | the v2.47 row in §8 |
| `tests/perf/probe_platedt.js` | the v2.48 row in §8 |
| `tests/perf/probe_riverwidth.js` | the v2.49 row in §8, and §2.3 |
| `tests/perf/probe_craterscale.js` | the v2.50 row in §8, and §7.9 |
| `tests/perf/probe_craterdepth.js` | the v2.51 row in §8 |
| `tests/perf/probe_cratertile.js` | the v2.52 row in §8, and §7.10 |
| `tests/perf/probe_hgt24.js` | §6h.1 — drives the real `atlasPut`/`atlasGet`; 2 of its 10 assertions fail on v2.52, which is what makes it evidence |
| `tests/perf/probe_heightbits.js` | §6h.1 — how many bits the word needs: 24 and 32 recover the SAME level count, because f32's mantissa is 24 bits |
| `tests/perf/probe_reliefloor.js` | §6h.2 and §6h.3 — 19 assertions; measures BOTH halves inside one build against their own off-state (floor omitted; `localContrastK` reassigned to `()=>0`) |
| `tests/perf/probe_lod7compare.js` | §6h — a FIGURE generator, not assertions: the four-rung ladder on one LOD-7 tile, every rung shipped code |
| `tests/perf/probe_geninfo.js` | the v2.54 row in §8.2 — verifies by REBUILDING, with a v2.53-shaped dump as the control that must fail |

Two harness traps the HTML hit that apply to any parity suite:

- **`tests/stub_head.js` returns `null` for any `getContext` but `'2d'`** — the headless
  suite has no WebGL2 at all, so it never exercises a GPU route and cannot see a
  GPU/CPU divergence.
- **Two suite assertions decide on an unpinned ambient seed** and are known-flaky (an
  SST warm/cold-cell check and a world-seam delta). If either goes red, re-run before
  believing it. The prescription on the HTML side is an aggregate over pinned seeds.
