# Cartalith routing system — audit, research and geographic model assessment

Audited against `Cartalith Gen1 v1.96.html`. Line numbers are from that file.

> **Status:** P0 shipped in **v1.97** (river direction + sea condition from the real current/wind
> fields + vessel sail polar). P1 U4+U5 shipped in **v1.98** (`edgeCost` hook + sea-lane geometry
> costed by round-trip sailing time). Remaining open: time-valued LAND cost (U6, deliberately
> deferred), route-class cost functions, gravity demand, seasonality, storm risk. Rows below are
> annotated where v1.97/v1.98 changed them. **v1.99** fixed a correctness bug this audit didn't
> cover — the downsampled routing grid (§K) plus `_civSmoothPath`'s Catmull-Rom smoothing could
> produce geometry crossing terrain the caller's mode declares forbidden — found via a live
> Journey-Planner audit against `travel-speeds.md`, not this document; see § M below and
> CHANGELOG.md for the full writeup.

This document is an **audit**, not an implementation. Nothing in it has been built. Section I
proposes a prioritised plan; it is a recommendation awaiting owner direction.

---

## A. Executive summary

**The land router is much better than the brief assumes. The maritime router is much worse.**

Cartalith already has a genuine cost-distance routing stack: a terrain/biome/river cost field, an
8-neighbour Dijkstra, and a three-pass network builder (MST → minimum-degree fill → shortcut
relief) with corridor reuse between passes. It is *not* the naive "one straight line per settlement
pair" system the brief warns about, and it already discovers passes and valleys emergently.

The maritime side is the opposite. Cartalith computes **real 2-D ocean-current and wind vector
fields** (`computeOceanCurrent`, `buildWind`, shipped v1.77–v1.82, with Ekman rotation, coastal
deflection and western intensification) — and **no routing code reads them.** Sea lanes are
pathfound over a cost grid where every ocean cell is the literal constant `1`
(line 20550), and the Journey Planner hardcodes every sea stage's route condition to `"Neutral"`
(line 18122). The `JP_ROUTE.sea` table has a `"Favorable Wind & Current": 1.40` entry that
**auto-derivation can never select** — it is reachable only by manual override.

So the answer to the brief's central maritime question — *does the system understand that the
fastest sea route is not the shortest?* — is **no, and it cannot**, because sea route geometry is
chosen by uniform-cost shortest-distance before any environmental term is consulted.

Three further structural findings:

1. **Two different objective functions.** World generation minimises an abstract dimensionless
   cost; the Journey Planner minimises time. The road network is therefore not built to minimise
   travel time, and the two systems cannot be checked against each other.
2. **A route-corridor field exists that the router never reads.** `buildRouteCorridors` (passes,
   fords, isthmuses — line 5759) feeds *settlement suitability* only (line 6328). The router
   rediscovers passes independently through slope cost.
3. **Route class is an output, not an input.** The six `CIV_WAY_TYPES` are assigned after routing
   from network centrality; there is no per-class cost function, so a footpath and a wagon road are
   pathfound identically.

Determinism is currently **sound**: the routing stack is RNG-free (verified — no `rng()` call
between lines 20300–21200), so identical inputs give identical networks.

---

## B. Current routing architecture

### B.1 Shared substrate

```
field / flowField / _riverNet.order / buildBiomeRaster() / currentWaterBodies()
        │
        ▼
_civRoutingGrid()            line 20381   downsample to ≤384px wide, share sc
        │
        ├──► _civLandCostGrid()    20386   buildTravelCost + lakes = Infinity
        ├──► _civWaterCostGrid()   20402   any water = 1, land = Infinity
        ├──► _civMixedCostGrid()   20441   land = slope+biome, water = _CIV_SEA_COST 0.6
        └──► _civEnhancedTravelCost() 20309  the full land model (below)
```

### B.2 The land cost function (`_civEnhancedTravelCost`, line 20309)

Multiplicative-with-additive-crossing hybrid, evaluated per cell:

