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
| Covered by this document | **v2.11 → v2.65** |

**The HTML source has two lines, and they diverged at v2.22.** This matters more
than anything else in this document:

- **Mainline** — `Cartalith Gen1 v2.22.html` is the newest mainline file. It carries
  everything up to and including v2.22.
- **DCC line** — `Cartalith v2.23 … v2.65 DCC test.html`. v2.23 duplicated v2.22 to
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

## 6i. The coastline is the level set of a blurred Voronoi map (v2.57, DCC line only)

**This is the single highest-leverage number in the whole height formula and the
HTML had never named it.** A port that reproduces `fillHeightRows` faithfully will
reproduce the defect faithfully too.

### What was measured, before any fix

Owner, on a 40 000 km world (seed 77805): *"tell me what geometric patterns you
see. And I literally mean shapes and how they translate to the water."* Seven
hypotheses were measured against the coarse `field` — never against an image —
each with an 800 km same-seed control.

The coast did **not** merely correlate with the plate polygons. The pure Voronoi
partition `plates[plateId[i]].base >= 0`, with no blur, no noise and no erosion,
**reproduces the land/sea mask at IoU 0.813 / 89.5% agreement**. Locally-straight
coast runs parallel to the nearest plate-boundary segment at mean |cos| **0.968**
against a shuffle null of **0.648** (= 2/π to 1.8%), and still **0.881** at 15–20
cells of separation, where the coast and boundary fitting windows cannot share a
cell — so it is not a touching artefact.

Term standard deviations in the height formula, measured on the live globals:

| Term | sd |
|---|---|
| `baseField` (plate base, blurred) | **0.2524** |
| stress / orogeny | 0.0653 |
| flexure | 0.0539 |
| heterogeneity | 0.0244 |
| noise × `rug` (the ageField-modulated amplitude) | **0.0232** |

Smooth terms out-gradient the whole noise term **10.64:1** at the coast, and
deleting the noise entirely moves the coastline's box-count dimension only
1.0537 → 1.0295.

### The pathway is `baseField`, and the plausible candidate was `ageField`

The chain "straight Voronoi edge → linear-ramp EDT → straight iso-age bands →
straight noise-amplitude bands" is real, and carries the **smallest** term in the
formula — 10.9× below the plate-base term. `grad(base)` and `grad(age)` are both
perpendicular to the same plate boundary, **so correlation alone cannot separate
them**; the term-magnitude decomposition and the surrogate-mask test can.

### The fix, and the two rules it carries

`baseField[i] = plates[plateId[i]].base` is piecewise-**constant**, and the only
thing that turns it into a ramp is one `gaussBlur`. A box blur's boundary gradient
scales as **1/radius**, so that radius alone decides how far the noise can push the
shoreline off the polygon edge. The HTML now names it:

```
plateBaseBlurR() = max(2, state.tect.blurR * PLATE_BASE_BLUR_K)    // K: 0.35 -> 0.18
```

applied at **both** sites that build `baseField` (the main substrate build and the
imported-DEM `inferTectonics` path).

**Sweep the sign; do not derive it.** A first reading argued a *wider* ramp would
let the noise wander further. That is wrong: widening also moves where the contour
sits, and hands it a better-conditioned place to track the polygon from. Measured
at seed 77805, monotone in both directions and asserted as such:

| K | 0.35 | 0.25 | 0.18 | 0.112 |
|---|---|---|---|---|
| straight % (world / 40 000 km) | 49.3 | 43.3 | **36.9** | 29.0 |
| box-count dimension | 1.032 | 1.061 | **1.074** | 1.092 |
| IoU vs the Voronoi partition | .812 | .781 | **.747** | .721 |
| straight % (region / 800 km, the app default) | 67.6 | 65.3 | **58.2** | 53.4 |

**Refuse a value on a constraint, not on taste.** 0.25 is the last value that costs
nothing anywhere, but it moves the app default's dimension only 1.034 → 1.038 —
too little to justify re-baselining every world. 0.112 lands *exactly* on the
`max(2, …)` floor at the shipped `blurR = 18`, so the knob would silently stop
responding to `blurR` at and below its own default — **the v2.49 defect, one
version later.**

### Second, independent cause: the carve inherited a DETECTION ease

`riverCoarseEase` exists for a good reason (v2.11-era: on a coarse map a real minor
stream's catchment can never accumulate the cell COUNT calibrated for an 800 km
reference, so water that genuinely exists goes undetected — 34% → 96% of land
within reach of a river at 40 000 km). **That argument is about whether a stream
EXISTS. It says nothing about whether the grid can hold its VALLEY**, and
`carveRiverValleys` cuts a real trench into `field`.

At 40 000 km the ease pinned at its cap of 16 and took the channel-initiation
threshold **209.7 → 13.1**, cutting **8.1×** more trench (12 204 cells, 2.33% of
the grid → 98 790, 18.84%). `enforceChannelDescent` floors every carve point at
`sea − 0.06`, so an order-1 headwater reaching the coast is cut **below sea level**
and floods, and the land between two adjacent floodings is left as a one-cell
bristle **39 km wide** — **21.3%** of the coastline, against 12.8% at the same seed
and mode at 800 km, and **1.5%** with the carve off.

The fix is `carveFlowThresh() = riverFlowThresh(GW,GH) * riverCoarseEase(mapWidthKm)`
— multiplying the ease back out rather than re-deriving the raw formula, so a future
retune of the base cell-count fraction still reaches both — plus an optional
`opts.flowThresh` on `buildRiverNetwork` whose **absence is `riverFlowThresh(W,H)`
exactly**, asserted bit-identical.

**The rule for the port: one threshold function, one consumer class.** The HTML had
already split a third consumer out for exactly this reason (the Journey Planner's
own drinking threshold, because a cartographic cap "has nothing to do with whether a
thirsty party can find a spring"). The carve was a **fourth** consumer and nobody
split it. If your port has one `river_flow_threshold()`, check every caller and ask
whether it is asking *does this exist* or *can the grid hold it*.

### Net effect, and the isolation that proves it

At 40 000 km vs the pre-fix build: straight **42.5 → 36.9%**, dimension **1.059 →
1.074**, IoU **0.813 → 0.747**, one-cell bristles **21.3 → 14.9%**, coastline
**4 354 → 4 792 cells**.

**Each half was measured against its OWN off-state inside one build.** Restoring
`PLATE_BASE_BLUR_K` to 0.35 makes the app default **bit-identical** to the previous
version (FNV 2783047521 both ways) — which is what proves the whole-battery
divergence is the blur and nothing else; and `riverCoarseEase` is 1.0 at and below
800 km, so the carve fix is a no-op at the app default by construction. Both are
assertions, not prose.

### Refuted — do not re-chase these in the port

- **The coast is NOT lattice-locked.** Its period-45° direction harmonic measures
  **R4 = 0.0255**, against **0.0247** for a control of literal Euclidean circles and
  **0.7824** for literal chamfer octagons. v2.48's exact-EDT fix holds. The chamfer's
  real fingerprint is the 22.5/67.5 family (octagon control **2.566×** enriched; this
  world **0.983×** — nothing).
- **The slivers are NOT triangular islands.** 6 land components, one holding 98.4% of
  all land; the filaments are 1 cell wide along their whole length (no taper) with no
  common axis (global axial R **0.247**). They are peninsulas a connected-component
  census cannot see.
- **The tan coast band is an ELEVATION band**, not a distance buffer:
  `beachT = smoothstep(0.03, 0, r) * 0.6`, i.e. everything under 120 m. Its apparent
  uniform offset with mitred corners is **grid quantisation** — median width 2 cells
  at BOTH 39.06 and 0.78 km/cell.
- **None of it is a scale defect.** Every straightness metric measures the same or
  worse at 800 km: PCA straightness **42.5%** at 40 000 km against **62.1%** at the
  app default. It is more *legible* at 40 000 km, where one 32-cell facet spans
  1 250 km instead of 25.

### One metric this document previously relied on is a bad detector

v2.48's 4-direction exactly-collinear-run test reads **0.00%** for genuine chamfer
octagons and **12.83%** for genuine Euclidean circles — **it moves backwards** — and
reports 7.28% where a direction-agnostic PCA fit reports **42.48%** on the same data,
because it is structurally blind to a facet at an arbitrary angle. **Any straightness
number from it is a floor, not a measurement.** Use a PCA residual over a fixed
window, plus a box-count dimension, plus IoU against the surrogate partition.

### Still open, disclosed

The coastline reaches box-count dimension **1.074** against a real coastline's ~1.25,
so this narrows the gap without closing it. The remaining lever is the noise term's
own amplitude (`beta`), which is a larger tuning question and was not taken.
`chamferDist()` and the civ layer's coast/ocean distance fields still carry v2.48's
8% anisotropy; they feed placement, not terrain height.

---

## 6j. River SELECTION has no scale term, and a fragment is not a river (v2.58, DCC line only)

**`hash_gen1.js` vs v2.57 is ALL IDENTICAL — this moves no generated value.** It is
here as a full section anyway because it is a *contract* about how a map decides
what to draw at a given scale, and a port that reimplements the river overlay will
otherwise reimplement the defect. Verified by `tests/perf/probe_riverscale.js`
(17 assertions) plus `tests/run.sh` 1280/0.

### The half that was missing

Owner: *"What if we draw a river with a catmull-rom line and only render it when we
zoom to LOD 7/8 when a map is 40 or 20000km. And only do that for rivers that are
actually big. Rhine, Amazon, yellow river. And do the same for smaller rivers as we
do ways for the smaller cities... Attest your own research adversarially."*

The research refuted **two of the request's own premises and one of my own
claims**, and what shipped is the part that survived. The first finding is the one
that matters to a port:

**River DRAWING has had a scale term since v2.25. River SELECTION never had one at
all.** `riverRenderPolys()`'s cache key is `_fieldGen|GWxGH|minO` — no zoom term, no
`mapWidthKm` term — so a 50 km region and a 40 000 km world chose *the same set of
rivers*. The Catmull-Rom spline the request asked for, v2.25's symbol-to-real-width
crossover and v2.40's in-tile resolution were all already built, refining a set that
nothing had ever selected.

**Port rule: check which half of a request already exists before building the whole
of it.** The drawing pipeline was mature; the missing piece was a selector.

### The ladder this was asked to mirror is itself the defect

`CIV_LOD_PLACE` and `CIV_LOD_ROAD` are **raw zoom scalars**
(`metropolis: 0 … hamlet: 1.4`; `highway: 0 … track: 0.7`). Raw zoom carries no
information about how much ground a screen covers, so on a 40 000 km world
`hamlet: 1.4` puts a hamlet on screen at a **28 571 km** view. Copying that
convention into hydrography would have reproduced the bug one subsystem over.

The gate that shipped is **`len * _z` — the stem's own length in SCREEN PIXELS**,
and it needs no `mapWidthKm` term at all, because `_z` is already
screen-pixels-per-grid-cell in **both** of the HTML's camera conventions (under
tiled LOD the coordinates are canvas px and one cell spans `zk` px; off LOD they are
grid units and the stack is CSS-scaled by `viewT.scale`). **One expression covers
both cameras** — which is the property a port should preserve, whatever its own
camera model is.

`RIVER_MIN_SCREEN_PX = 20`.

### Two external premises, refuted — do not re-chase either

**Töpfer & Pillewizer's Radical Law is wrong for hydrography.** `n_f = n_a·√(M_a/M_f)`
predicts **49%** flowline retention from 1:24 000 to 1:100 000; USGS's own measured
figure for its hydrography layers is **10–11%**. My first derivation of it was wrong
twice over — wrong reading (fixed-sheet count vs count per unit ground area) and
wrong law. Selection here is by on-screen length, not by any count law.

**Strahler order cannot express the tiers the request named.** Measured max order in
this engine: **4** at world extent, **2** at the app's default extent (3 with
`hydro.integrate` on), against **8–12** for a real Amazon, Rhine or Yellow River.
Ordering the world's rivers by Strahler yields at most four buckets, three of which
are headwaters. **Port rule: assert what a vocabulary can actually hold before
keying a visibility ladder on it.**

**And one of my own claims was partly fabricated, which is why the probe asserts a
band rather than a number.** I stated that I had "independently verified"
OpenMapTiles' 26.165 px waterway constant, and had in fact **extrapolated two rows
of its table from the halving sequence rather than reading the table**. Read
properly, the real thresholds span **13.08 px (z11, 1000 m) to 490.6 px**, and the
z9 = z10 equality I had read as design intent is an arithmetic identity of the
halving. 20 px sits inside the real band; the assertion is the band.

### A length gate is only legitimate because the geometry changed

