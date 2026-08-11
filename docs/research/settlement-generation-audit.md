# Settlement Generation — Architectural Audit (v1.16 → v1.17)

> **Status: shipped.** Every phase of the refactor direction below (S1–S7) was implemented in
> `Cartalith Gen1 v1.17.html`. See the CHANGELOG v1.17 entry for the as-built record and
> verification numbers (engine 984/984, UME 852/852, hash vs v1.16 ALL IDENTICAL, smoke 216/216).

Audit of the full settlement pipeline — selection → classification → adapter → city-generation
engine → renderer — against the principle that **a settlement is the visible consequence of
terrain, hydrology, transport, resources, economy, political function, and historical growth; the
renderer never invents geography.** Every subsystem is classified as *Physically correct /
Simplified but acceptable / Decorative / Incorrect / Redundant / Missing*. Line numbers refer to
`Cartalith Gen1 v1.16.html` (22,680 lines) at the time of audit.

The pipeline under audit spans three script blocks:

```
World (block 1)                    Civ layer (block 2)                 UME engine (block 4)
terrain / hydrology / resources →  selection + traits + adapter     →  streets/blocks/walls/
field, flowField, _riverNet,       _civIterativeAutoWorld,             harbour/districts/buildings
currentResourcePotentials, …       _umPlaceContext, _umWaterCtx,       UME.cityGen(seed,opts)
                                   _umPrimaryPaths, _umModelFor     ←  model → _umDrawLayout
```

Historical context: v0.96–v1.05 already eliminated most *water/road* decorative geography (real
roads as the town's arterial skeleton v0.97, real river centerline + water mask v0.98, bilinear
coastlines v0.99, no-roads-over-water + market-on-land v1.00, settlements-never-in-water v1.01,
wall aspect/bank caps v1.03–v1.04). This audit establishes what REMAINS synthetic, inferred, or
absent.

---

## 1. Classification table