| Term | Line | Form |
|---|---|---|
| base slope | 20331 | `c = 1 + 50·slope²` (isotropic) |
| mountain-pass saddle relief | 20334–20336 | slope penalty ×0.40 at a detected saddle |
| swamp / floodplain | 20340 | `×1.8` near-sea land with high flow |
| **river crossing** | 20342–20347 | `c += 8·mag·fordK`, `mag=min(1, ln(flow/thresh+1)/5)`, `fordK` = 0.35 (order ≤2) / 0.75 (≤4) / 1.0 |
| navigable river discount | 20349 | `×_civNavigableRiverDiscount(order)` — order ≥3 only (v1.95) |
| biome friction | 20352 | `×_civBiomeFriction(b)` — 1.0–1.6 |
| road reuse | 20354 | `×roadReuseK` (0.55) where `usageCount>0` |
| existing ways | 20844 | `×_CIV_EXISTING_WAY_DISCOUNT` (0.25) |
| settlement gravity | 20847 | capped local discount disc around each place (v0.73) |

### B.3 Network construction (`_civHierarchicalNetwork`, line 20802)

```
PASS 1  cost1 = enhanced(no usage)  → per-place roadDijkstra → Prim MST → usageCount++
PASS 2  cost2 = enhanced(usageCount) → minimum-degree fill by tier
        minDeg = {metropolis:5, capital:5, city:4, town:3, village:2, hamlet:1}   line 20808
PASS 3  shortcut relief: Floyd–Warshall over the settlement graph; add a direct edge when
        through-network distance > 1.7 × direct AND direct ≤ 2.5 × median edge      20904–20935
```

Corridor merging is real: PASS 2 re-costs against PASS 1's `usageCount`, so later edges are
attracted onto earlier ones. This is the mechanism the brief asks for in §7 and §9.

Then: `_civPreferSeaRoutes` (20665) drops a land way when a sea lane is cheaper on Diocletian
ratios and the land way is not the sole overland link; `_civConnectVillageAddons` attaches addon
villages via a batched Prim forest (v1.79); `_civNetworkMetrics` (21135) runs Brandes betweenness
to assign hierarchy → `CIV_WAY_TYPES`.

### B.4 Maritime construction (`_civMstRoutes`, line 20527)

```
cost[i] = (waterBody[i]===1) ? 1 : Infinity        line 20549–20550
        → roadDijkstra per port → Prim MST → sea lanes
```

That is the whole maritime cost model.

### B.5 Journey Planner (logistics, separate)

`_jpDeriveStages` (18036) **samples an already-drawn polyline**; it does not path-find. Per stage it
derives terrain, biome, infrastructure tier, and then:

- land: `routeCond` from settlement density / user override (line 18121)
- **water: `c.routeCond = "Neutral"` — hardcoded (line 18122)**

Speed = `ship.speed × hours × JP_TERRAIN × JP_ROUTE × JP_PACE × JP_INFRA × weather`, with
`JP_WATER_WINDOW` giving Open Sea a 22 h day vs Coastal 11 h (v1.43).

### B.6 The generation / logistics boundary

Clean, and **not duplicated** — but the two halves optimise different quantities (see §G).

---

## C. Land routing assessment

**Works**

- Genuine least-cost-path over a real cost surface, not Euclidean or straight-line.
- Passes are discovered emergently (explicit saddle detection *plus* slope cost).
- River crossings are already modelled by discharge and Strahler order — better than the brief
  assumes; the brief's §6 concern is largely already met.
- Existing infrastructure attracts new routes at two levels (in-run `usageCount`, cross-run
  `existingWays`).
- Network is hierarchical, not pairwise. The "six independent paths" failure mode does not occur.
- Settlement tier drives connectivity via `minDeg`.

**Deficient**

1. **Isotropic slope cost.** `1 + 50·slope²` is direction-independent. Real movement cost is
   anisotropic (uphill ≠ downhill ≠ cross-slope) — see §F-2. A route up a 20 % grade and down it
   cost the same.
2. **Cost is dimensionless, not time.** Nothing converts the field to hours, so the network is not
   a minimum-time network and cannot be validated against the Journey Planner.