`traceRiverPolylines` returns **fragments** — pieces of receiver chain cut wherever
the next point is not reachable by a straight stroke (v1.29's `splitRiverPolylines`)
— and fragment length **anti-correlates** with importance:

| Measured on the real network, 40 000 km | |
|---|---|
| ρ(fragment length, drainage area) | **0.207** |
| top-100-by-length ∩ top-100-by-flow | **2%** |
| median length of the top 100 by flow | **141 km** |
| median length of the top 100 by length | **1 341 km** |

So a length gate over fragments selects the *wrong* rivers. The fix is a new
geometry, not a new threshold.

**`buildMainStems(net, W, H, wrapX)`** walks the channel network and emits whole
stems: drainage area is accumulated over the receiver graph by **Kahn's algorithm**
(topological order over in-degree), each confluence keeps its **largest-area
tributary** as the continuing main stem, and stems are emitted highest-area-first
with each cell claimed once. ρ(length, accumulated area) **0.207 → 0.963**, which is
what makes a length gate select whole rivers rather than arbitrary fragments.

> **CORRECTED IN v2.59 — read this before quoting the 0.963.** That accumulation runs
> over `net.recv`, which covers **channel cells only**, so it is a count of upstream
> *channel cells*, not a catchment **area**. Measured against `computeFlow`'s real
> catchment raster the same stems give ρ **0.105–0.398** — *worse* than Strahler
> order's own 0.379–0.734 on the identical data. §6g's "two different trees" a third
> time, this time inside a published figure. The fragments-vs-stems comparison above
> is like-for-like on one quantity and **stands**; what does not stand is reading it
> as "length tracks drainage area". **The length gate is a cartographic disclosure
> rule** — a stem's own extent in screen pixels, which is literally what it measures —
> **and a port must not reuse it as an importance ranking.** See §6k.

### Two defects in the first cut, both of which a port will meet

**(1) Accumulate on the CHANNEL tree, not the flow raster.** A first cut ranked
`net.recv` chains by `flowField` and produced stems that terminated after 5–10 steps
while the flow raster read **163 405** at their head. Cause: `net.recv`
(`buildRiverNetwork`'s aspect-projected single-receiver tree, defined only over
channel cells) and `flowField` (`computeFlow`'s D8 accumulation over *all* cells)
**are two different trees** — exactly what §6g/v2.41 already recorded when it had to
thread one routing surface through both. A port that keeps both structures inherits
the same trap.

**(2) A wrapped receiver charges a full map width, and the gate RANKS on that
number.** `buildRiverNetwork` wraps receivers in world mode
(`nx = ((nx % W) + W) % W`), so a raw `Math.hypot` across the antimeridian bills one
whole map width per seam crossing. Measured before the fix: longest "main stem"
**41 097 km ≈ 2 104 cells on a 2 048-cell grid** — one seam jump plus 56 real cells
— and because the ladder ranks on length, **every seam-crossing river was promoted
to the top of it**. Fixed with `dx -= Math.round(dx/W)*W`. This is the v1.29 / v2.37
wrap rule in a third place; a port must apply it to **every** length, geometry or
ranking computation over a wrapping receiver chain. After the fix the longest stem
is **4 879 km** (a real Amazon is 6 400 km, a Rhine 1 230 km), and the probe asserts
no consecutive step exceeds 1.414 cells.

### The spline's own geometry was grid-keyed and never refined

`drawRiverWays`'s RDP tolerance, Catmull-Rom step and meander wavelength were
`GW/900`, `GW/360` and `GW/40` — **resolution-keyed, with no zoom and no real-km
term**. At 40 000 km that is a **111 km** control-point spacing and a **1 000 km**
meander wavelength, *constant at every zoom level*: the line the request wanted to
resolve on zoom could not.

- `step` and `eps` are divided by `_z`, so the spline resolves as the camera zooms.
  They are **bit-identical to v2.57 at `_z = 1`** at GW 1024 / 2048 / 4096 (asserted),
  and measured refinement is **2.844 → 0.356 → 0.044** cells as zoom rises.
- `wl` is keyed on real km: `RIVER_MEANDER_WL_KM = 20`, which **reproduces `GW/40`
  exactly at the app's own default extent (800 km)** at any resolution, so the
  default meander is unchanged by construction rather than by coincidence.

**This is the eighth occurrence of the real-km defect** in this document's span
(v1.60 / v2.05 / v2.07 / v2.49 / v2.51 / v2.55 / v2.57, now v2.58) — see §7.11. A
port that keys any spatial constant on grid width inherits it again.

### The overlay re-traced the whole network on every call

`drawRiverWays` ran `traceRiverPolylines` + `splitRiverPolylines` + the spline pass
**uncached, per frame**, while `riverRenderPolys` cached the identical work four
hundred lines away. Measured at world extent: **32 ms per call** (11.1 ms trace and
split, 20.6 ms spline over 9 395 stems). `mainRiverStems()` uses the same cache key
its sibling uses — `_fieldGen|GWxGH|minO`, with the Min-stream-order slider inside
it (7456 → 1112 → 7456, asserted). **Port rule: when two call sites compute the same
derived geometry, they need the same cache, not two.**

### The ladder, and the premise it refutes

Stems clearing 20 screen px, world extent, LOD 0–8:

| LOD | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|---|
| stems drawn | **975** | 2531 | 4426 | 6087 | 7215 | **7456** | 7456 | 7456 | 7456 |

Three properties, all asserted: it is **monotone** (zooming in only ever *adds* a
river — a ladder that removed one would read as a bug), it **discloses** (13.1% of
the network at world scale, not all of it and not none), and it **saturates at
LOD 5**. That last point refutes the request's own "LOD 7/8" premise: everything is
already on screen two levels earlier, so LOD 7/8 is where the *spline resolves*, not
where selection happens.

### Two things a port must carry as decisions, not as defaults

**`state.hydro.integrate` is load-bearing for this ladder and stays default OFF.**
Without depression-filled routing, 66.5% of land drains into an interior pit (§6g),
so stems terminate early: the longest is **1 799 km** with it off against **4 879 km**
with it on. The HTML did not flip it here, because doing so re-baselines every world
generated from every seed — an owner decision, disclosed rather than taken. **A port
that turns integrated routing on by default gets longer, more realistic main stems
and a different world from the same seed; it should make that trade explicitly.**

**20 000 km and 40 000 km are the same measurement.** `riverCoarseEase` saturates at
`mapWidthKm = 12 800` (v2.49's finding, from the other side), so both extents the
request named share one channel-initiation threshold. Both were run; there is only
one set of numbers to quote.

---

## 6k. An ordinal river vocabulary cannot carry a cross-map threshold (v2.59, DCC line only)

**A deliberate re-baseline: `field` itself moves**, because `carveRiverValleys()` cuts
a real trench along the traced network and integration is what changes that network.
Verified by `tests/perf/probe_riverorder.js` (18 assertions) plus `tests/run.sh`
1280/0, `probe_deltas.js` 18/0 and `probe_riverscale.js` 17/0.

Owner, on v2.58's closing disclosure: *"Turn it on and give me a comparison with for
example strahler and if we should replace it."*

### The flag flip — what `state.hydro.integrate` was hiding

§6g (v2.41) built depression-filled routing, measured what leaving it off costs, and
then **shipped it off** so no existing world would move. v2.59 turns it on. Re-measured
inside ONE build at the HTML's own default (region, 800 km, 512 px, seed 12345):

| | integrate off | integrate on |
|---|---|---|
| land terminating in an interior **pit** | **68.5 %** | **0.0 %** |
| land draining to the **sea** | 26 031 cells | **42 475 cells (×1.63)** |
| longest whole main stem | 66 km | **118 km (×1.80)** |
| biggest catchment arriving at a stem mouth | 5 078 km² | **92 815 km² (×18.3)** |
| max Strahler order / outlet order | 3 / 3 | **3 / 3 — unchanged** |

**A port inherits the decision, not the default.** Two things travel with it:

- **`deltas` stays OFF.** It deposits real sediment and builds new land; that is a look
  decision, separate from whether drainage is routed correctly.
- **The save-compat guard still defaults `integrate` to FALSE, and must.** A project
  that predates v2.41 carries no `hydro` block and *was* generated without integration,
  so it has to reload as the world it was. The state literal and the loader deliberately
  disagree — the same convention §6b's `passes` and the physical crater model use.
- **Isolation is what makes "deliberate re-baseline" a checkable claim.** The whole
  hash battery diverges; with `integrate` forced equal on both sides it is **identical
  both ways** — v2.58 at `true` reproduces v2.59's default `field` FNV `3273059064`, and
  v2.59 at `false` reproduces v2.58's `528640695`, exactly.

### The comparison, and why it is a porting constraint rather than trivia

Strahler order is the HTML's river-importance currency and **nine places read it**:
`order>=3` gates navigability and harbour validity, `order>=4` the fishing
specialisation, `10+order*7` a town layout's river width, `0.45*(order-1)` the channel
half-width, `_civNavigableRiverDiscount(order)` the routing discount, plus the
Min-stream-order render slider, the `strahler` debug view and the GeoJSON export. A
port reimplements all of that. Four measured properties bound what it can carry:

1. **It is NOT resolution-dependent here, and that refutes the obvious criticism.**
   `riverFlowThresh` is `gw*gh*0.0004 / riverCoarseEase(mapWidthKm)` — keyed on the
   **cell count** — so the channel mask is a roughly constant *fraction* of the grid and
   the tributary ladder does not deepen as the grid refines: max order **3 / 3 / 3** at
   512/1024/2048 px on one seed. **A port that instead keys channel initiation on a real
   area (km²) will break this property and make order resolution-dependent.**
2. **It IS extent-dependent, and that is the defect with teeth.** At one seed and one
   resolution, `order>=3` covers **0.32 % / 3.73 % / 4.10 %** of channel cells at
   800 / 8 000 / 40 000 km — a **12.8×** swing in what "navigable" means, decided by the
   map's width rather than by the river.
3. **It cannot rank inside itself.** The top bucket spans **123.7×** in real catchment.
4. **It is not monotone in catchment.** In **4 of 6** configurations the world's single
   largest river is not the top-order one; one reads order 2 of 4 — *below* its own
   navigability gate. The whole vocabulary reaches **4**, against 8–12 for an Amazon.

**The flip is the sharpest demonstration of the limit.** Re-routing 68.5 % of a world's
land and lengthening its trunk by 80 % changes the hydrology of every river on the map,
and order reports **nothing** (3 → 3, outlet 3 → 3). §6g's own note recorded "outlet
Strahler 2 → 3"; that is true of the network `probe_deltas.js` rebuilds at
`riverDensity:1` and **not** of the shipped `_riverNet` every consumer reads — and it
fails identically on v2.58, so the drift is pre-existing (§6h/§6i's re-baselines moved
that seed's terrain). The assertion was replaced with the discharge arriving at the
coast, which is what the claim was always about.

**Stem length is not the replacement, and v2.58 never claimed it was.** Mid-ranked
Spearman against the real catchment raster: ρ(order) **0.379–0.734** vs ρ(length)
**0.105–0.398** — order wins in all six configurations. See the correction box in §6j.

### The ruling a port should carry

**Keep the ordinal tier; do not let it carry a threshold that must mean the same thing
on two maps.** Strahler order is cheap, resolution-stable and correlates with real
catchment better than any alternative in the file — it is a good *tier*. The quantity
for `order>=3`-style gates is the **catchment area `computeFlow` already accumulates,
in km²**: extent-free, resolution-free and continuous. In the HTML that is a
nine-consumer change and a re-baseline of every generated settlement, so it is
**disclosed as the owner's call and not yet made**. A port writing these consumers from
scratch pays none of that migration cost and should key them on area from the start.

### Measurement notes a port's own harness will need

- **Do not re-derive the drainage walk.** A first cut here built its own D8 receiver
  tree, omitted the **map-edge outlet** case (a region crop's edge is a legitimate
  outlet, not a pit) and read **55 % pit** on a world the existing probe measures at
  under 1 %. §7.2's defect, self-inflicted.
- **`currentRoutingSurface()` returns `null` when `integrate` is off.** That is its
  contract; the off-state must walk the raw `field`, exactly as `computeFlow` does.
- **Spearman over a 4-value vocabulary needs mid-ranks.** Ties dominate; assigning tied
  values arbitrary distinct ranks read **−0.26 to +0.41** on plainly monotone data.

---

## 6l. A river drawn as a chain of cells is not a continuous feature (v2.60, DCC line only)

**Owner report:** rivers read as *"small strokes one after another"*, and should be
painted the way lakes are once zoomed in; then, explicitly, *"be sure a river doesn't
break up into parts"* and *"use a bit of the sculpt tools to make sure a length of the
river is deep enough to constitute river."*

**A deliberate re-baseline**: `field`, `flow` and `rgba` all move at the app default.
Verified by `tests/perf/probe_riverfill.js` (20 assertions), `tests/run.sh` 1280/0,
`run_um.sh` 852/852, and the five existing river probes.

### The defect was one number, and its sibling had been right for thirty versions

`buildRiverNetwork` does not draw a polyline into the raster. It **stamps a disc at
each channel cell**, radius `r = ceil(halfW)`, and `halfW` floors at **0.5**. For any
`halfW` in (0,1) that gives `r = 1`, so the only distance in reach that passes is
`d === 0`: **one cell per centreline point**, at full amplitude.

A D8 receiver chain steps diagonally about **42 %** of the time, and two diagonally
adjacent SQUARES touch only at a corner. Measured at seed 12345 / 1024 px:

| | v2.59 | v2.60 |
|---|---|---|
| main stems that break into parts | **884 of 1 305** | **0 of 1 104** |
| individual breaks | **3 554** | **0** |
| 4-connected components in the river raster | **4 856** (for 134 rivers) | **486** |
| painted cells with no orthogonal neighbour | 2 792 (21 %) | — |

**The sibling already had the fix.** `carveRiverValleys` cuts the same rivers at
`halfW = 0.8` at order 1 — above a cell's circumradius `sqrt(1/2) = 0.7071` — and
**v2.30 fixed exactly this defect for the carve** ("a diagonal step broke the trench").
Nobody carried it to the render stamp, which is the one you can see.

### 8-connectivity is free on a D8 chain and therefore proves nothing

Every metric here is **4-connectivity**, and the headline one walks each main stem
cell by cell asking whether the painted water (river ∪ lake ∪ sea) is 4-connected from
one chain cell to the next. That is what the eye reads as one line, and it is the only
measure that cannot be satisfied by painting pixels somewhere else. **A port's own
harness must assert the same thing** — an 8-connected check passes on the broken build.

### Five render changes, none of them a new mechanism

1. **The main map joins the tile evaluator.** v2.40 routed `riverFieldTile` into the
   LOD tile renderer and both bakes and never into `surfaceColor`; v2.60 adds
   `riverMainField()` over the existing per-name render cache, so the off-LOD map and
   the tile now evaluate one function. The per-cell stamp read (and its `omax` gate)
   is gone from `surfaceColor`.
2. **A second floor, in GRID CELLS.** The existing floor `RIVER_TILE_MIN_PX = 0.55` is
   in TILE PIXELS, so it shrinks to **0.014 cells** at a 9.4 km view — the symbol
   evaporates exactly where the user is looking. `RIVER_MIN_HALF_CELLS = 0.8` is the
   grid's own statement (a centreline is known to ±half a cell). **Deliberately not
   real km** — §7.11's rule is about PHYSICAL quantities; a resolution floor must bind
   on a coarse map and release on a fine one, which it does for free (0.069 real
   half-cells at 800 km/1024 px, 1.1 at a 50 km region).
3. **The profile becomes a CROSSOVER, not a widened fade.** `1 - d/w` fades over the
   DRAWN width, so a bound floor spends the whole cross-section on a channel that is
   not there (measured: 17 081 cells painted, 9 775 at the shipped amplitude, still
   4 365 pieces). Fading over the REAL width was built and also rejected: it puts the
   elbow inside its own fade band, so the bigger the river the fainter its corner. The
   shipped form is `rIn >= wIn ? 1 - dist/wIn : 1` — the real cross-section where the
   grid resolves the channel, a FLAT mark of one circumradius where it does not. That
   is v2.25's `max(symbol, real)` expressed as a profile, and what a lake shoreline
   already does (a boolean coverage test at full strength, never a fade).
4. **The raster geometry becomes WHOLE STEMS.** `traceRiverPolylines` emits one
   polyline per SOURCE: one 14-cell crop held **13 polylines — a single 59-point stem
   and TWELVE 2–3 point stubs**, each drawn wider than it is long. Those stubs are the
   "beans" and the scalloped edge. v2.58 named this and built `buildMainStems`; the
   stroked overlay used it, the raster path did not. Same crop after: **4**.
5. **The raster stops cutting at lakes.** Every raster consumer paints the LAKE and not
   the river on a lake pixel (the tile renderer `continue`s, the bake returns the lake
   colour, the main loop overwrites), so the cut removed nothing visible and opened the
   junction — all 115 remaining breaks were lake-adjacent and every one diagonal. v1.29
   had already drawn this distinction for the GeoJSON export, in these words: *"a lake
   reach is real hydrology."* The STROKED overlay keeps its cut.

### Refuted — do not re-chase these in the port

- **Capping the floor by the reach's own length** (§7.9's rule, applied here) removes
  the beans and takes broken stems **0 → 111**: a thin stub stops covering the diagonal
  elbow to the trunk it joins. The width that guarantees continuity and the width that
  stops a stub reading as a blob **are the same number**.
- **`rdpSimplify` at eps 0.25** changes the painted mask by *exactly nothing* (same
  area 72 407, same perimeter 1 727, max deviation 0) — the scalloping was never the
  D8 zigzag.
- **A minimum-lake-SIZE gate** moves breaks 115 → 105. The breaks are along flooded
  reaches of real lakes, not at ponds.

### The sculpt half: a finishing descent pass, and why it is small

The carve cuts the network built from the **pre-carve** field while `_riverNet` is
rebuilt from the **post-carve** one, so **1 433 of 11 414 drawn steps CLIMB** (median
12.7 m, worst 816 m, 748 of them outside any lake). Root cause, and a porting
constraint in its own right: with integrated drainage on, receivers follow the
**depression-filled** surface while `buildWaterBodies` runs its **own separate**
priority-flood over the raw field — **two depression models answering one question**
(§7.3). v2.60 does not resolve that; it stops the renderer drawing the disagreement.

Sized by measurement, not by taste:

| | climbing steps | share of map moved |
|---|---|---|
| v2.59 | 12.55 % | — |
| full re-carve of the final network | 9.88 % | **5.16 %** |
| **descent on the chain's OWN cells** (`CHANNEL_DESCENT_CENTRE_HALFW = 0.5`) | **4.33 %** | **0.74 %** |

ONE pass, deliberately: it cannot converge, because every cut moves `flowField` and so
moves the network it is chasing.

### Read the EXTENT, not the stem count

v2.30's rule again. Stems fall **1 305 → 1 104** because tributaries that used to stop
at an uphill step now run on into their trunk — a MERGE, not a loss. Traced extent
RISES **13 395 → 13 907 cells (+3.8 %)** and carved coverage goes **87.7 → 91.6 %**;
median incision 88.7 → 103.9 m.

### Isolation, and one seam the change caught

Isolated both ways (§7 discipline): `carveRivers` off **and** `showRivers` off on both
sides is **ALL IDENTICAL**; `carveRivers` off alone leaves `field` and `flow`
IDENTICAL with only `rgba` moving. So the terrain divergence is exactly the finishing
pass and the render divergence is exactly the river blend.

**A widened symbol needs a widened REJECT MARGIN.** The tile evaluator pads a
polyline's bbox by the maximum REAL half-width while the stamp draws at the FLOORED
one, so a polyline outside a tile whose band reached inside was accepted in one tile
and rejected in its neighbour — a real seam, measured **0.498** on the shared column
against v2.40's `< 0.02` bound. Latent while the two radii were close; an 11× floor
bit immediately. Any port that floors a drawn width must widen the cull test with it.

### A porter re-measuring §6k on v2.60 will get different numbers, and should

v2.60 re-baselines the terrain, so every figure §6k quotes was re-measured on it: the
flip's longest whole main stem reads **66 → 212 km (3.22×**, against §6k's 1.80×), the
top Strahler bucket spans **145.9×** in catchment (123.7×), the widest order measured
anywhere is **5** (4), and `order ≥ 3` covers **0.50 % / 4.12 % / 4.58 %** of channel at
800 / 8 000 / 40 000 km (0.32 / 3.73 / 4.10). **§6k's ruling is unchanged and all 18 of
its assertions still pass** — order still reports **3 → 3** across a trunk 3.22× longer,
still cannot rank inside its own top bucket, and is still non-monotone in **4 of 6**
configurations. Quote a figure against the version it was measured on.

### The instrument, not just the result

§6k's resolution-ladder check compared max order at 512/1024/2048 px on **one seed** and
demanded equality. v2.60's terrain flipped it (3/3/3 → 3/3/**4**) without touching the
threshold it was testing. Re-measured over **five pinned seeds**: v2.59 deepens on 0 of
5, v2.60 on 1 of 5 by one step, and v2.59's mean change is *negative* — noise in both
directions, against the **1.7–2.5 levels** Horton's bifurcation ratio predicts over a 16×
cell count. The HTML's assertion is that aggregate now. **A port's harness should be
written as the aggregate from the start**: a single-seed equality on generated terrain is
the fragile-outlier shape, and it flipped here one version after it was written.

### Disclosed, not fixed

- **`buildMainStems` emits stems that terminate in mid-air**, and a floored width makes each one
  louder. 194 of 1 104 stems end neither at a confluence, nor at sea or a lake, nor at the map
  edge (58 of them ≤3 points); at the 0.8-cell floor such a 2.4-cell headwater draws as a
  1.6-cell lozenge instead of a hairline. v2.60 **cuts every one of those counts by 21–29 %**
  (246 → 194 orphans, 82 → 58 tiny, 383 → 289 stems of ≤3 points) and does not remove them —
  removing them is the refuted length-cap, which costs 111 broken stems. **A port assembling its
  own main stems should make a stem's terminus a real one** (confluence, water body, or domain
  edge); that closes this at the source rather than at the renderer.
- With **Show-lakes OFF** a lake pixel renders as land, so the river is then painted
  across the lake bed — its real course, with the lake being what is hidden.
- The two depression models still disagree, which is what leaves 4.33 % of steps
  climbing.
- `probe_lodrivers.js` asserts v2.39's design, which **v2.40 deliberately reverted**:
  5 of its 13 have failed on every version since, v2.59 and v2.60 alike.

---

## 6s. The ward sets the plot grain (v2.67, DCC line only)

Urban-layout generation again — no height, climate, flow or pixel moves, and `hash_gen1.js`
vs v2.66 is ALL IDENTICAL. **Two of the four sub-sections below are findings a port should
not have to rediscover**, whatever it decides about the feature itself.

### 6s.1 Two functions were answering one question, and one could only say "near" or "far"

`buildParcels` decided the plot-depth median with a hardcoded **`dM < 160`** — one
two-bucket radial proxy for "which quarter is this?" — while `assignDistricts`, 130 lines
later, answered the same question with the market plaza, the wall ring, the river, the
quay and the market radius, and produced **seven wards**. So the plot grain could only
ever say near or far, and a harbour, a suburb, an agrarian fringe and a riverside craft
quarter all platted identically — **while the harbour's own source comment cites lit.
review §1.1 #22 (*"warehouses = deepest plots at quay; plot frontage narrowest of any
family"*) as the reason that ward exists at all.**

The fix is the extraction, not a new mechanism: `makeWardAt(site, anchors, plaza,
wallState, maxRF, harbour)` returns the chain verbatim as a pure function of the POINT,
and `cityGen` builds **one** and hands it to both consumers. Two of its terms belong to a
parcel rather than a point — the plaza test wants the plot's own depth and the quay test
its frontage class — so both arrive as arguments; a BLOCK passes its own depth target and
no class, and the quay's geometric fallback still catches quay-side blocks.

**A port that has `assignDistricts` already has this classifier.** What it is missing is
the call from the parcel pass, and the ordering fact that makes the call possible: every
input the classifier reads (site, anchors, plaza, wall ring, maxRF, harbour) is finished
before `buildBlocks` runs.

### 6s.2 MEASURE THE HALF YOU ARE ABOUT TO BUILD — depth is inert, and that decided the design

The obvious feature is ward-driven plot DEPTH, and it would have been the defect §6o's own
probe exists to catch: a control that stores a number nothing reads. Measured over **8 016
parcels across six towns**:

- **67.6% of parcels come out BELOW `depthTarget`'s own 14 m floor.** The block's waist —
  the ray-cast `tMin * 0.42` that bounds how far a plot can run back before it meets the
  other side of its block — binds, not the lognormal draw.
- **Tripling `plotDepthVariance` 0.22 → 0.60 moves median depth 11.09 → 11.07 m.** That
  is one of the 22 parameters §6r's own UI exposes as a slider; it is nearly inert on the
  realised geometry, and a port should not expect it to do anything.
- The same measurement explains an older number nobody had connected to it: realised plot
  aspect runs **1.09–1.74** against M-PAR-2's documented 1:3–1:10 burgage band. **That is
  a block-SIZING question, not a parcel one.**

So the depth column of the grain table **redistributes inside the envelope the old proxy
already produced (20–30 m, and no row deeper than 30)** rather than extending it, and the
port should read it as retiring a duplicate answer rather than as a feature.

### 6s.3 What ships: the SUBDIVISION, which is what was live

`splits = min(subdivisionCap, floor(age/3))` was driven by **street age alone**, and the
per-round halving chance was a flat `0.4` everywhere — so a market frontage and an
agrarian-fringe frontage of the same age subdivided identically. M-PAR-1's own register
row says mature widths are generated by *"grant + subdivision history (halves/quarters,
p_split per epoch ≈ 0.1–0.2)"*, and what historically selected a plot for halving is the
**VALUE of its frontage**. `WARD_GRAIN[ward].sub` is that value — one number per
documented quarter, scaling both terms.