| # | Subsystem | Where (v1.16 lines) | Verdict |
|---|---|---|---|
| 1 | Settlement **selection** | `_civExtendedSuitability` (15575): river-confluence, navigable-Strahler, coast/estuary/lake, mineral (`currentResourcePotentials`), defensibility, agri soil×temp×rain, freshwater bonuses layered over `buildSettlementSuitability` (5011) | **Physically correct** |
| 2 | **Renderer** (`_umDrawLayout`/`Preview`, 16777/16867) | Draws ONLY model geometry (waterPoly, blocks, street graph, wall, buildings); colours/line-widths are the only renderer additions | **Physically correct** — no renderer-invented geography |
| 3 | Town **water** | `_umWaterCtx` (16580): real centerline from `traceRiverPolylines`, width from Strahler order (12–46 m), 22 m bilinear water mask + chamfer DT, lakes via `currentWaterBodies`, open-water bail | **Physically correct** |
| 4 | Town **roads** | `_umPrimaryPaths`/`_umRouteEnds`: real `civWays` resampled ~55 m; engine builds the town AROUND them (`buildPrimariesFromPaths`, 20570) | **Physically correct** |
| 5 | **Orientation** | orient forced 0 in the real-water frame; `_umTerrainOrient` river-PCA/sea-direction fallback (16450); engine rotates nothing — host applies `_umOrient` externally | **Physically correct** — random rotation does not exist anywhere |
| 6 | **Bridge** — single-river sites | `buildPrimaries` cost field: water passable only within 14 m of `site.bridgePt` (20540) → a span exists only because a primary road actually crosses there | **Physically correct** |
| 7 | **Historical growth** | Epoch stamps on streets→parcels→buildings; age drives frontage subdivision + burgage wing-infill (21961/22142/22176); `wallGenerations`: superseded wall persists in `wallState.history`, its land arc becomes a ringroad (21320, Ringstrasse analogy); ribbon suburbs along approach roads (21226) | **Simplified but acceptable** — the requested "historical layers" mechanism already exists and is sound |
| 8 | Age / harbour-scale inference | `_umInferAge` = log(pop), clamped 30–1000 (16404); `_umHarbourScale` = (pop/3000)^0.4, clamped 0.6–3 (16426) | Simplified but acceptable (no real founding-date signal exists) |
| 9 | Tier (`kind`) assignment | Suitability-rank thresholds + betweenness promotion/demotion (17700–17751) | Simplified but acceptable |
| 10 | **Districts** | `assignDistricts` (22021): purely radial distance zoning — market / craftriver / harbour / burgher / artisan / suburb / agrarian; `buildMarkets` specialised squares by pop threshold (20870). No granary/mill/smelter/wharf concept exists anywhere in the block | Simplified as base zoning; **MISSING the economic dimension** |
| 11 | **Synthetic relief** | `buildSite` invents **3 randomly-placed Gaussian hills** + inland rise + water-terrace dip (20353–20419); this fake `height()`/`slope()` drives street route costs (20537), market siting (20518), `bridgePt` = "flattest approach" (20423), `grow()`'s slope>0.34 street rejection (21253), and `terrainSuitability` (20482) | **Decorative / Incorrect** — the single largest remaining piece of invented geography. The engine's streets avoid hills that don't exist while ignoring the ones that do. |
| 12 | **Wall existence** | `_umInferWalls` (16412): tier rank ≥ 2 OR `fortified` trait | **Incorrect in part** — `fortress` kind (rank 1) is unwalled by default; existence keyed on settlement size, never on threat, function, or wealth |
| 13 | **Wall material** | Engine knows only `'curtain'` and `'bastioned'` (`applyStarFort` ≥2500 pop + fortified, 21613); no ditch/palisade/hedge/none ladder | **MISSING** — every walled village gets the same stone curtain a city gets |
| 14 | **Wall geometry** | `builtMassHull` convex hull ×1.10 + v1.03 aspect cap + v1.04 bank-walk fallback (21407–21604); follows the real water bank, but never terrain (impossible while the engine has no real relief, see #11) | Simplified; **Incorrect vs. the target spec** (terrain-blind: no ridge/spur/cliff/steep-slope exploitation) |
| 15 | **Bridges** — riverthrough sites | `addRiverBridges` (20849): **two automatic spans at fixed river fractions**, unconditioned on any road crossing | **Decorative** |
| 16 | **Harbours** | `buildHarbour` (20733): fires for EVERY water site (`site.harbour.pt` set for all water kinds); **zero depth/navigability checks**; no cliff-shore exclusion. Geometry itself does follow the (real) shoreline with land-clearance guards | **Incorrect in part** — a Strahler-2 stream can sprout quay+piers; a cliff coast auto-docks |
| 17 | Detail objects | `buildDetails` (22415): wells (network-degree placed), market cross, cranes/bollards (quay), garden trees, fences, strip-fields/orchards | Decorative by design (acknowledged props; low stakes; placement is rule-based, not data-false) |
| 18 | **Functional classification** | `p.specialisation` is **NEVER assigned by auto-populate** — the single write site in the whole file is the manual editor dropdown (13534). Generated worlds have no per-settlement economy; the v1.16 Economy pages read defensive defaults | **MISSING** — the biggest single gap vs. the target spec |
| 19 | **Economy → engine** | specialisation / resources / biome / climate: none reach `UME.cityGen` (`_umPlaceContext` return, 16688) | **MISSING** — a mining town and a fishing town of equal population on similar water get IDENTICAL layouts |
| 20 | Traits | `port` = real coastal test; but `mining` = elevation>0.55 and `fortified` = elevation>0.62 (17716) — raw-altitude proxies, despite `currentResourcePotentials` and real defensibility being computed and used by SELECTION in the same function | Simplified-but-crude — contradicts its own available data |
| 21 | Culture | Hardcoded `'medieval'` (16694) though per-faction `civFactionCulture` has existed since v1.07 | Decorative |
| 22 | Model **cache key** | `_umCacheKey` water fingerprint = wet-cell COUNT + riverPath vertex COUNT + rounded width (16716) — two different coastlines with matching counts collide on a cached layout | **Incorrect** (latent wrong-layout bug) |
| 23 | Synthetic water fallback | `buildSite`'s sinusoid rivers / Gaussian-bay coasts (20365–20396) | Acceptable — reachable only when no `opts.water` is supplied, i.e. the headless UME suite; in-browser the real water context always wins |
| 24 | `_civSnapPlacesToLand` (15459) | Water revalidation with connected-road endpoint re-anchoring | Physically correct |
| 25 | Sub-cell terrain availability | `field` (GW×GH) is the finest stored terrain; `amplifyRegion` (8156) synthesizes deterministic sub-cell detail; `_umWaterCtx` already samples bilinearly per 22 m cell for exactly this reason | Constraint, not a defect — any town-box terrain must come from bilinear/amplified sampling |

No subsystem was found to be *Redundant* — the v0.95–v1.05 porting work left no parallel duplicate
systems (the one prior instance, dual river renderers, was fixed in v1.14).

## 2. Architectural weaknesses, in priority order

1. **The engine models water but not land.** v0.98 made the town's water real; nothing ever did
   the same for relief. Every terrain-driven decision inside the 1700×1250 m town box — street
   route costs, market position, bridge point, steep-street rejection, building suitability —
   consumes three seeded random Gaussian bumps. Real slopes, ridges, and cliffs are invisible to
   the layout. This violates "the renderer never invents geography" at the *generator* level: the
   renderer faithfully draws a model that was itself shaped by invented terrain.
2. **No generated settlement has a function.** Auto-populate computes rich resource/coast/river
   context to *place* settlements, then throws it away: `specialisation` stays undefined. The
   owner's "classify by function, derive morphology from classification" has no data to act on.
3. **Walls answer "how big is the town", not "why fortify".** Existence = tier rank (with the
   fortress-kind paradoxically unwalled); geometry = street-mass hull; material = one stone
   curtain. No ditch/palisade tier for villages, no terrain exploitation, no unwalled prosperous
   towns.
4. **Validity rules are missing at the water's edge.** Harbours require no navigable water and
   ignore shore steepness; through-river towns get two free bridges no road asked for. (Single-
   river bridges, by contrast, are already road-justified — the correct pattern exists in-engine.)
5. **Cheap correctness bugs:** the model-cache water fingerprint can collide (wrong cached layout
   for a different coastline); the `mining` trait keys on altitude rather than the ore fields the
   same function already computes.
6. **What does NOT need rework** (explicitly, to prevent churn): site selection, town
   water/road consumption, orientation, renderer discipline, the historical-growth machinery
   (epochs/wall generations/ribbon suburbs), and snap-to-land. These already satisfy their part of
   the spec; the refactor must feed them better inputs, not replace them.

## 3. Refactor direction (implemented as v1.17)

The remedy applies the proven v0.98 pattern — package real map data into the layout's local box
frame, key every new engine capability on the presence of a new `opts.*`, leave the synthetic path
byte-identical for the headless suite — to the remaining gaps:

- **S1 Site Profile**: one cached per-settlement object assembled entirely from existing engine
  primitives (`settlementSeedInfo` is the prototype; slope/aspect/curvature, river
  order/width/confluence, flood field, coast distance, resources, biome, climate, defensibility,
  buildable area). No rendering before this profile exists. Plus the cache-key fix.
- **S2 Functional classification**: auto-populate derives `p.specialisation` from the profile;
  the `mining` trait re-keys onto real resource potentials.
- **S3 Real terrain into the engine**: `_umTerrainCtx` (the land twin of `_umWaterCtx`) replaces
  the Gaussian hills whenever present; all downstream consumers (street costs, market, bridge
  point, slope rejection, building suitability) need zero changes — they already read
  `site.height/slope`.
- **S4 Wall justification + terrain-aware ring**: a {none, ditch, palisade, stone, bastioned}
  spec derived from tier+function+threat+age+wealth; ring vertices deflect (bounded) toward
  steep-slope crests and banks.
- **S5 Bridge/harbour validity**: riverthrough bridges only at real road crossings (ford
  fallback); harbours require navigability (sea/lake or Strahler ≥3) and a non-cliff shore.
- **S6 Economic districts**: `opts.economy` drives function overrides on the radial zoning
  (ore-yards, drying yards, sawyards, granary+mill, warehouse rows) and per-economy details.
- **S7 Diagnostic overlays**: a site-profile raster view + a settlement-diagnostics vector
  overlay (footprint, wall spec, bridge/ford/harbour validity) for development validation.

Known non-goals, documented rather than fabricated: valley WIDTH has no engine primitive (the
flood field is the floodplain proxy); full per-culture morphology (UME has two profiles) stays
future work; wall-ring terrain-following is bounded hull deflection, not a ridge-tracing solver.