3. **No physical / construction / economic separation.** The brief's §4 distinction does not exist:
   one scalar conflates "hard to walk", "hard to build", and "worth building". Consequence: no
   way to express *"a wagon road detours to avoid a grade a footpath would take"*.
4. **No demand model.** Connectivity comes from `minDeg` by tier, not from any interaction term.
   Two adjacent capitals and two distant hamlets are treated by rank alone.
5. **The corridor field is unused by the router** (line 5759 → 6328 only).
6. **Route class does not affect routing.** No per-class cost function.

---

## D. River routing assessment

**Works**

- Rivers are a first-class cost term in the land router (crossing penalty by order + discharge).
- `_civNavigableRiverDiscount(order)` gates navigability at Strahler order ≥3 — one shared curve
  since v1.95.
- `_jpModeForRoute('River Transport') → 'mixed'` gives barge journeys a genuine river-following
  bias with land portages allowed (line 19731).
- `JP_ROUTE.river` models upstream/downstream asymmetry (0.55 → 1.40) — a real anisotropy, and
  the *only* directional term anywhere in the routing stack.

**Deficient**

1. **`JP_ROUTE.river` is never auto-derived.** Like sea, river stages are hardcoded `"Neutral"`
   (line 18122). The system knows flow direction (`flowField`, D8 receivers) but never asks whether
   the traveller is going with it. This is the single cheapest realism win in the whole audit.
2. **No river-mode routing domain.** `'mixed'` prefers rivers; it cannot require them. Disclosed
   already in the code comment at 19726.
3. **No portage / rapids / depth model.** `JP_TERRAIN.river` has "River with Rapids" as a
   *user-selected* label, not a derived property of the reach.
4. **No explicit river ports as mode-transition nodes.**

---

## E. Maritime routing assessment

**This is the weakest subsystem and the clearest target.**

**Works**

- Land/ocean topology is correct; lakes correctly excluded from ocean lanes (line 20546–20549).
- Sea terrain class is genuinely geographic: distance-to-coast → Sheltered Bay / Coastal Waters /
  Open Sea (line 18061).
- `JP_WATER_WINDOW` correctly models that an open-water passage sails through the night while a
  coastal hull anchors (v1.43) — a real and well-grounded distinction.
- Vessel↔water feasibility is unified (`_jpVesselWaterBlock`, v1.23).
- Seasonal sea closure exists (`jpSeaClosure`, v1.52).

**Deficient**

1. **Sea lane geometry ignores currents and wind entirely** — cost is the constant `1`
   (line 20550). Shortest-distance-over-water *is* the model.
2. **The vector fields exist and are orphaned.** `computeOceanCurrent` (5240) and `buildWind`
   (5320) produce real `(u,v)` fields with Ekman rotation, coastal deflection, western
   intensification and hemisphere-correct sign. Consumers: SST feedback, wind-throw, three debug
   views, one particle animation. **Routing: none.**
3. **`JP_ROUTE.sea`'s favourable entries are unreachable by auto-derivation** (hardcoded
   `"Neutral"`). The table implies a model that is not wired.
4. **No directionality anywhere at sea.** A→B and B→A always cost the same, which is the defining
   error the brief's §12–13 target.
5. **No vessel-type route preference.** A coastal trader and an ocean-going hull path identically;
   the distinction exists only in *speed* tables afterwards.
6. **No waves / storm risk / depth** in routing. No multi-objective (fastest vs safest) support.

---

## F. Research findings

Format: CLAIM · SOURCE · EVIDENCE · IMPLICATION · CONFIDENCE.
Categories: **[P]** established physical principle · **[A]** useful approximation ·
**[H]** historical generalisation · **[S]** speculative worldbuilding abstraction.