**Verify it as a per-ward SIGN against that ward's own baseline, never as a ranking across
the wards.** A first cut asserted that mean frontage at full strength is monotone across
the wards ordered by pressure and failed by 0.08 m: **mean frontage per ward is not a
function of the pressure alone, because the BLOCKS differ per ward** (harbour blocks are
small quay-side strips, agrarian ones broad fringe faces) and that geometry is already in
the baseline. Measured, comparing each ward to itself: market **−1.36 m**, burgher −0.68,
harbour −0.53, artisan +0.03, craftriver −0.06, suburb **+1.57**, agrarian **+1.60**;
core-to-fringe gap **0.88 → 3.85 m**; the dear ward gains parcels off the same streets
(market 1 528 → 1 721) while the cheap one loses them (agrarian 419 → 378).

`wardGrain` is a new `DEFAULT_RULES.parcels` field defaulting to **1**, and **0 reproduces
the prior arithmetic bit-identically by construction** — `1+(k−1)*0` is exactly 1,
`age/3*1` and `0.4*1` are exact, and the RNG draw count is unchanged (asserted against
v2.66 over 30 towns: 0 divergent). At the default the re-baseline is **441 of 20 316
parcels, 2.17%**. **A port with no rules UI can keep `wardGrain` at 1 as a constant, or at
0 to stay on the old grain; both are one branch.**

### 6s.4 THE CORNERS ARE NOT THE FOOTPRINT EITHER — a pre-existing hole, made reachable

`buildParcels` rejects a candidate whose footprint touches water, and since v0.95 that
test has sampled **the four corners**. A plot spanning a NARROW channel has every corner
on dry bank and its middle in the river: the 852-golden suite's own model-level check
(parcel centroid inside `site.waterPoly`) caught exactly one — **craftriver, 41.3 m deep
on a 5.8 m frontage, every corner dry**.

**The hole is pre-existing and was simply unreachable** while every block platted at one
grain. That is measured, not assumed: on v2.66 across the same 80 (culture × site ×
fortified × seed) combinations, at the default *and* at `plotDepthVariance 0.60` where
depth reaches the 46 m clamp, there are **zero wet parcels** — which is what proves
widening the test rejects nothing on the old path.

The fix samples the depth axis at its **middle** as well as its ends — the centroid and
the two side midpoints, seven points in all — rather than running a polygon intersection,
which would cost a real geometric test per candidate parcel in the hottest loop in this
function for a case three samples already cover. **Any port of `buildParcels` inherits the
corner-only test and should carry this with it.**

### 6s.5 Two smaller facts

- **`par.ward` is written on every parcel** — the ward its PLOT SERIES was cut for,
  deliberately a different field from `par.district`, which is per-parcel and is then
  re-tagged by the economy (§6p) and status (§6q) passes, so a market-block parcel can end
  up a tan yard. Only the block ward had any say in how the frontage was cut, so it is the
  one that has to be readable if the grain is ever to be audited. It is **not** in
  `hashModel`'s digest.
- **The urban-morphology golden suite's assertion TOTAL is not a constant** (852 → 882
  here): its `venus waterway never crosses a building` check is per-BUILDING, and the Venus
  town gained 30 buildings. A port mirroring that suite should quote it as **"0 failed"**,
  never as a count.

## 6r. The layout engine's own parameter table gets a UI (v2.66, DCC line only)

Urban-layout generation again — no height, climate, flow or pixel moves, and `hash_gen1.js`
vs v2.65 is ALL IDENTICAL. **This section exists for one porting fact, not for the menu.**

### 6r.1 `opts.rules` was a live input of `cityGen` and nothing ever set it

The urban-morphology engine has exported **`DEFAULT_RULES` — 22 named generation
parameters across three groups** — since v0.95, together with `resolveRules(partial)`,
`applyWildness(rules,w)` and `applyPlotChaos(rules,c)`, and `generate()` reads
`opts.rules` on its very first lines and threads the **resolved** object down to the
growth loop and the parcel pass. **The host adapter (`_umPlaceContext`) has never set
it.** So every town the HTML has ever drawn came out at `DEFAULT_RULES`, and the table
was 22 numbers each declared once, each effectively hardcoded, with no control anywhere.

| group | what it governs | count |
|---|---|---|
| `street` | branch/continuation jitter, exploration share/decay/minimum, segment length median and variance, pierce chance, junction angle limit, market gradient decay, parallel-street spacing, dead-end bias, bridgehead distance and probability | 14 |
| `parcels` | frontage-width variance, plot-depth variance, subdivision cap | 3 |
| `settlement` | wall-generation threshold, minimum years between circuits, minimum extramural share, maximum wall generations, carrying-capacity weight | 5 |

**A port that has ported `cityGen` already has this table.** What v2.66 adds host-side is
where the values come from.

### 6r.2 The storage, and why a port can skip it

`state.civTypeRules[kind]` holds a rules object per settlement kind; the adapter passes
`civTypeRulesFor(p.kind)` as `opts.rules`. Three constraints:

- **The accessor returns `null`, never an empty object**, so an untouched type leaves
  `rules` ABSENT from the options and `resolveRules` takes its `DEFAULT_RULES` branch.
  Bit-identity is **structural**, exactly as §6o made it for `civParams` — so **a port
  with no such UI keeps `DEFAULT_RULES` and skips this section entirely**, and a save
  written before v2.66 reloads as the world it was.
- **The per-settlement layout cache key must carry a fingerprint of the rules.** Without
  it a stale town survives a rules edit for ever and silently — the HTML's own v1.28
  lesson about a render cache that omitted an input. Any port caching generated layouts
  has the identical hazard.
- **`civTypeRules` joins the generation-parameter dump's block list**, so the reproduction
  text carries it; like `civParams`, an import only populates keys the reference state
  already has (§ the "bounded by the reference" rule).

### 6r.3 What is NOT stored per type, and the rule that decided it

Site, culture, specialisation, fortification, population, age and seed are exposed in the
menu and **deliberately not stored per type**, because every one of them already has a
per-settlement field or is derived from the terrain. A second per-type copy of a value
that already has a home is the one-control-two-surfaces defect the HTML has paid for
repeatedly. **A port should apply the same test before adding a scope: does this value
already have an owner?**

### 6r.4b One layout lesson, since it cost a round here

The first cut put the parameters and the preview in two columns. Every DOM assertion
passed — the canvas existed, was visible, and measured 648×498 — and the layout was still
wrong: the shell caps its reading column at 720 px, so the two columns wrapped and the
preview landed below the fold, which is exactly what a comparison pane must not do. **Only
a screenshot reported it.** A port building the equivalent surface should assert the
geometry it actually needs (the preview sharing the viewport with the controls that change
it), not merely that the element is present and visible.

### 6r.5 Exposing the table reached a NON-TERMINATING region of that same range

**This is the part of v2.66 a port must carry, and it is a defect in the layout engine
itself, not in the menu.** `buildParcels` lays frontage grants along a block edge by
drawing a width from a lognormal (median 11 m, sigma = `frontageWidthVariance`, clamped
to [4.5, 16]) and **re-drawing whenever the draw does not fit the remaining frontage**.
The retry has no upper bound. When the remainder sits just above the 4.5 m floor but
below a typical grant, the only escape is drawing from the distribution's far lower
tail, and the expected number of retries explodes as the variance falls:

| `frontageWidthVariance` | expected retries against a 4.6 m remainder |
|---|---|
| 0.40 | 68 |
| 0.28 | 1 081 |
| **0.22 — the engine default** | **28 571** |
| 0.18 | ~2 000 000 |
| 0.12 | effectively never |
| **0.10 — the parameter's own documented minimum** | effectively never |

**It was latent for the whole life of the HTML because nothing ever set `opts.rules`**
(§6r.1), so every town ever generated ran at 0.22 and merely paid the tail occasionally.
It becomes reachable the instant the rules are editable — and the proof of concept's own
'Planned Grid' profile sets 0.12, so the first preset in the list was an unbounded hang.

**Three things a port should take from this.**

1. **A retry-until-it-fits loop over a heavy-tailed draw is a hang waiting for a
   parameter change.** The termination argument depends on the distribution's spread,
   which is itself a tunable. Any loop of that shape needs an explicit bound.
2. **The fix is a BOUND, not a new formula — and the obvious claim for a bound is
   FALSE here, which is worth knowing before a port repeats it.** The HTML caps
   consecutive non-placing iterations (`PARCEL_GRANT_MAX_SPIN = 4096`), and the natural
   thing to say is that a cap cannot change a run which terminates below it. It can, and
   it does: the 852 goldens pass (their fixtures never reach the cap), but an ordinary
   default-variance town runs to a measured maximum of **172 644** spins. So the bound is
   a **deliberate, bounded re-baseline of generated town layouts**, and the HTML sizes it
   by measurement rather than argument — raising the cap to 2^22 and regenerating the
   same twelve towns, **10 of 12 differ, by at most 10 parcels (1.08%), 46 of 8 939
   parcels overall (0.51%)**: the last grants of a block edge, never the town's character.
   **A test that asserted only "the goldens still pass" would have reported a
   byte-identity that does not hold**, which is why the HTML exports a seam that lets the
   cap be raised for the comparison. A port rewriting the loop properly (place the
   remainder, or stop once the remainder is below the floor) changes more and needs its
   own re-baseline; this bound changes half a percent of parcels.
3. **Bisect, do not read.** Each of the nine street parameters measured ~450 ms alone at
   population 900; the parcels group alone did not return. The combination looked like a
   scaling problem and was not one.

Measured after the bound: all six profiles × eight populations from 900 to 20 000 return
in **440–930 ms**, where the affected ones previously did not return at all.

### 6r.4 Three more engine options the host still never sets