**F-1 [P] Minimum-time ship routing is a genuinely different problem from minimum-distance, and
the standard solution is the isochrone method.**
Source: Hagiwara/Spaans-lineage isochrone literature; 3-D modified isochrone method
([ResearchGate](https://www.researchgate.net/publication/267621767_The_Ship-Routing_Optimization_Based_on_the_Three-Dimensional_Modified_Isochrone_Method)),
[Taylor & Francis review of isochrone improvements](https://www.tandfonline.com/doi/full/10.1080/17445302.2024.2329011),
[Cambridge *Journal of Navigation*](https://www.cambridge.org/core/journals/journal-of-navigation/article/ship-routing-optimisation-based-on-forecasted-weather-data-and-considering-safety-criteria/AAB546D746C2EC966F8C655760E3EF88).
Evidence: operational weather-routing systems expand *time fronts* through a time-varying
wind/wave/current field rather than minimising distance; commercial routing packages factor tides
and currents directly.
Implication: the brief's §13 premise is correct and mainstream. Cartalith's flat sea cost is a real
deficiency, not a stylistic choice. **Confidence: High.**

**F-2 [P/A] Terrain movement cost is anisotropic; isotropic slope² is a simplification.**
Source: Tobler's hiking function and the archaeological least-cost-path literature
([Herzog, *Archeologia e Calcolatori* 25](https://www.archcalc.cnr.it/indice/PDF25/12_Herzog.pdf),
[USC review](https://spatial.usc.edu/wp-content/uploads/2016/04/Schild-Alex.pdf)).
Evidence: Tobler's function is a *speed* function of signed slope with its maximum at a slight
downhill (≈ −2.86 %), not at zero; the review notes nearly all archaeological LCP studies compute
cost from slope and that anisotropic treatment is the more appropriate approach, while conceding it
is substantially more complex to implement.
Implication: Cartalith's `1 + 50·slope²` is defensible as an approximation but is **not** the
literature default. Adopting a Tobler-style *speed* function would also convert cost to time,
fixing two gaps at once. Caution flagged by the same sources: the function is widely misapplied —
it needs rise/run, not degrees or percent. **Confidence: High that it is anisotropic; Medium on how
much a fantasy-scale generator gains from modelling it.**

**F-3 [P] Sailing speed is a non-monotonic function of true wind angle; "wind behind = fastest" is
wrong.**
Source: polar diagrams / VPP
([Wikipedia: polar diagram (sailing)](https://en.wikipedia.org/wiki/Polar_diagram_(sailing)),
[Raymarine polar performance guide](https://www.raymarine.com/en-us/learning/online-guides/polar-performance-data)).
Evidence: boat speed is zero in the no-go zone (dead upwind), peaks on a beam or broad reach, and
*falls off* dead downwind; upwind progress requires tacking, which lengthens the sailed distance.
Implication: the brief's §14 caution is correct. A simple cosine bonus would be wrong in two
places (upwind and dead-downwind). A 3–5 point piecewise curve per vessel class is enough at
Cartalith's fidelity. **Confidence: High.**

**F-4 [A] MST + gravity-model demand is an established procedural road-network approach.**
Source: [arXiv 2001.08180, *A model for the generation of road networks*](https://arxiv.org/pdf/2001.08180);
[Gravity Model for Transportation Network Based on Optimal Expected Traffic](https://link.springer.com/chapter/10.1007/978-3-642-02466-5_49).
Evidence: iterative growth models trade off added construction distance against MST
reconfiguration; gravity-estimated flows have been used to generate networks validated against
real Mexican and Chinese highway systems.
Implication: Cartalith's MST + degree-fill + shortcut structure is **already in this family** and
is sound. A gravity term would refine *which* pairs justify an edge, replacing the current
tier-only `minDeg`. This is a refinement, not a correction. **Confidence: Medium-High.**

**F-5 [H] Pre-industrial roads follow passes, fords and valleys, and reuse existing alignments.**
Evidence: consistent with the LCP-archaeology corpus above, which exists precisely because
historical routes are reconstructable as least-cost paths.
Implication: Cartalith's emergent-pass behaviour and `_CIV_EXISTING_WAY_DISCOUNT` are
historically defensible. **Confidence: Medium-High.**

**F-6 [S] Speculative / flagged as weak.** The brief's §9 positive-feedback concern is real but
Cartalith already bounds it (discounts are fixed multipliers, not accumulating; PASS 3 adds relief
edges). The brief's §16 six optimisation modes are **not** justified by current requirements —
implementing all six would be over-engineering. The brief's §12 `V_eff = V_vessel + V_current +
V_wind` is dimensionally wrong as written: wind does not add to velocity, it drives it through the
sail polar. The defensible form is `V_eff = V_sail(vessel, TWS, TWA) + V_current`.
**Confidence: High that the brief's formulation needs this correction.**

---

## G. Gap analysis

| System | Current implementation | Desired behaviour | Gap | Severity |
|---|---|---|---|---|
| Land cost | `1+50·slope²` × biome × river, isotropic, dimensionless (20309) | anisotropic, time-valued | No direction; not time | **P1** |
| Slope | isotropic square law (20331) | Tobler-style signed-slope speed | Uphill = downhill | P2 |
| Passes | saddle detection + slope cost (20334) | emergent | **None — works** | — |
| Rivers (crossing) | order + discharge penalty (20342) | ford/bridge by width & bank | Works; no bank slope, no persistent bridges | P2 |
| Rivers (direction) | **derived from real signed gradient (v1.97 `_jpRiverCondition`)** | derive from flow vs heading | **Closed** | ✅ v1.97 |
| Roads as attractors | `usageCount` + `existingWays` (20354, 20844) | corridor convergence | **None — works** | — |
| Network build | MST + degree-fill + shortcut (20802) | hierarchical, merged | **None — works** | — |
| Settlement demand | `minDeg` by tier (20808) | gravity-weighted demand | Rank-only | P2 |
| Route corridors | computed (5759), used only for settlement placement (6328) | feed the router | **Orphaned field** | P2 |
| Route classes | 6 types, assigned post-hoc from centrality | per-class cost functions | Class is output not input | P2 |
| **Sea currents** | **time (v1.97) + lane GEOMETRY via round-trip cost (v1.98 `_civSeaTimeEdgeCost`)** | directional, time-varying | **Closed** (seasonality still P3) | ✅ v1.98 |
| **Wind** | **rig polar drives both reported time (v1.97) and lane geometry (v1.98)** | polar-curve vessel speed | **Closed** | ✅ v1.98 |
| Storms / waves | none in routing | risk term / safest mode | Complete | P3 |
| Seasonality | closures only (`jpSeaClosure`, seasonal passes) | time-varying fields | Geometry is season-invariant | P3 |
| Travel time | JP only, post-hoc on a fixed polyline (18036) | the routing objective | Two objective functions | **P1** |
| Mixed-mode | `'mixed'` grid + stage classification | explicit transition nodes | Partial; no port nodes | P2 |
| Determinism | RNG-free router | deterministic | **None — verified** | — |

---

## H. Recommended architecture

Do **not** rebuild. The land stack is sound; the correct move is to make the *existing*
environmental fields reachable by the *existing* router.

The one structural addition worth making is a **directional edge cost**. Every current cost
function is `cost(cell)`; currents, wind and river flow are all `cost(from → to)`. `roadDijkstra`
already evaluates `0.5·(cost[i]+cost[j])` per edge (line 3186), so it is one hook away from
supporting an optional `edgeCost(i, j, dx, dy)` callback. That single change unlocks P0 and P1
without touching the network builder, the MST, hierarchy, or determinism.

```
getTraversalCost(from, to, mode, vessel, season)
   ├── LAND   scalar field (today) → + optional signed-slope term
   ├── RIVER  scalar + flow·heading dot product
   └── SEA    |V_sail(vessel,TWS,TWA) + V_current| → time per edge
```

Keep world-generation cost and Journey-Planner time as one shared *time* quantity where feasible —
that removes the two-objective-function problem rather than papering over it.

---

## I. Implementation plan

**P0 — correctness (data that exists but is unreachable)** — ✅ **SHIPPED in v1.97** (items 1-2;
item 3 moves to P1 with U4/U5)
1. Derive `routeCond` for river stages from real flow direction vs travel heading, replacing the
   hardcoded `"Neutral"` (line 18122). Uses `flowField`, already present. Small, high realism gain.
2. Same for sea stages, sampling `currentOceanField()` / `currentWindField()` along the stage.
   This alone makes `JP_ROUTE.sea`'s favourable entries reachable and makes A→B ≠ B→A.
3. Add the `edgeCost` hook to `roadDijkstra` (default = today's behaviour, so bit-identical when
   unused).

**P1 — architecture** — ✅ **items 4 SHIPPED in v1.98**; item 5 (time-valued land cost) deliberately deferred
4. Directional sea cost: `V_eff = V_sail(vessel, TWS, TWA) + V_current`, minimise `Σ ds/|V_eff|`.
   Sea-lane MST then produces genuinely time-optimal, direction-dependent lanes.
5. Convert land cost to a time quantity (Tobler-style), unifying the two objective functions.

**P2 — realism**
6. Feed `buildRouteCorridors` into the router (it is already computed).
7. Per-route-class cost functions (foot / wagon / military / trade).
8. Gravity-weighted demand replacing tier-only `minDeg`.
9. Explicit port / river-port mode-transition nodes.

**P3 — advanced**
10. Seasonal field variants; storm/wave risk; multi-objective (fastest vs safest) — only if the
    owner wants a risk model. Not justified by current requirements.

---

## J. Test plan

Mapped to the brief's §28. A–C should **pass today** (regression guards); D–G are the real targets.

| Test | Setup | Expected | Status today |
|---|---|---|---|
| A Mountain | 2 settlements, ridge with one low pass | route finds the pass | expected pass |
| B River | large river, one favourable crossing | routes converge on it | expected pass |
| C Existing road | overlapping corridors | shared, not parallel | expected pass (v1.76/v1.79 measured) |
| D Sea current | adverse direct, favourable detour | fastest route is longer | ✅ **v1.98** — 36 better / 0 worse, up to 15.3% faster on paths up to 24% longer |
| E Wind | two routes, different TWA | vessel choice changes route | time ✅ v1.97 · lane geometry ✅ v1.98 (fixed reference rig) |
| F Season | same voyage, two seasons | routes may differ | **fails** (geometry season-invariant) |
| G Vessel type | coastal vs ocean hull | different preferred routes | time ✅ v1.97 (rig polar) · geometry still fails (U5) |
| H Mixed | land → river → sea → land | transitions at valid nodes | partial |

Add an asymmetry invariant: after P0, `time(A→B) ≠ time(B→A)` on any stage with a non-zero current
or river-flow component — the single sharpest check that directionality is real.

---

## K. Performance assessment

- Routing runs on the **downsampled ≤384px grid** (`_civRoutingGrid`, 20381), not the full raster —
  the brief's §26 concern is already handled.
- Cost is `O(P)` full-grid Dijkstras (one per settlement, ×2 passes). At ~40 settlements this was
  measured at 397 ms after v1.72 removed villages from the trunk build.
- A directional `edgeCost` callback adds a per-edge function call: expect a constant-factor
  increase, not a complexity change. Sampling two vector fields per edge is two array reads.
- The sea-lane MST is small (ports only) — the P1 change is cheap there.
- Recommend measuring before optimising, per this project's own standing discipline. Hierarchical /
  contraction techniques are **not** currently warranted.

## L. Risks and trade-offs — what to deliberately NOT implement

- **Do not implement all six optimisation modes** (§16). Nothing in the current product needs
  `SAFEST`/`MOST_RELIABLE`; each adds a scoring axis with no data behind it.
- **Do not adopt `V_eff = V_vessel + V_current + V_wind` literally** — see F-6. Wind acts through
  the sail polar.
- **Do not build a full ocean-physics model.** The existing current field is a documented heuristic
  (western intensification is a distance-to-coast proxy, not a solved Sverdrup model). Routing on it
  is legitimate; deepening it is out of scope.
- **Do not make routing non-deterministic.** The router is currently RNG-free; any parallel or
  approximate scheme must preserve that.
- **Watch the positive-feedback risk** (§9) if discount factors are ever made cumulative. They are
  currently fixed multipliers, which is why the network has not collapsed onto single trunks.
- **Anisotropic land cost is optional.** It is the literature default but the gain at fantasy map
  scale is unproven; P2, not P0.

---

## M. Addendum (v1.99): the downsampled grid + spline smoothing can produce invalid geometry

Not a finding of this audit — discovered afterward, via a live Journey-Planner audit run against
`docs/research/travel-speeds.md` rather than against this document. Recorded here because it sits
squarely in §K's territory (the downsampled `_civRoutingGrid`) and the next reader of this audit
should know about it before touching that code.

- §K notes the ≤384px downsample "handles the brief's §26 performance concern" — true, but it also
  means `_civLandCostGrid`/`_civWaterCostGrid` decide passability from ONE sampled full-res pixel
  per coarse cell, which can misclassify a coarse cell that contains a small patch of the forbidden
  terrain. Worse: this is not purely a downsampling artefact — `_civSmoothPath`'s Catmull-Rom
  reconstruction is not guaranteed to stay within its own control points' convex hull, so it can
  swing across a concave water/land boundary even with NO downsampling in effect.
- Fixed with a full-resolution repair pass (`_civTerrainValidTest`/`_civNearestValidPt`) inside
  `_civSmoothPath` itself, applied wherever a 'land'- or 'water'-mode path is built. 'mixed' mode
  (§F, §H — the general Route tool, sea lanes excluded) is exempt: crossing water there is already
  legitimate by design, so there is no forbidden terrain to repair against.
- One real subtlety worth flagging for anyone extending this: `_civDijkstraPath`'s own land-mode
  cost grid carries a pre-existing "an existing sea-lane way is a traversable ferry crossing" (v1.53)
  exception that none of the OTHER land-only cost-grid builders in §B (`_civHierarchicalNetwork`,
  `_civConnectPlaceToNetwork`, `_civConnectVillageAddons`) share — a naive version of this fix
  didn't account for that and "corrected" a legitimate ferry crossing back onto dry land. Any future
  change to where ferries are allowed needs to touch `_civTerrainValidTest`'s `allowSeaLanes` flag,
  not just the cost grid, or the two will drift the same way every duplicate-logic instance in this
  file's own CHANGELOG eventually does.
- Full writeup, measurements and test coverage: CHANGELOG.md's v1.99 entry.

---

## Sources

- [The Ship-Routing Optimization Based on the Three-Dimensional Modified Isochrone Method](https://www.researchgate.net/publication/267621767_The_Ship-Routing_Optimization_Based_on_the_Three-Dimensional_Modified_Isochrone_Method)
- [Strategies to improve the isochrone algorithm for ship voyage optimisation](https://www.tandfonline.com/doi/full/10.1080/17445302.2024.2329011)
- [Ship routing optimisation based on forecasted weather data and considering safety criteria — *Journal of Navigation*](https://www.cambridge.org/core/journals/journal-of-navigation/article/ship-routing-optimisation-based-on-forecasted-weather-data-and-considering-safety-criteria/AAB546D746C2EC966F8C655760E3EF88)
- [Herzog, *A review of case studies in archaeological least-cost analysis*](https://www.archcalc.cnr.it/indice/PDF25/12_Herzog.pdf)
- [Archaeological least cost path modeling (USC)](https://spatial.usc.edu/wp-content/uploads/2016/04/Schild-Alex.pdf)
- [Polar diagram (sailing) — Wikipedia](https://en.wikipedia.org/wiki/Polar_diagram_(sailing))
- [Raymarine — Polar Performance Data](https://www.raymarine.com/en-us/learning/online-guides/polar-performance-data)
- [A model for the generation of road networks (arXiv 2001.08180)](https://arxiv.org/pdf/2001.08180)
- [Gravity Model for Transportation Network Based on Optimal Expected Traffic](https://link.springer.com/chapter/10.1007/978-3-642-02466-5_49)