`generate()` also reads **`opts.faith`, `opts.civicStyle` and `opts.harbourDefence`**, and
the adapter sets none of them — so the religious building, the civic hall and the harbour
defence works are all chosen by the engine's own culture defaults on every town the HTML
draws. v2.66 exposes them in the preview and labels them as preview-only rather than
leaving them silently inert. **A port wiring these to a settlement needs a field for each**
(faith is arguably the faction religion's job) — that decision is open, in both codebases.

## 6q. The status gradient, made explicit (v2.65, DCC line only)

Urban-layout generation again — no height, climate, flow or pixel, and `hash_gen1.js` vs
v2.64 is ALL IDENTICAL.

### 6q.1 `par.status`

`docs/05` §7.3's closing bullet: the status gradient is *already* implicit in the layout
(density, centrality) and nothing can read it — make it **explicit**. `assignDistricts`
now writes a `status` in `[0,1]` on **every** parcel, from quantities it already had:

- proximity to the market (the integration proxy, M-NET-10);
- whether the parcel is intramural at all (a faubourg is cheaper ground, §3.1's suburbs row);
- how far **downwind** it sits, on §6p's wind bearing, when one is available.

No new pass, no new field to thread, no constant of its own.

### 6q.2 It behaves as a gradient, and that is what to verify

A port should check the SHAPE, not that the field exists. Measured in the HTML over 14 real
towns and 10 371 parcels:

| | mean status |
|---|---|
| within 220 m of the market | **0.662** |
| beyond it | **0.218** |
| outer ground **downwind** | **0.143** |
| outer ground **upwind** | **0.403** |

§3.1 summarises the poor quarter as "to the edge **and** downwind". Both terms are in the one
number, so selecting it is a single sort rather than a second rule.

### 6q.3 The two visible ends

`patrician` is carved out of the **burgher ring**, never out of the commercial core — §3.1
row 2 is the prime frontages *adjoining* the market. `slum` comes from `suburb`/`artisan` at
the bottom of the gradient. Measured 50 patrician at mean **0.573** against 44 slum at
**0.050**. Both respect §6p.5's rule: they never steal a parcel an earlier rule claimed.

### 6q.4 There are TWO district palettes, and this is a trap worth naming

`_UM_ECON_TINT` colours the **buildings**; `_UM_DISTRICT_FILL` colours the **parcel** at the
City Viewer's `CV_LOD_CITY` tier, and there an unknown district hits `if(!fill) continue` and
is silently skipped. §6p's four districts were added to the first only, so they drew with no
quarter fill at all — and §6p's own probe asserted only the palette it had remembered.

Strengthening that assertion to cover **every district actually observed on a parcel**, rather
than a hand-written list, then surfaced a **pre-existing** hole: `buildFaithSites` has always
tagged its precinct parcels `church` and this palette had never held an entry for one
(confirmed absent in v2.63), so a cathedral close drew with no fill. **A hand-list of things
to check is the same defect as a hand-list of things to define** — the derived assertion is
what closed both.

### 6q.5 Not built

§3.1's institutional quarters — cathedral close as a *precinct*, castle bailey, Jewry by the
castle, the foreign-merchant factory at the quay, friaries in the suburbs, monastic precincts,
hospitals at the gates — each need a building or precinct to anchor on, which is a larger
piece than a gradient. `par.status` feeds district choice only; it does not drive building
grammar or parcel grain (the `buildBlocks`→`buildParcels` seam).

---

## 6p. Two site-model vectors, and the industry siting that consumes them (v2.64, DCC line only)

**This is urban-layout generation, not terrain — it writes no height, climate, flow or
pixel.** It belongs here rather than in §8.1 because it adds two real fields to the site
model and a placement pass that reads them, which a port building the urban layer needs.

### 6p.1 The two vectors

`docs/05-settlement-evolution-and-function.md` §7.1 names them as the cheap additions that
unblock everything else. `_umFlowBearings(p, orient)` returns both as **bearings in the
layout's own frame** — `worldAngle − orient`, which is the convention `_umOreBearing`
(v1.17 S6) already established, so the consumer reads them the way it already reads the ore
bearing rather than carrying a second frame convention.

| bearing | source | meaning |
|---|---|---|
| `wind` | `currentWindField()` sampled at the settlement's coarse cell | the nuisance direction |
| `downstream` | `_civRiverFlowField()` (v2.62, §6n.7) at the nearest CHANNEL cell | which way the water runs |

**Neither is invented per seed, and a port must not invent them either.** The wind is the
same field that drives the world's rainfall; the downstream direction is the same receiver
tree the router and the planner read. Both return **null on failure** — a world with no wind,
or no channel within the search radius, gets no sorting rather than a fabricated direction.

### 6p.2 Select the nearest channel on the VECTOR, not on catchment

`_civRiverFlowField` fills `km2` for **every** cell (from `flowField`, whose accumulation is
non-zero almost everywhere) and fills `fx`/`fy` **only where the receiver tree has a
channel**. A nearest-cell search keyed on `km2 > 0` therefore lands on the settlement's own
dry ground — a town sits *beside* its river — and finds no vector there.

Measured in the HTML: **1 of 14 towns got a downstream bearing. Selecting on the vector gives
14 of 14.** Search radius is `max(2, GW/64)`, the same hinterland disc `_umOreBearing` uses.

### 6p.3 "Downstream of the centre" is the wrong operationalisation — use the ORDER

This is the part most worth carrying, because the obvious reading of §4.2 is wrong and fails
quietly. Scoring riverside parcels by the sign of `(c − market) · downstream` produced **zero
tan yards across 14 real towns**, while the mirrored test produced mill races.

The cause is geometry, not the vector: one town carried **62 riverside parcels and all 62 sat
upstream of its market**, because a river clips the town box on one side and the market does
not sit in the middle of the frontage. **"Downstream of the market" is unsatisfiable for most
towns.**

What §4.2 actually describes is the **downstream END of the town's own river frontage**, which
needs no origin at all. Both trades are ranked along the one vector:

- **tan yards** (hides, dye vats, the shambles — clean water in, foul water out) take the
  **downstream** end;
- **mill races** (water *power* wants head) take the **upstream** end.

**Same field, opposite ends — which is why one vector buys both, and why "on the water" could
never have expressed either.** This is §6n's own lesson in a second subsystem: *the direction
is what is real; the origin is not.*

### 6p.4 The wind half

Kiln yards (furnaces, potteries, the stench trades) take the **extramural edge, downwind** —
§4.3/§4.4, the mechanism behind the enduring east-end/west-end sorting under the westerlies
(Heblich, Trew & Zylberberg, *JPE* 2021). Inn yards, stables and farriers gather **at the
gates**, where road traffic breaks (§4.6). Measured: 24 kiln parcels across 14 towns, **0
upwind**; 20 inn parcels.

### 6p.5 A siting driver is a property of the SITE, not of the town's trade

The pass sits **outside** the specialisation chain (`mining`/`fishing`/`timber`/…), because
every town has a downwind edge and a downstream reach. It runs **after** that chain and never
retags a parcel the chain already claimed, nor the harbour: **one industry per parcel, first
claim wins.** A fishing town keeps its waterfront.

Consequently `opts.economy` is built whenever **either bearing** exists, not only when a
specialisation does — otherwise an ordinary town would carry no vectors at all. Its
`specialisation` is then `'none'`, which every branch of the chain correctly ignores.

### 6p.6 Guards and verification

Absent `opts.economy` the pass is **inert**, so the synthetic path and the 852 urban-morphology
goldens are untouched by construction (the v0.98 opt-in rule). Each of the four districts
carries a renderer tint and its own provenance string — a district with no tint renders as the
default brown, which is the invisible-feature defect the HTML paid for twice (v1.80, v2.15).

`tests/perf/probe_industry.js`, 23 assertions. **The load-bearing one is an ORDERING, not a
sign**: within one town every tan yard must lie downstream of every mill race. A frontage
entirely on one side of its market satisfies no sign rule and must still satisfy this — and it
can only hold if the vector is genuinely consulted.

### 6p.7 A cost a port will hit too

`currentWindField()` is **36 ms** at 512px and `_umFlowBearings` calls it once per settlement, so a
235-settlement world spends ~**8.5 s** rebuilding an identical field. The HTML memoises it **in the
adapter**, deliberately not inside `currentWindField`, which v1.86 leaves uncached on purpose so the
Wind/Ocean debug views track the tilt/rotation sliders live. The key names everything that function
reads — `_fieldGen`, grid, world mode, sea level, and the whole of `climate` and `planet` — because
a missed input is a silently stale wind, which is the failure v1.86 was avoiding. **36 ms → 0.01 ms
per settlement**; the probe asserts both the reuse and the invalidation.

### 6p.8 Not built

The windmill on the windward rampart (§4.7 row 2) is a **building**, not a district, and needs
the building grammar. Warehouses at the quay were already built (v1.17 S6). Mining/quarry/salt
ribbon form is a whole settlement shape, not a quarter.

---

## 6o. Seven generation constants become parameters (v2.63, DCC line only)

**What a port needs from this section is small and exact: seven numbers that were
compile-time constants are runtime parameters now, their defaults are the old
constants, and three neighbours were deliberately left as constants.** The screen
that edits them is UI and belongs in §8.1; the parameter surface is generation and
belongs here.

### 6o.1 The seven, their keys and their defaults

`CIV_PARAM_DEFS` (block 1) holds each number **once**. `civParamDef(key)` is the
default; `civParam(key)` is the live value, falling back to the default.

| key | default | was | read by |
|---|---|---|---|
| `settleSeedThresh` | 0.42 | `SETTLE_SEED_THRESH` | `findSettlementSeeds` gate, 4 sites |
| `portPreferenceMult` | 3 | `PORT_PREFERENCE_MULT` | v1.46 coastal swap |
| `villageSuitThresh` | 0.32 | `VILLAGE_SUIT_THRESH` | `_civSeedVillages` floor |
| `villageSpacingKm` | 10 | `VILLAGE_SPACING_KM` | village suppression radius + road-proximity decay |
| `villageCap` | 200 | `_CIV_VILLAGE_CAP` | `_civSeedVillages` cap |
| `foodSurplusRatioMax` | 0.35 | `FOOD_SURPLUS_RATIO_MAX` | `foodSurplusRatio` ceiling |
| `foodShedMinPop` | 50 | `FOOD_SHED_MIN_POP` | `_civApplyFoodShedCeilings` floor |

The four constants that used to own their own literal (`VILLAGE_SUIT_THRESH`,
`FOOD_SURPLUS_RATIO_MAX`, `FOOD_SHED_MIN_POP`, and `PORT_PREFERENCE_MULT`'s inline
`3`) now read the table instead. **One number per knob** — a hand-written second
copy is how v1.72 BUG-A, v2.45 and v2.54 each drifted, and this document's own §7
records that lesson.

### 6o.2 Bit-identity is structural, not checked

`state.civParams` starts **empty**. `civParam()` falls through to the table, so a
world generated without touching the screen **cannot** differ — `hash_gen1.js` vs
v2.62 is ALL IDENTICAL by construction rather than by measurement. A pre-v2.63 save
carries no `civParams` block and reloads as the world it was (the v2.17
`state.passes` convention, §1).

**A port that has no parameter UI can ignore this section entirely and keep the
seven as constants** — the values are unchanged.

### 6o.3 These are read at AUTO-POPULATE time, never by `generate()`

None of the seven touches the height, climate or flow fields. Changing one moves no
terrain; it changes what the next settlement pass produces. The HTML deliberately
does **not** re-run that pass on a change (v2.38 measured it at 11.2 s).

### 6o.4 The surplus ceiling must scale BOTH ag-tech branches

`foodSurplusRatio`'s cap is `isDefault ? maxRatio : min(ABS_MAX, baseRatio*richMult)`
where `richMult = maxRatio / FOOD_BASE_SURPLUS_RATIO`. **Both** branches must read
the live ceiling. A first cut scaled only the `isDefault` branch, which would have
left a traditional faction responsive to the knob and an industrial one deaf — one
parameter meaning two different things.

Measured on the shipped function at soil 1.0, reference soil 0.5:

| farmers per urbanite | default (0.35) | knob at 0.70 |
|---|---|---|
| 9 (traditional) | 0.350 | **0.556** |
| 6 | 0.450 | **0.571** |
| 1 (industrial) | 0.750 | **0.750 — unchanged** |

The last row is **correct and is asserted as such**: there the yield-derived ratio
already sits below the cap, so the ceiling is not the binding constraint. A port
should not "fix" that into responsiveness — doing so invents surplus.

### 6o.5 Three neighbours are deliberately NOT parameters

Refused on a constraint, not on taste (§7's rule), and a port should leave them
alone for the same reasons:

- **`FARMERS_PER_URBANITE`** is already per-faction through v1.54's `AG_TECH_LEVELS`
  rungs. A global knob too would be two surfaces for one question (v1.57).
- **`FOOD_BASE_SURPLUS_RATIO`** is the constant `foodSurplusRatio()` pins itself to
  so existing worlds stay bit-identical. Exposing it retunes the pin, not the model.
- **`SETTLE_COAST_SWAP_TOLERANCE`** is an internal tolerance, not a world-shaping
  quantity.

### 6o.6 `civParams` joins `GEN_PARAM_BLOCKS`

The generation-parameter dump (§8.1, v2.54) carries the knobs, so its caption's
promise to reproduce the exact world stays true. v2.54's one-list-two-consumers
design made that a single list entry.

### 6o.7 How the HTML verified it

`tests/perf/probe_settleparams.js`, 30 assertions. The load-bearing ones: a knob
must measurably change what the pass produces (threshold 0.30 → **94** seeded sites,
default → **67**, 0.60 → **18**, cleared → 67 exactly), the defaults must equal the
constants they replaced, and the inert industrial case above must stay inert.

---

## 6n. A navigable river is a route, and it has a direction (v2.62, DCC line only)

Owner: *"these rivers should be navigateable as a route (but with a lower friction/cost)
and a course or direction the water flows (this also informs the travel planner)."*

This is a **routing and planner** change: it writes no height, climate or pixel
(`hash_gen1.js` vs v2.61 is ALL IDENTICAL). What it does move is **generated road
geometry**, deliberately. A port that generates roads must port it or its networks
will keep ignoring the rivers.

### 6n.1 The defect: a per-cell cost cannot express a direction

`_civTravelHours` charged the ford/bridge wait on every river **cell**, and multiplied
the navigable-river discount into every river cell as well. So:

- following a channel priced as **fording it once per cell**, and
- a road crossing a trunk river **square-on** collected 35% off the very cell it was
  building a bridge over.

Measured against plain ground at seed 12345 / 512 px, before the fix — an order-1 river
cell cost **1.44x**, order-2 **1.71x**, and an order-3 cell (the only kind a hull can
use) **3.46x**, its 1.894 h of bridge dwarfing the 0.587 h of travel it was discounted
to. **The most navigable river on the map was the most expensive ground on it.**

This is **§4's own lesson** (v2.33 moved slope to the edge because *"it is the only
place a direction exists"*) applied to the river terms sitting ten lines below in the
same function, which that version did not carry. **If the port implemented v2.33's
`edgeCost` hook, the river terms belong there too.**

### 6n.2 What replaces it

Both terms move into `_civLandTimeEdgeCost`, scaled by how squarely the step cuts the
water. Per edge `(i,j)`, with `al` the mean of `|cos|` between the step and the flow
direction at each end:

```
along = clamp01((al - CIV_RIVER_ACROSS) / (CIV_RIVER_ALONG - CIV_RIVER_ACROSS))
nav   = (both ends navigable) ? along : 0
travel /= 1 + (riverSpeed - 1) * nav      // the boat, only where a hull fits
cross  *= 1 - along                        // the ford, only where you cut across
```

`CIV_RIVER_ALONG = 0.85` (~32° off the channel) and `CIV_RIVER_ACROSS = 0.50` (60° off)
are direction cosines with a blend between them, so no cliff flips a whole edge.

Measured after: ALONG **0.302 h** against plain ground's **0.746** (0.41x); ACROSS
**1.347 h** (1.81x — the whole bridge, still paid); a **4.5x** spread on the same water.
A river below the navigability bar gets no boat whichever way you walk beside it
(**0.99x** plain).

**`|align|` — absolute value — is load-bearing, not incidental.** v1.98 (§5) established
that an undirected Prim MST has no well-defined answer for an asymmetric cost, and a
permanent road beside a river is used both ways. Measured relative asymmetry over 8 000
edge pairs: **exactly 0**. A port that makes this directional breaks its own MST.

### 6n.3 Navigability is a catchment area, not a Strahler order

**This is §6k's recommendation being taken, for a new consumer.** The three existing
consumers (`_civPlaceNavigability`, `buildSettlementSuitability`'s navigable-river term,
`_civNavigableRiverDiscount`) all gate on `order>=3`; copying that here was the obvious
move and the measurement refuses it. With only the map extent varied, one `order>=3`
label describes rivers of median catchment

| extent | median catchment of an `order>=3` cell |
|---|---|
| 800 km | **6 417 km²** |
| 40 000 km | **1 429 009 km²** |

— a **223x swing in what the word "navigable" means**, decided by how wide the map is.
The catchment is also **resolution-free**, which the order ladder is not: channel median
**483 km² at 512 px against 482 km² at 1024 px** on the same 800 km world.

```
CIV_RIVER_NAVIGABLE_KM2 = 1000
navigable(i)  ⇔  flowField[i] > riverFlowThresh  AND  flowField[i] * cellKm² >= 1000
```

`flowField` is rainfall-weighted against the world mean (§2), so this is **km² of
average-rainfall catchment** — the right quantity, since a desert river of a given area
carries less water than a wet one.

**1 000 km² is anchored twice over, independently**: (a) the head of navigation on real
barge rivers — the Thames at Lechlade and the Severn at Welshpool both sit near
1 000 km²; (b) the discharge rule — a small barge wants roughly 10 m³/s, and at a
temperate runoff near 300 mm/yr that is ~1 050 km². Neither is the source file's taste.

It covers **30.0% of channel cells at the app default against `order>=3`'s 2.0%** —
**42 → 728** navigable routing edges.

**The three existing `order>=3` consumers are untouched by v2.62.** §6k's nine-consumer
migration is still open and still the owner's call; do not read this as it having
happened.

### 6n.4 The river's worth is a SPEED, from the vessel table

The obvious number is wrong and the measurement says so. A river's real advantage is
**cost per ton, not speed** — Diocletian's Price Edict puts road freight at ~5.5x river,
which the HTML already uses for its food shed — but the router's grid is in **hours**,
and spending a cost ratio as a time ratio is exactly the dimensionless-number defect
§4 removed from this same function.

Nor is a river uniformly quicker. Measured through the planner's own `jpVesselDayKm`:

| hull | cruise | vs walking | cargo |
|---|---|---|---|
| Longship | 11 km/h | **2.18x** | 5 000 kg |
| River Barge | 2 km/h | **0.40x** | 30 000 kg |

Both are true and they answer different questions. `_civRiverTravelSpeed()` prices the
**fastest hull the water admits**, resolved through the planner's own eligibility test —
the §4 rule (the land surface speed is a SPEED from `JP_TERRAIN.land`, read through the
classifier the planner itself runs), applied to water. **No constant is invented**:
change a vessel's speed and the router follows. Falls back to 1 if the tables are
unreachable, so a missing planner can only ever cost the river its bonus.

### 6n.5 The planner's current was backwards one step in five

`_jpRiverCondition` inferred the current from **the route's own elevation profile** —
`(loss - gain) / km` over the chunk — which is a proxy for the water's direction rather
than the water's direction. Measured on real generated roads at seed 12345 / 512 px:
of 48 river steps where the proxy had an opinion, **10 (20.8%) got the current
backwards**.

**This had already been caught once, at one symptom.** v1.102 special-cased lakes out of
this function because a flat lake bed's DEM noise reads as a real current under exactly
this proxy. That was the same defect.

The sign now comes from the receiver tree and the magnitude from the channel's own
gradient (`net.slope * metersPerUnit() / mapWidthKm`, in m/km, against the existing
`JP_RIVER_GRAD_MILD = 8` / `JP_RIVER_GRAD_STRONG = 35` bands — no new thresholds).

**Assert it as a round trip.** The same 90.6 km chain walked both ways must report
opposite currents: measured **Strong Downstream** downstream and **Strong Upstream**
upstream. An elevation proxy cannot guarantee that; a receiver tree cannot get it wrong.

### 6n.6 `cat:'river'` was structurally unreachable for a real river

`_jpDeriveStages` classified **every** river cell as `cat:'land'` and used its `riv`
flag only to count crossings. So the entire vessel / `JP_TERRAIN.river` /
`JP_ROUTE.river` / `jpVesselDayKm` model applied to **lakes and to nothing else** — the
half of the request that had no implementation at all, not merely a weak one.

A step that both sits on a navigable channel and follows it
(`|align| >= CIV_RIVER_ALONG`) now derives as a river stage: measured **54 km of a
90.6 km chain**, against 0 before.

### 6n.7 One definition of which way the water runs

`_civRiverFlowField()` returns `{fx, fy, km2}` from `_riverNet.recv` and is read by
**both** the router and the planner — the shape this file keeps paying for, avoided
rather than repeated. Cached on a key naming every input it reads (`_fieldGen`,
`GW`/`GH`, extent, world mode). Covers **99.8%** of land channel cells. **A wrapped
receiver is ONE step, never a map width** (`dx -= round(dx/GW)*GW`) — the fourth site
where this has had to be fixed.

### 6n.8 What did NOT change

- `_civLandTimeEdgeCost` with the `river` argument omitted reproduces the pre-v2.62
  arithmetic **exactly**, asserted by diffing every non-river edge on a real world.
  `_jpRiverCondition` with `pts` omitted likewise gives the old elevation answer.
- The ford's own `fordK` still bands on **Strahler order** (`ord<=2` cheap, `ord<=4`
  mid) — pre-existing, and carrying the extent-dependence the gate just shed. Disclosed,
  not fixed: changing it re-baselines road geometry for no measured reason.
- Sea lanes keep their own edge cost (§5) untouched.

### 6n.9 The effect, measured end to end

A/B over three seeds at the app default, scoring **both** builds with the identical
navigability test applied to raw `flowField` so the comparison is fair:

| | v2.61 | v2.62 |
|---|---|---|
| road km running ALONG a navigable river | 251.0 | **405.0** (1.61x) |
| road km crossing one square-on | 224.2 | 217.2 (0.97x) |
| total road km | 16 590.7 | 16 472.7 |

Every seed improves (0.79→1.70%, 3.21→4.16%, 0.43→1.42% of total road length), crossings
stay flat — the fix does not buy river mileage by manufacturing bridges — and the network
is marginally shorter.

**Verification**: `tests/perf/probe_rivernav.js` (19 assertions; hard-errors on v2.61).

---

## 6m. A river is painted as water, in the lake's own colour (v2.61, DCC line only)

**Source**: `Cartalith v2.61 DCC test.html`. Owner, across four messages: *"Maybe we shouldn't
carve, maybe we should only paint the current line in the same color as the lakes. And make sure the
lines aren't broken bits"*, then *"Then at those pits should be lakes no?"*, then *"Maybe we should
revert the digging thing derived from the sculpt function and focus on coloring the river banks
accordingly."*

**Two of the four changes are RENDER-ONLY (§8.1) and two are not.** The lake-classification term and
the reverted digging pass both change generated values, so they belong in §8.2. Read all four before
porting either half.

### 6m.1 The renderer and the lake renderer disagreed about what water IS

A LAKE is **opaque**: `lakeWaterColor(gx,gy,T,vig)` RETURNS a colour and the caller writes it, at a
flat 0.95 shade, so a lake reads as a level surface. A RIVER was a translucent **Beer-Lambert tint**
over whatever land colour was underneath — `applyRiverWater` blended at alpha `sV*0.85`, and `sV` is
`amp*coverage` where `amp = min(1, 0.45+mag*0.7)` carries the **discharge magnitude**. So a small
river was drawn at **38% opacity** with hillshade showing through, and the drawn band measured a
flat-edged **2.5 km slab** with the terrain's own ridges visible across it.

**Port rule**: a river's alpha is its **coverage**, never its discharge. Emit coverage as its own
channel alongside intensity — the widest river over a pixel is not necessarily the brightest, so it
must max-combine independently.

**And bound the antialias band by the channel, not by a pixel.** `min(RIVER_EDGE_PX, halfWidth)`.
A flat one-pixel edge is correct in a refined tile and wrong on the coarse grid, where one "tile
pixel" IS a cell and the half-width is 0.8 of one: it capped the centreline at 80% and averaged 50%,
which is a tint again. Measured after: **57.3% of painted cells fully opaque, against 0% before.**

**One definition of the lake's colour.** It was written out verbatim in THREE places in the HTML
(`lakeColor`, `lakeColorSampled`, `renderBiomeTileRGBA`'s inline lake branch) and v2.61 adds a
fourth consumer. A port writing this from scratch gets that for free and should keep it that way.

### 6m.2 The banks come out of the walk that is already happening

The HTML already had a damp-bank/wetland/floodplain palette (`applyCoastRiverSDFv`'s river half,
v0.097) and it was unreachable in practice: gated on `state.viz.sdfRivers` (default 0) and fed by
`buildRiverSDF`, which v2.56 measured at **3078 ms** and which v1.29 lists among the per-tile passes
that ARE a seam because they have a spatial neighbourhood.

**Port rule**: the bank band is a distance from the water's edge and falls out of the same segment
stamp the water does — no distance transform, no extra pass, seam-free for the §6j/v2.40 reason (a
pixel depends only on its own world position and the world-wide polyline set). Its width is a
fraction of the river's own drawn half-width (`RIVER_BANK_K = 0.75`), so it needs no real-km or
per-cell term and inherits whichever floor already bound the water.

### 6m.3 A depression a river flows into is a LAKE — §8.2, this MOVES generated values

`buildWaterBodies` gated a pooled depression on **local rainfall**:

```
if(depth > lakeDepth && (!rain || rain[i] >= lakeRain)) out[i] = 2;    // lakeRain = 0.22
```

That is the right question for an **unfed hollow** (Death Valley, the Qattara Depression) and the
wrong one for a **terminal lake**, which exists precisely because a river delivers water from a
wetter catchment. Chad sits in the Sahel, Eyre and the Aral in deserts, the Dead Sea in a hyper-arid
basin — **every one fails a local-rainfall test and every one is a lake.**

The supply term was already computed: `flowField` is rainfall-weighted accumulated discharge (§6f /
v2.41), and the threshold is the engine's own definition of "a channel exists here",
`riverFlowThresh`. So the fix introduces **no constant of its own**:

```
if(depth > lakeDepth && (!rain || rain[i] >= lakeRain || (flow && flow[i] >= flowLake))) out[i] = 2;
```

Measured at seed 12345 / 1024 px / 800 km: lakes **15 904 → 17 725 cells** (4.58% → 5.11% of land).
The term is **monotone** — it can only ever ADD a lake — and that is asserted, not assumed. Keep it
an optional input to the primitive so its absence reproduces the old classification exactly.

### 6m.4 The sculpt-derived digging pass is REVERTED — §8.2

§6l's step 2c ran the Sculpt editor's own `enforceChannelDescent` over the post-carve network, taking
the drawn chains' climb from 12.55% to 4.33% at the cost of moving 0.74% of the map. The owner
reverted it: the decision it embodied was that the TERRAIN should be edited until the drawn river
reads right, and v2.61 takes the other branch — leave the ground alone, make the paint read as water,
and classify the pits as the lakes they are.

**The isolation is exact and is the thing to port against**: against v2.59 the battery reads `field`
**3273059064**, `temp` 2151860328, `rain` 1311039392, `flow` 1721724374 — **identical on both sides
in all five scenarios**, only `rgba` moving. So v2.61's terrain IS v2.59's byte for byte, which is
what proves step 2c was the whole of v2.60's field divergence. **A port that has not yet implemented
§6l's step 2c should not implement it at all.** `CHANNEL_DESCENT_CENTRE_HALFW` is gone with it.

### 6m.5 Remove a stem, never narrow one

§6l recorded that capping a river's WIDTH by its length broke the network **0 → 111 breaks**, because
a thin stub stops covering the diagonal elbow to the trunk it joins. v2.61 gets the effect that cap
was reaching for, safely, by removing whole stems instead:

- A stem is **attached** if its downstream end lands on another stem's cell, or its receiver leaves
  it into another stem, or it reaches the sea, a lake or the map edge — or some other stem ends on it.
- An **unattached** stem shorter than `2 × RIVER_MIN_HALF_CELLS` (i.e. shorter than it is drawn wide)
  is not drawn.

**Removing a stem that nothing joins and that joins nothing cannot disconnect anything, by
construction** — which is exactly why this is safe where narrowing was not. Measured: 455 of 1104
stems are isolated, but their **median length is 6.2 cells**, so culling every isolated stem would
delete 41% of the network; only the sub-width ones go. Breaks stay **0**.

### 6m.6 REFUTED — do not fill the carved trench

Measured before building, and rejected on the measurement:

- At the app default the terrain **carries no river-scale cross-section**. Wetted half-width at
  bed + 1 m is **below the 0.8-cell floor at 88.1% of vertices**, p50 **0**; the channel's own real
  half-width is p50 **0.054 cells** against a 3.1 km cell.
- At the depths where it is not zero, **13% (bed+20 m) to 37% (bed+50 m) saturate** into floodplain.
- At a 50 km region, filling to the trench's own bank reaches **38–62 cells** — a 3 km flood for a
  headwater.
- A backwater sweep along the centreline with **no cap** floods **650 839 of 670 720 cells**. An
  unbounded fill-to-a-level is a lake-maker, not a river renderer.

### 6m.7 Still open, and worth a port getting right from the start

**The carve and the renderer use two different widths**, from two different formulas:

| Strahler order | carve half-width | drawn half-width | ratio |
|---|---|---|---|
| 1 | 0.80 | 0.80 | 1.00 |
| 2 | **1.30** | 0.80 | **1.62×** |
| 3 | **1.80** | 0.80 | **2.24×** |

**24.6% of the drawn river at the app default sits in a trench wider than its water, and 100% at a
50 km region, up to 5.5×.** That is the "two functions answering one question" shape §7 records, and
it is left standing in the HTML rather than resolved by widening the paint to a size §6h/v2.49
already established is four times the Amazon. A port should derive both from one width.

Also still true: the two depression models (the routing surface's priority-flood vs
`buildWaterBodies`' own) disagree, so 12.55% of drawn chain steps climb. v2.61 stops editing the
ground to hide that and classifies the genuine terminal pits as lakes; the disagreement itself is
unresolved.

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

### 7.11 A spatial constant keyed on grid width is keyed on nothing
**The most-repeated defect in this whole span — eight occurrences, and the port will
make a ninth unless this is a review item.** v1.60 (relief frequency and the channel
threshold), v2.05 (the LOD detail ladder), v2.07 (channel width), v2.49 (the width
scale's shared floor), v2.51 (crater depth), v2.55 (the sub-cell relief floor), v2.57
(the plate-base blur radius) and v2.58 (the river spline's step, tolerance and meander
wavelength) are all the same mistake: a quantity that means something physical was
written as a fraction of `GW`, `GH` or a cell count, so it silently means a different
physical thing at every map extent.

The tell is that the constant survives a resolution change unharmed and breaks on an
**extent** change — which is why it keeps shipping. A 40 000 km world at 1024 px and an
800 km world at 1024 px have identical grids and a 50× difference in what one cell is.

Three rules, each of which this span had to learn separately:

**(a) Key it in real units and convert once.** `SUBCELL_RELIEF_M = 3.0` metres through
`metersPerUnit()`; `RIVER_MEANDER_WL_KM = 20` divided by the world's own `cellKm`. **The
converted value is a value, not a constant to copy** — v2.55 records the resulting
7.250e-3 as explicitly non-portable.

**(b) Anchor the conversion at the reference extent so the default is unchanged by
construction.** Both v2.55 and v2.58 pin their new real-unit constant so it reproduces
the old grid-keyed expression *exactly* at the app's own default (v2.58's `wl`
reproduces `GW/40` at 800 km at any resolution, asserted). That is what lets a
scale-correctness fix ship without a re-baseline, and it is worth designing for.

**(c) A scale factor's clamp must be checked for saturation, and a floor must not
inflate.** See 7.8 and 7.9 — those are the two ways a *correct* real-km conversion still
stops scaling.

### 7.12 An ordinal vocabulary cannot carry a threshold that must mean one thing on two maps
**Measured in v2.59 (§6k), and the port writes every consumer from scratch, so this is
cheap to get right once and expensive to retrofit.** Strahler order is a *rank*: it
counts how many levels of tributary a channel mask happens to resolve, so it is a
property of the river **and** of the threshold that detected it. In the HTML the
threshold is grid-relative, which makes order pleasingly resolution-stable (3/3/3 across
a 4× grid span) and **extent-dependent**: `order>=3` covers 0.32 % of the channel network
on an 800 km map and 4.10 % on a 40 000 km one, same seed, same resolution. It also spans
**123.7×** in real catchment inside its own top bucket and is **not monotone** — the
world's biggest river reads *below* its own navigability gate in 4 of 6 configurations,
and a change that re-routed 68.5 % of the land moved it not at all.

So: **an ordinal tier is fine for what it is — a tier.** The moment a rule needs to
compare two rivers, or to mean the same thing on two maps, key it on the continuous
physical quantity the engine already has (upstream catchment **area**, in km²), not on
the rank. The same caution applies to any other rank-shaped currency a port introduces:
ask what its buckets are a function of before a threshold is written against it.

---

### 7.13 A DETECTION threshold is not a CARVE threshold, and neither is a DISPLAY one

Three instances in this file, in three different consumers, of one constant answering
three different questions.

The HTML has a helper that eases its channel-initiation threshold on a coarse map:
`riverCoarseEase(mapWidthKm)`, capped at 16x, introduced in v1.101 to answer *"would a
stream EXIST here at all"* — because on a 40 000 km map one cell is 39 km and a real
tributary's catchment can never accumulate a threshold calibrated for an 800 km region.
That is the right answer to that question and the wrong answer to two others:

- **v2.57 split the CARVE off it.** `carveRiverValleys` cuts a real trench into `field`,
  and whether a stream exists says nothing about whether the grid can hold its VALLEY. At
  world extent the shared ease took the channel threshold 209.7 → 13.1 and cut **8.1x more
  trench**; since every carve point is floored just below sea level, a headwater reaching
  the coast flooded and left the land between two floodings as a one-cell bristle —
  **21.3% of the coastline** against 12.8% at 800 km and 1.5% with the carve off.
- **v2.72 split the DISPLAY off it.** Nor does it say whether the MAP can legibly carry the
  stream. The raster river renderer spent the full 16x ease on what it draws: **13.02% of
  the map painted as channel at 40 000 km against an 800 km region's 4.27%** — a 3x denser
  network, drawn at a uniform floored width (§6l) in an opaque lake colour (§6m), three
  individually-correct things compounding into a solid block. **The median stem is the same
  length in cells at both extents** (5.7 vs 6.4), so this is density, not size. A display
  bar keyed on each stem's own upstream drainage takes the channel share back to **3.58%**,
  just under the region map's own 4.19% — which is the target, not a tuned number.

**For a port the rule is the general one**: an easing factor belongs to the question it was
derived for. Before reusing one, name the question the new consumer is asking. Existence,
incision and legibility are three questions, and a single constant that answers all three
is answering at most one of them correctly. The same shape appears in §7.8 (a clamp shared
across a family of scale factors) and §7.12 (an ordinal vocabulary carrying a threshold);
this is the version where the sharing is across CONSUMERS rather than across scales.

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

Each row moves `field`, the tiles drawn from it, **or what the map renderer puts on
screen** — v2.58 and v2.72 are the two rows of the third kind, and they are listed here
rather than under §8.2 because each changes the visible map at scale even though neither
moves a generated value. **Eight are deliberate re-baselines** (v2.48, v2.50, v2.51, v2.57, v2.59, v2.60, v2.61,
and v2.49 above 12 800 km): a world generated from the same seed does not come back
the same, which is a decision to carry across deliberately, not a regression to
chase. **Nine of the eighteen rows already own full sections above** and appear
here only so the span reads complete — go to the section, not the row:
**v2.39 → §6e, v2.40 → §6f, v2.41 → §6g, v2.53 → §6h.1, v2.55 → §6h.2/§6h.3,
v2.57 → §6i, v2.58 → §6j, v2.59 → §6k, v2.60 → §6l, v2.61 → §6m.** (This list said *five* and omitted the two §6h rows
until 2026-09-17 — the same append-without-re-reading habit that produced §8's
own split.)

| Version | Change | Why it may still matter to the port |
|---|---|---|
| v2.72 | The raster river geometry is CUT where it wraps the antimeridian, and the render stops spending a DETECTION ease on what it DRAWS | **Moves no generated value — `hash_gen1.js` vs v2.71 is ALL IDENTICAL — and changes the drawn map materially at large extents, which is why it sits here beside v2.58 rather than in §8.2.** Two independent defects, from one screenshot of a 40 000 km world, and **both landed on the wrong one of two renderers**. **(1) THE STRAIGHT LINES ACROSS THE MAP ARE RIVERS.** The receiver tree wraps in X in world mode, so consecutive points of one stem sit at x≈W−0.5 and x≈0.5; the tile renderer stamps a band along every segment, so one wrapped step paints a river clean across the map. The HTML has had a splitter for exactly this since v1.29 (`splitRiverPolylines`) and had applied it at **three** of four sites — the stroked overlay, the GeoJSON export, and the carve (added v2.37). The fourth is the geometry the RASTER path draws from, **and the raster path is the one that draws at the default** (the stroked overlay's flag has been false since v2.29), so the renderer that splits is the one nobody was looking at. **The sharpest part for a port**: v2.58 unwrapped the same stem's LENGTH in that very loop (`dx -= round(dx/W)*W`) while still handing the drawer the wrapped points — **fixing a measurement OF a quantity is not fixing the thing the quantity describes**, and it is why this read as fixed for fourteen versions. Measured: wrapped segments **44 → 0**, worst step **1023 cells of 1024 → 1**, rows with their horizontal detail wiped out **52 → 0**. **(2) A DETECTION EASE IS NOT A DISPLAY THRESHOLD** — see **§7.13**, whose third instance this is. **WHY EVERY EXISTING HARNESS WAS BLIND, which a port's test plan should copy**: region mode cannot wrap and its ease is 1, so both fixes are no-ops there *by construction* — every river probe in the HTML runs region mode and its hash battery never sets world mode. v2.37 recorded that blind spot and it caught the carve, not the render. **A field hash would not have caught this either** (nothing generated moved) and would have gone red for innocent reasons a dozen times first. Assert the MECHANISM — no segment handed to the drawer may span more than half the map — which survives any later retune of colour, width or floor. **And a probe assertion of the author's own was wrong first**, comparing stems drawn against stems available and so crediting the new bar with a pre-existing v2.61 length cull: **comparing across two culls attributes one's work to the other.** |
| v2.69 | Tile refinement may no longer cross sea level; the settlement adapter reconstructs the surface the tile renders | **This changes what every refined tile near a coast contains, and it carries two rules a port needs before it writes its own LOD.** The HTML's own report was that a deep-zoom coastline walks away from the settlement drawn on it, and the decomposition is the useful part because **the obvious suspects were both innocent**: the channel-burn and feature-morphology passes are null at the app defaults, and the sub-cell crater registry runs and makes *exactly zero* difference. **(1) TWO RECONSTRUCTIONS OF ONE SURFACE.** The adapter that builds a town's water mask sampled the coarse field BILINEARLY while v2.47 had moved the tile's own reconstruction to Catmull-Rom — **17x the land-vs-sea disagreement** (2.89% against 0.17% with the filters matched). A port that keeps one reconstruction function cannot have this; a port that writes a second one inherits it. **(2) A ONE-SIDED TAPER.** `amplifyRegion` fades its detail band out going DOWN from the shelf and does nothing going UP, so a land pixel a hair above sea level took the full ±detailAmp/2 band and could be pushed under: **2.31% land→sea against 0.30% the other way, a 7.6 : 1 asymmetry that grew with depth (0.40% of pixels at z=2, 8.56% at z=8)**. The rule is v2.40's — refinement adds RESOLUTION and does not invent — and the land/sea boundary is a decision settlement placement, the water mask, the flooded-cell test and the road network are every one of them built against, so a renderer that moves it is drawing a different world from the one the rest of the app agreed on. **Clamp the DELTA, not the result**: clamping the height to sea level pins a coastal band to one value and makes a flat shelf, so cap the excursion TOWARD sea level at half the remaining headroom — no constant, never reaches the far side, and bit-identical wherever the detail is under half the headroom. **And the band is added in TWO places** (`amplifyRegion` and `addZoomDetail`); guarding one left a third of the drift, caught only because the control reported an identical drowned-pixel count on both sides. After: worst-level disagreement **8.56% → 0.35%**, **0 of 41 984** pixels drowned, seam delta still exactly 0. **A separate defect measured in the same pass and NOT fixed there**: the town derives its river width as `10+order*7` capped at 46 m — its own formula — while v2.49 gave the renderer a real hydraulic half-width, so the map draws a river far wider than the town was built around. **CORRECTED AND CLOSED, and the correction is the part a port needs**: comparing the town to the RAW hydraulic half-width gives 9.1x/18.8x/24.7x, but that is NOT what the map draws — the drawn width also carries v2.60's 0.8-cell connectivity floor, which is in GRID CELLS, so at the app default (1.5625 km per cell) **every Strahler order draws at 2500 m against a 1700 m settlement box**, the order distinction collapses entirely, and the true ratio is **~104x**. At 200 km the floor binds on order 1 only; at 50 km it never fires. **The owner's decision is to LEAVE IT, because both numbers are honest and a port should reproduce the same split rather than "fix" it**: the floor is a RESOLUTION statement — the centreline's position is known to +/-half a cell, so a narrower channel claims precision the data does not carry — and at 800 km one cell IS 1.5 km, so the map cannot locate a 30 m river. The town's channel is the physical truth; the band is the positional truth. Forcing agreement costs more than the disagreement in all three directions (narrowing re-baselines every tile and risks the 0 -> 111 broken-stem regression v2.60 measured; widening drowns the village; suppressing inside the town box helps only where a town is drawn). **A known limit of a coarse cell, not an open defect.** The centreline is shared; only the width is not. |
| v2.60 | A river stops being drawn as a chain of one-cell discs: a grid-cell width floor, a crossover profile, whole stems instead of fragments, and a finishing descent pass | **A deliberate re-baseline — `field`, `flow` and `rgba` all move — and a correctness constraint for any port that rasterises a D8 chain: see §6l.** `buildRiverNetwork` stamps a disc per channel cell at `halfW` floored to 0.5, so only the centre cell is painted and a diagonal step leaves a corner-touch gap: **884 of 1 305 main stems broke into parts, 3 554 breaks, 4 856 four-connected components for 134 rivers.** The carve had the right number (0.8, above a cell's circumradius) since v2.30 and it was never carried to the render stamp. Assert **4-connectivity** — 8-connectivity is free on a D8 chain and passes on the broken build. |
| v2.61 | A river is painted as WATER in the lake's own colour, at true coverage, with banks; a river-fed pit is a LAKE; v2.60's sculpt-derived digging pass is reverted | **Two halves, and they land in different sections — read §6m before porting either.** The PAINT half is render-only: a lake is opaque while a river was a translucent Beer-Lambert tint at alpha `sV*0.85`, and `sV` carries the DISCHARGE MAGNITUDE, so a small river drew at **38% opacity** through a flat **2.5 km** slab. **A river's alpha is its COVERAGE, never its discharge**, and the antialias band is bounded by the channel, not by a pixel. The other half MOVES GENERATED VALUES: `buildWaterBodies` gated a pooled depression on LOCAL RAINFALL, which is right for an unfed hollow and wrong for a TERMINAL lake — Chad, Eyre, the Aral and the Dead Sea all fail that test and all are lakes; gating on `flowField >= riverFlowThresh` instead adds no constant and takes lakes **15 904 -> 17 725 cells**, monotone. And v2.60's step 2c is REVERTED: `field` is byte-identical to **v2.59** in all five scenarios, which proves 2c was the whole of v2.60's divergence — **a port that has not implemented it should not**. |
| v2.59 | Integrated drainage becomes the default, and Strahler order is measured against upstream catchment area | **A deliberate re-baseline — `field` itself moves, because `carveRiverValleys()` cuts along the network integration changes. See §6k.** §6g's depression-filled routing was built in v2.41, measured, and shipped OFF; v2.59 turns it on. At the HTML's own default: land terminating in an interior pit **68.5 % → 0.0 %**, land draining to the sea **×1.63**, longest whole main stem **66 → 118 km**, biggest catchment at a stem mouth **5 078 → 92 815 km² (×18.3)**. `deltas` stays off; the save-compat guard still defaults `integrate` FALSE so a pre-v2.41 project reloads as the world it was. Isolated both ways against v2.58, so the re-baseline claim is checkable. The comparison that came with it is the part a port must carry: **Strahler order is NOT resolution-dependent here** (the channel threshold is keyed on the cell count — a port keying it on real km² would break that), **but it IS extent-dependent** (`order>=3` covers 0.32 % / 3.73 % / 4.10 % of channel at 800/8 000/40 000 km), **cannot rank inside its own top bucket** (123.7× in catchment) and **is not monotone** (the world's biggest river reads below its own navigability gate in 4 of 6 configs). Ruling: keep the ordinal tier, key the gates on catchment **area** — §7.12. Verified by `tests/perf/probe_riverorder.js` (18 assertions). |
| v2.58 | River SELECTION gains a scale term at last: whole main stems instead of fragments, an on-screen-pixel gate, and a spline that refines with zoom | **Not a re-baseline — `hash_gen1.js` vs v2.57 is ALL IDENTICAL — but a rendering contract a port will otherwise reimplement wrongly. See §6j.** The HTML's river *drawing* has had a scale term since v2.25; its river *selection* never had one, so a 50 km region and a 40 000 km world chose the same set of rivers. Four constraints a port must carry. (1) **Gate on screen pixels, not on raw zoom.** The settlement/road ladders this was asked to mirror are raw zoom scalars, which on a 40 000 km world put a hamlet on screen at a **28 571 km** view; `len * _z` needs no map-extent term because `_z` is screen-px-per-grid-cell in both camera conventions. (2) **A traced polyline is a FRAGMENT, and fragment length anti-correlates with importance** — measured ρ(length, drainage area) **0.207**, top-100-by-length ∩ top-100-by-flow **2%**. Assemble whole stems first (drainage area accumulated over the channel receiver graph by Kahn's algorithm, largest tributary kept at each confluence): ρ **0.207 → 0.963** against that same accumulation, which is what makes a length gate select whole rivers rather than fragments. **v2.59 correction: that 0.963 is length against upstream CHANNEL CELLS, not catchment AREA** (0.105–0.398 against the real catchment raster, worse than Strahler order's own 0.379–0.734) — the gate is a cartographic disclosure rule, never an importance ranking. See §6j's correction box and §6k. (3) **Accumulate on the CHANNEL tree, and make every length wrap-aware.** Ranking `net.recv` chains by `flowField` mixes two different trees (§6g's own finding) and produces stems dying after 5–10 steps at flow 163 405; a raw `hypot` across the antimeridian bills a full map width, which measured a longest "stem" of **41 097 km = 2 104 cells on a 2 048-cell grid** and, because the ladder RANKS on length, promoted every seam-crossing river to the top of it. (4) **The spline's own geometry was grid-keyed with no zoom or real-km term** — a 111 km control-point spacing and a 1 000 km meander wavelength at 40 000 km, constant at every zoom. **Eighth occurrence of the real-km defect** (§7.11). Also refuted and worth not re-chasing: **Töpfer & Pillewizer is wrong for hydrography** (predicts 49% flowline retention 1:24k→1:100k against USGS's measured 10–11%), **Strahler cannot carry the named tiers** (max order 4 at world extent, 2 at the default, against 8–12 for a real Amazon), and the ladder **saturates at LOD 5**, so LOD 7/8 is where the spline resolves rather than where selection happens. Verified by `tests/perf/probe_riverscale.js` (17 assertions). |
| v2.57 | The coastline stops being the plate polygon (`PLATE_BASE_BLUR_K` 0.35 → 0.18), and the river carve stops inheriting a detection ease | **Core simulation, two deliberate re-baselines, and the highest-leverage single constant in the height formula** — see **§6i**. The pure plate-Voronoi partition reproduced the land mask at IoU 0.813; the pathway is `baseField` (sd 0.2524), not `ageField` (0.0232). A port that reproduces `fillHeightRows` faithfully reproduces this faithfully too. |
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
| v2.73 | A village green as a distinct plaza KIND, and a footpath class whose source actually runs | **Not simulation — `hash_gen1.js` vs v2.72 is ALL IDENTICAL and the LAYOUT hash is identical too — and both halves carry a design finding a port should have before it writes either feature.** **(1) A GREEN IS A PLAZA WITH A DIFFERENT PURPOSE, not a smaller one.** `buildPlaza` already cuts a widened bay off the principal street, which is the geometry of a market place and of a village green alike; what separates them is **market right, not size**. A chartered town's plaza is commercial — stall encroachment shapes the frontages around it and a market CROSS stands in it as the legal marker of the right to trade — while a village's green is common land (grazing, assembly, the pond) and carries no cross, because there is no right to mark. So the change is a `kind` field and a branch on it: **the same three `addStreet` calls, in the same order, at the same widths**, which is what makes blocks/parcels/buildings bit-identical. The threshold is **the civic-building pass's own** chartered-town line (pop 1500), reused rather than invented a second time for the same distinction. **(2) THE FOOTPATH HAD A FREE SOURCE THAT NEVER RUNS, AND ONLY MEASURING IT CAUGHT THAT.** The alley-privatisation pass models a through-alley taken into the adjoining plots: the edge must leave the STREET graph — nothing may route a cart down it, and blocks, routing, metrics and the wall trace must all keep seeing it gone — while the foot traffic survives, which is what a snicket is. Recording the killed line instead of discarding it is four lines. **It is also unreachable on the profile the app generates**: that pass opens `if(!bias) return;` and the default rules table sets `deadEndBias` to **0**, with only the medina family's documented 0.16 floor ever setting it — **0 paths across six populations on the default profile**. Shipping that half alone would have been a feature that computes correctly and shows nothing on the map anyone actually makes. **The always-on source invents no geometry either**: a village's footpaths are the lines worn between the street and what people walk to daily, and the engine already places those — church, public wells, the pond — so any of them not already fronting a street has a path to the nearest one by definition. 2 paths on a village, 4 at pop 12 000, 7–17 m, bounded at both ends. **A path may not run through a house, and the first cut did**: **11.4% of sampled path length fell inside a building footprint** at pop 12 000, because the nearest street point to a feature is not always reachable in a straight line once the plots are built out — seven samples against the real footprints (the same call the parcel water test makes, §6s.4), and a rejected feature simply gets no path, which is the honest outcome. 0.0% after. **ORDER IS LOAD-BEARING and is what keeps the goldens still**: both sources run after the block/parcel/building passes, so neither can move one, and the model hash covers graph/blocks/parcels/buildings and **not** the plaza, the details or the paths — so a new model field cannot move a golden *by construction*, asserted directly rather than as "the goldens still pass". **The renderer rule is §7.9's, in a renderer**: each of the three new marks gates on **its own** size reaching about half a pixel, never a shared zoom tier — a 1.4 m path floored to 0.5 px reads as wide as the street beside it. The green is GROUND and has no gate; the pond and the path are person-sized and are disclosed only at their own scale. **Two probe assertions of the author's own were wrong, and both corrections generalise**: the map renderer derives its metres-to-pixels scale from the LIVE camera span, not from the transform it is handed, so a probe must drive the camera rather than the transform; and the `max(1, span)` floor that produces its real **`GW/1000` px/m ceiling** lives in that CONSUMER, not in the span function — on a 256-cell grid a 1.4 m path tops out at 0.36 px and is correctly never drawn on the main map, while every resolution the app ships at clears its gate. |
| v2.71 | Nothing the settlement layer draws may sit on water; woodland becomes an AREA | **Not simulation — `hash_gen1.js` vs v2.70 is ALL IDENTICAL — and the first half is a rule a port inherits whether or not it copies the feature.** The owner's own question was whether a new guidance layer was needed to keep a town off the water. It was not: the adapter has always built a **22 m mask of the real sea, lakes and river band**, and the engine's `isWater` predicate reads it. What it never did was reach the RENDERER — the model record handed to the drawing code is deliberately function-free, and the mask was not among the fields copied across. **The trap is the one a port will hit in the same place**: on the real-map-water path `buildSite` sets **`waterPoly = []` on purpose** (the map already paints the sea beneath the town, so the town must not paint a second one) — correct for FILLING, and it means a clip keyed on that polygon passes every synthetic fixture and **does nothing in the live app**; measured, **7 of 39 real towns carry an empty one**. Carry the mask instead, run-length encoded (**38 of 39**), and when a town carries BOTH — and real ones do, 54–113 land runs alongside 4/6/10/14-point polys built from a river centreline — **the mask must win**, or the clip trims the town against a sliver of one channel and leaves the rest on open sea. Push each run's corners through the layout transform, not a screen-space rect, or the clip is axis-aligned and wrong the moment the layout is rotated. A 22 m stair-step is already finer than the coarse grid's own ±half-cell uncertainty about where the coast is (§6l), so a traced contour would be many lines spent making a boundary smoother than the data under it. **WHY A CLIP RATHER THAN A GEOMETRY FIX, measured**: zero blocks, parcels, buildings or fringe parcels have a sample inside the water, and zero street CENTRELINES do either — what crosses is the street's **stroked WIDTH** (~8.4 m with its casing), and the engine's own water-crossing removal samples centrelines only. Insetting the streets would leave blocks, walls, buildings and every future feature to be fixed one at a time. Release the clip for bridges and fords, which are meant to span water, and give the agricultural fringe its own span — it must stay drawn BENEATH the water fill (§6t), and where no water fill exists that ordering stops being a backstop. **THE MEASUREMENT METHOD IS THE REUSABLE PART, because two cheaper ones both lied.** Overdrawn water as a SHARE of the water body reads **0.14%** and looks like antialiasing (the denominator is a 211 000-pixel sea while every wrong pixel is in the one place the eye goes); matching pixels against the built palette misses it **entirely**, because a street edge over water is antialiased and matches no entry. The honest test renders the town, renders it again with the settlement layer stripped, and diffs INSIDE the water. An eyeball check was also wrong in this pass — a stroked waterline appeared to run through the shore blocks and was the stroke straddling a street that stops exactly at it. Real world before → after: **36→0, 61→0, 24→0, 1→0, 4→4, 0→0**; the one real offender was a river-through town, where primaries are exempted from crossing removal as presumed bridges and that town had **0 bridges and no ford**. **Woodland**: the engine has scattered tree POINTS since v0.95, and no density of stipple makes a mass with an edge — so a wood is a polygon with a few trees inside it as texture. Two rules: the arable wins (assarting clears woodland FOR fields, so a wood is rejected where a field already sits, never the reverse), and a wood has **no street-crossing guard** even though the farmland generators do — a furlong with a road through it is not a furlong; a wood with a road through it is an ordinary wood. **And the search frame was wrong before it was measured, twice**: keyed on the built radius the candidate band ended inside the site box while the arable owned everything closer, so every surviving wood landed in the sliver between — a ring one wood thick. Loosening the farm clearance and dropping the street guard each changed the count by **zero**; only reframing the search on the site BOX did. |
| v2.70 | A flat limited-palette "Village map" look, added as one more step in the existing per-pixel style chain | **Not simulation — `hash_gen1.js` vs v2.69 is ALL IDENTICAL and the preset is opt-in — and the row is here for the SHAPE, which a port should copy rather than rediscover.** The HTML has exactly one function that colours land (`landColorCore`) and one that colours water (`seaColorCore`), each called by the main per-pixel loop, the LOD tile renderer and the flat bake, and each already ending in a chain of per-pixel style steps gated on their own flags. So a whole new map style costs **one flag, one step in each chain, one preset row and one button**, and every surface that draws a map picks it up for free — including the exported image, which is what makes an export match the screen without a second code path. A port whose colour logic is duplicated per renderer pays for each style N times instead of once; a port that keeps the single-function shape gets this property. **Two rules from the same pass**: quantise the LIT colour rather than the biome (it has already absorbed slope, aspect, material and shading, so quantising it keeps every distinction and removes only the gradient — which is the whole difference between a relief map and a drawn one), and turn the hillshade OFF in the recipe, because it is applied BEFORE the ramp and otherwise puts the gradient straight back. The water step is a REPLACEMENT applied at the very end of the sea function, not another mix into the colour, because the seabed grain, the bathymetric hillshade and the smooth depth ramp are all folded in by then. **And one trap that cost a red suite**: a new style flag must be declared in the state literal in the same edit that adds it to the managed-key list, or it does not survive a save — the HTML's own suite has guarded exactly that since v0.63. |
| v2.68 | The agricultural fringe is DRAWN — `field`/`pasture` polygons the generator has produced since v0.95 and that nothing ever rendered | **Not simulation — `hash_gen1.js` vs v2.67 is ALL IDENTICAL — and the row exists because a port that copies the reference faithfully inherits the defect.** `buildFarmland` has pushed `field`/`pasture` polygons into `model.details` since the layout engine was written; neither HTML map renderer reads `model.details` at all, and the City Viewer's detail pass branches on well/cross/crane/bollard/spoilheap/tree/dryingrack/logboom/fence with **no branch for either kind**, so every field fell through every `else if` and drew nothing, silently, for the life of the file. Measured: **62–86 field and 17–22 pasture polygons per pop-440 village.** A porter reading the reference sees a farmland generator, ports it, and never learns it was invisible — so **assert that each detail kind the generator can emit is reachable by the renderer**, derived from what a real town actually produces rather than from a hand-written list (v2.65's rule, and this is its third instance). **The other finding is a build a port should NOT make**: the obvious way to render furrow hatching with a per-parcel plough direction is a clipped hatch plus a stored bearing, and it is unnecessary — `stripFields` already cuts each grant at **5.8 × 91.5 m, every polygon a quad, aspect 15.8 : 1**, which is a *selion* (one plough-run), and a furlong is a bundle of parallel selions. Over 98 parcels: **10 distinct bearings, largest bundle 52 sharing one.** The texture and the orientation are already the geometry. Two renderer constraints go with it: the fringe is **ground**, so it draws under the water (it spans **1726 × 401 m against a 187 × 332 m built mass**), and the parcel outline must be gated on the parcel's **own** shortest edge — a 5.8 m strip drawn 0.6 px wide with a 0.5 px dark outline reads as the outline, so the whole fringe comes out one dark wash. **Disclosed and not fixed**: the strips do not tile (grants are spaced **28–40 m apart at 4–7 m wide**), nothing is generated within **330 m of the market**, and `FARM_SPEC` covers only `medieval` and `venus`, so every other profile generates no farmland at all. |
| v2.67 | The ward sets the plot grain; the SUBDIVISION is what it sets | **Not simulation — `hash_gen1.js` vs v2.66 is ALL IDENTICAL — and two findings a port should not rediscover: see §6s.** `buildParcels` decided plot grain with a hardcoded `dM<160` while `assignDistricts` answered the same question 130 lines later with seven wards, so a harbour, a suburb, an agrarian fringe and a riverside craft quarter all platted identically — with the harbour's own comment citing §1.1 #22 (*"deepest plots at quay; plot frontage narrowest of any family"*) as its reason for existing. **§6s.2 is the one to read before building the obvious version**: ward-driven DEPTH is inert, because **67.6% of parcels never reach `depthTarget`'s own 14 m floor** (the block waist binds, not the draw) and tripling `plotDepthVariance` 0.22 → 0.60 moves median depth **11.09 → 11.07 m** — which also explains why realised aspect is 1.09–1.74 against M-PAR-2's 1:3–1:10 band (a block-SIZING question). So what ships is the subdivision, whose two terms were driven by STREET AGE alone and a flat 0.4 halving chance. **Verify it as a per-ward SIGN against that ward's own baseline, never as a ranking across wards** — mean frontage per ward is not a function of pressure alone, because the blocks differ per ward. `wardGrain` defaults to 1 and **0 is bit-identical by construction**, so a port can keep either as a constant. **§6s.4 is the other one**: `buildParcels`' water rejection has sampled only the four CORNERS since v0.95, and a plot spanning a narrow channel has every corner dry — pre-existing, unreachable until the grain varied, and proven so by measuring v2.66 at `plotDepthVariance 0.60` (zero wet parcels). Any port of `buildParcels` inherits that hole. |
| v2.66 | The layout engine's own 22-parameter rules table gets a UI, saved per settlement TYPE | **Not simulation — `hash_gen1.js` vs v2.65 is ALL IDENTICAL — and one porting fact worth the row: see §6r.** The urban-morphology engine has exported `DEFAULT_RULES` (22 named generation parameters across street / parcels / settlement) plus `resolveRules`/`applyWildness`/`applyPlotChaos` since v0.95, and `generate()` reads `opts.rules` on its first lines — **and the host adapter has never set it**, so every town the HTML has ever drawn came out at the defaults. **A port that has ported `cityGen` already has the table**; what this adds host-side is where the values come from. Storage is `state.civTypeRules[kind]`, the accessor returns **null rather than an empty object** so an untouched type leaves `rules` absent and the engine takes its defaults branch — bit-identity is structural, exactly as §6o made it for the settlement parameters, **so a port with no such UI keeps the defaults and skips this entirely**. Two constraints if it does not: the per-settlement layout cache key must carry a fingerprint of the rules, or a stale town survives the edit for ever and silently; and `opts.faith`, `opts.civicStyle` and `opts.harbourDefence` are **three more live engine inputs the adapter still never sets** (§6r.4), so those three choices are made by the culture defaults on every town the HTML draws. **And read §6r.5 whatever the port decides about the UI**: exposing those parameters reached a NON-TERMINATING region of the engine's own documented range — `buildParcels` re-draws a frontage grant until one fits, unbounded, and the escape probability collapses with `frontageWidthVariance` (the 0.22 DEFAULT already expects ~28 571 retries; 0.12, the PoC's own 'Planned Grid' profile, effectively never escapes). **A retry-until-it-fits loop over a heavy-tailed draw is a hang waiting for a parameter change.** |
| v2.56 | The render prologue's eight derived fields (AO, crest, SVF, sun shadows, coast distance, coast/river/biome SDF) became generation-keyed | **Not simulation — bit-identical output — but a performance constraint any port that derives the same fields will meet.** The HTML rebuilt all eight on EVERY render with no cache key: measured at 1024px with their sliders on, **one render spent 9407 ms rebuilding them against 1219 ms of pixels**, and nothing they read had changed, so a slider drag cost ~12 s per step (11 720 ms -> 1 444 ms after the fix). Two rules worth carrying: **assemble the cache key at the CALL SITE, not inside the helper**, so a builder that gains a parameter has to name it where it is passed (`_fieldGen` covers field/flow/geoid, `_climGen` the biome raster's climate, and slider value / sun azimuth / sea level / which array the effective field resolved to are explicit); and **assert the EXACT rebuild set per input**, because a cache that never invalidates passes a "nothing was rebuilt" test perfectly. A field-generation bump rebuilding all eight is the contract, not a stampede. |
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

- ~~**Multi-ridge orogenic belts ("the Himalaya problem").** ... not in any shipped
  file, nothing was committed. Do not implement it from this note.~~ **CORRECTED
  2026-09-21 — BOTH claims below were false, checked directly against
  `Cartalith_v2.71_DCC_test.html` (the real DCC-line file, `VERSION='2.71'` at its
  own line 2682), owner-supplied to resolve exactly this contradiction with
  `§6b.2` below.** `buildOrogenyField` (line 3865) is real, present, shipped code —
  multi-sheet stacking (`nSheet`), `beltSpan=1.55*halfBelt` (line 3884, the exact
  figure `§6b.2` cites), per-sheet tapering toward the foreland, an orogenic-plateau
  fill and a foreland basin term. It runs whenever `state.tect.tectonicGraph` is on,
  which is not a rare manual override: any active World-Structure archetype sets it
  automatically (`state.tect.tectonicGraph=true`, line 3252, "T5: an active
  World-Structure archetype turns on structured orogeny"). `§6b.2` was right; this
  bullet was wrong, and the false claim carried no version or evidence to check it
  against — an unfalsifiable bullet is not a safer default than a wrong one.
- **Causal landmark generation** — not started in the HTML. `LANDMARK_GENERATION_SCOPE.md`
  in this repository is the port's own design, not a port target.
- ~~**Real-km-aware orogenic belt width.** `blurR` is in grid cells and never reads
  `mapWidthKm`... Known and open on the HTML side.~~ **CORRECTED 2026-09-21 —
  also false, same file.** `orogenyWidthScaleK(mapWidthKm)` (line 3472) exists,
  reads `mapWidthKm` directly, and is threaded into every `buildOrogenyField` call
  (lines 4006, 4370) as `widthK`. Its own doc comment names it the sixth sibling of
  `terrainDetailK`/`riverCoarseEase`/`lodDetailFreqK`/`riverWidthScaleK` — the
  identical family `§6b.2` and this file's `v2.36`/`v2.49` rows already document —
  and dates it v2.36, the same version `§6b.2` describes. Not open; fixed at the
  same time as the multi-sheet belt itself.

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
| `tests/perf/probe_coastgeom.js` | §6i — the coastline must stop being the plate polygon and the carve must stop combing it. Takes the pre-fix file as a control and measures each half against its own off-state inside one build |
| `tests/perf/probe_renderfields.js` | the v2.56 row in §8.2 — asserts the exact rebuild set per input, not merely that something was reused |
| `tests/perf/probe_geninfo.js` | the v2.54 row in §8.2 — verifies by REBUILDING, with a v2.53-shaped dump as the control that must fail |

Two harness traps the HTML hit that apply to any parity suite:

- **`tests/stub_head.js` returns `null` for any `getContext` but `'2d'`** — the headless
  suite has no WebGL2 at all, so it never exercises a GPU route and cannot see a
  GPU/CPU divergence.
- **Two suite assertions decide on an unpinned ambient seed** and are known-flaky (an
  SST warm/cold-cell check and a world-seam delta). If either goes red, re-run before
  believing it. The prescription on the HTML side is an aggregate over pinned seeds.
