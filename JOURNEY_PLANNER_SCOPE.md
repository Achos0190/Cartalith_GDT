# JOURNEY_PLANNER_SCOPE.md — the `jp*` route planner: engine milestones, census, integration

**What this is:** the definition of the Journey Planner's engine port — six
milestones, the 74-function census that makes "every real
`jp*` function" checkable, the five integration steps that turn the engine into
a feature, and the design findings each pass made. **What it is not:** status —
that is `cartalith-native/docs/STATUS.md`'s "Journey Planner" group (JP-1…
JP-PARTY), and open work is routed from `OUTSTANDING_WORK.md`. Nor is it the
planner's layout: that is `JOURNEY_PLANNER_SPEC.md`. The Travel Library that
feeds the party form is `TRAVEL_LIBRARY_SPEC.md`.

The Journey Planner (`jp*`/`_jp*`, `reference/Cartalith Gen1 v2.10.html` lines
~17300-20400; every line number here resolves in v2.10) is Phase 2's largest
sub-phase: `ECONOMY_SCOPE.md` and `ROADMAP.md` sized it at ~70 functions, and the
exact count is 74 (see the census).

## Build order — the milestone numbers are identifiers, not an order

The numbers were assigned by `ECONOMY_SCOPE.md`'s categorisation before anyone
read the code, and milestone 3's reading then found a real dependency
inversion: its two stage calculators need milestone 4's mass model. The numbers
are **not** renumbered — `STATUS.md`, the retired `CHANGELOG.md`, commit messages
and code comments cite them by name.

| Dependency order | Milestone | Why this position |
|---|---|---|
| 1 | **1**, and the self-contained parts of **2** and **3** | No plan, route or mass model needed |
| 2 | **4** — consumption/resupply | Unblocks milestone 3's two calculators and part of milestone 2; `jp_fmt_kg` comes with it |
| 3 | **3 (tail) + 2 (partial)** | `jp_calc_land`/`jp_calc_water` and `_jpBestLandTransportForStage` need only milestone 4 |
| 4 | **5** — route/stage derivation | Needs 2-4; the largest single milestone |
| 5 | **6** — verdict/reporting | Needs 5's plan output to verify against |
| 6 | **2 (remainder)** | `jpAutoPickTransport` and `_jpBestPackageForStage` need 5's plan/stage shapes |
| — | **`_jpRerouteForMode`** | Its whole body is the Route tool's pathfinder, which `UNIFIED_TOOL_PLAN.md` milestone D ported; what remained was small and a UI action, so it never had a milestone number |

## Milestone 1 — physical-modeling primitives + seasonal/closure cluster

No dependency on a plan/route/vessel context and none on another `jp*` function.

- **Physical-modeling primitives**: `jp_fatigue`, `jp_load_penalty`,
  `jp_surface_gain`, `jp_can_use_wheels`, with `JP_LOAD_INVALID_RATIO` (v1.63's
  "above this ratio the stage is infeasible, not just slow").
- **Seasonal/closure logic**: `jp_season_at`, `jp_rest_days`,
  `jp_seasonal_closure`, `jp_sea_closure` — the reference's v1.52 block (the rest
  vs travel-day split, season drift on long journeys, sea-lane and
  mountain-pass winter closure). Real, sourced fixes per the reference's own
  comments; ported faithfully, not redesigned.

Small, pure, branch-complete functions with no RNG or iteration-order risk, so
unit tests over every band, gate and wrap-around case suffice — the precedent
`civ_resource_trade_balance`/`civ_culture_terrain_fit` set.

## Milestone 2 — transport mode selection

Ten functions: `jpBestAnimalForContext`, `jpPickSpeciesForRoute`,
`jpResolveMount`, `jpVesselMatrix`, `_jpVesselFits`, `_jpAutoStageVessel` (all
self-contained given a caller-supplied stage list), plus four that are not:
`jpAutoPickTransport` (17814) and `jpAutoPickVessel` (18012) open with
`_jpEnsurePlan` + `_jpDeriveStages` (milestone 5); `_jpBestLandTransportForStage`
(18053) calls `jpCalcLand` (milestone 3, which needs 4); `_jpBestPackageForStage`
(18080) takes an `eff` plan shaped by milestone 5. Porting those early would have
meant inventing the shape of data two unbuilt milestones had not defined — so
they waited (build order above). With them come the data (`JP_ANIMALS`,
`JP_ANIMAL_TERRAIN_OVERRIDE`, `JP_TERRAIN`'s land/river/sea tables, `JP_SHIPS`,
`JP_VESSEL_PREFERENCE`, `JP_WATER_WINDOW`) and helpers (`jpAnimalTerrainMod`,
`jpWaterWindow`, `_jpVesselWaterBlock`, `jpVesselDayKm`).

- **Biome mapping, resolved by the reference itself.** The worry was that
  `biome.desertLike`/`bestAnimals` would not map onto this port's `u8`
  `classify_biome`. They need no new table: `jpLegacyBiomeOf` (18310) already
  maps `classifyBiome`'s keys (`ice`/`tundra`/`boreal`/…/`tropWet`) onto
  `JP_BIOMES`' legacy names, and those keys are exactly the climate scheme
  `classify_biome` golden-verifies. Ported as `jp_biome_key(biome_id, temp_c)`,
  a direct transcription of that fallback, including `desert` → `T<10 ? "Cold
  Desert / Badlands" : "Hot Desert"`. Water biomes fall through to the
  reference's own default (`"Temperate Forest"`).
- **The bottleneck veto** (`jpPickSpeciesForRoute`, v1.50): a mostly-plains route
  with one real mountain-pass stretch switches the whole route's animal,
  hand-verified against `JP_BOTTLENECK_PENALTY`/`JP_BOTTLENECK_MIN_SHARE`.
- **`jp_auto_pick_transport`'s HTML hint strings are not ported** — presentation
  is Godot's (`ARCHITECTURE.md`). `JpAutoTransport` is an enum over the
  reference's six outcomes (no land stages / not a land mode / walking within
  capacity / walking overloaded / mount picked / baggage train), carrying every
  number the hints print, including the `fodderInfeasible` divergence case. It
  mutates the plan exactly as the reference mutates `jn.plan`, and needed
  `plan.autoPromote` (`JpPlan::auto_promote`), the last `_jpEnsurePlan` default.
- **`jp_best_package_for_stage` needs only a stage and an `eff` plan** — no
  derived route — the same finding milestone 4 made about
  `_jpBestLandTransportForStage`.

## Milestone 3 — physical travel cost

Eleven functions. Seven are self-contained given a caller-supplied party/leg
summary: `jp_train_pace`, `jp_sail_factor`, `jp_wx_weighted`,
`jp_weather_factor`, `jp_column_length_km`, `jp_column_factor`,
`jp_journey_cost`, with their data (`JP_TRAIN_PACE`, the `JP_RIG`/`JP_SHIP_RIG`
sail polars, `JP_WEATHER`, `JP_ANIMAL_WEATHER_OVERRIDE`, `JP_FILES_BY_TERRAIN`
and the column-spacing constants, `JP_COST_*`) and one shared `JpParty`. Two
(`jp_water_window`, `jp_animal_terrain_mod`) belong to milestone 2, which needed
them first.

**The last two are the ordering inversion.** `jpCalcLand` (18912) calls
`jpCapacity`, `jpForaging`, `jpAssessResupply` and `_jpDesertTierForGap`;
`jpCalcWater` (19124) calls `jpAssessResupply` and `jpHumanWaterRate` — every one
on milestone 4's list, and not thin shims: `jpCapacity` is the whole
seasonal-physiology / draft-shortfall / saddlebag mass model, and `jpForaging`
reaches through `_jpWildlifeForageMod` into the world's wildlife richness.

- **`JP_BIOMES`' weather distributions** (12 biomes × 4 seasons × 5 conditions)
  are ported here, beside the two functions that read them.
- **`jp_journey_cost` is portable** — the reference calls it "pure over the plan
  object — no globals, no DOM", and it holds. It reads a small per-leg summary
  (`cat`/`st.km`/`days`/`crew`/`blocked`), one `claimedFrac` per stage, the trip
  totals and the party. Ported with a `JourneyLeg` narrowed to exactly those
  fields, which is also the shape the stage calculators produce.

## Milestone 4 — consumption/resupply

Thirteen functions: `jp_human_water_rate`, `jp_human_water_carry_days`,
`jp_animal_water_carry_days`, `jp_desert_tier_for_gap`,
`jp_consumption_factors`, `jp_foraging`, `jp_capacity`, `jp_assess_resupply`,
`jp_world_mean_richness`, `jp_wildlife_forage_mod`, `jp_resupply_reach`,
`jp_drinking_coarse_ease`, `jp_stage_dry_km` — plus `jp_water_reach_cells`
(milestone 5's list, but `_jpStageDryKm` calls it). With them, the data
milestones 2 and 3 left out: `JP_BIOMES`' `water`/`forage`/`waterForage`/
`grazing` columns (milestone 2's two-field lookup now delegates to the one full
biome record), the four seasonal/grazing tables, `JP_PACE`/`JP_INFRA`/
`JP_ROUTE`/`JP_GROUP_CLASSES`/`JP_LAND_TRANSPORTS`/`JP_DESERT_WATER`, and the
vehicle and ration constants.

Four functions other milestones own are built here, because this is where they
unblock: `jp_calc_land`/`jp_calc_water` (milestone 3), `jp_fmt_kg` (milestone
6), and `_jp_best_land_transport_for_stage` (milestone 2 — its `eff` is only a
plan with per-stage overrides merged in, so `jp_calc_land` was all it needed).

- **The stage calculators return `Result<_, JpBlocked>`** rather than the
  reference's `{blocked:"…"}` sentinel, so a blocked stage cannot be read as a
  computed one by accident.
- **Their `formula` strings are not ported** — prose is presentation. The
  structured chain *is* engine fact and crosses the boundary as `JpTerm` /
  `JpLandCalc::trace` / `JpWaterCalc::trace` (see "Engine additions").
- **Wildlife richness.** `richness` is a per-ecoregion **species count**
  (`assignWildlife`'s roster clipped by `regionRichness`), a different quantity
  from `build_npp`/`build_carrying_capacity`, which are only its inputs. The
  engine takes it as an input rather than reaching into world state:
  `jp_wildlife_forage_mod(region_richness, world_mean_richness)` and
  `jp_world_mean_richness` are pure ports; `JpStage` carries the finished
  multiplier; and `jp_plan` takes a closure in `_jpWildlifeForageMod(mx, my)`'s
  position. The boundary supplies a real closure over
  `sample_bridge::WildlifeCache` (`cartalith-civ/src/wildlife.rs`'s ported
  ecoregion model); `1.0` is only the fallback for the reference's own no-data
  cases — no civilisation layer, no region under the stage midpoint, a world
  mean of zero — which is also the reference's answer there.
- One workspace edge: `cartalith-civ` → `cartalith-terrain`, for
  `river_coarse_ease`, which `jp_stage_dry_km` divides back out to substitute
  JP's own uncapped ease.

## Milestone 5 — route/stage derivation, in three sub-milestones

The largest milestone, and it falls into three parts, each impossible before the
previous one:

- **5a — world sampling.** Everything that turns the world into per-stage facts
  without a chunked route: `_jp_road_cells` (with `_civWalkWayCells`),
  `_jp_infra_context`, `_jp_claimed_at`, `_jp_stage_infra`,
  `_jp_river_condition`, `_jp_sea_condition`, `_jp_coarse_idx`, `_jp_stop_key`,
  `_jp_mode_for_route`, `_civ_transshipments`/`_civ_transfer_overhead`,
  `_civ_passed_settlements`.
- **5b — `_jp_derive_stages`.** The chunker: per-point classification,
  contiguous-run chunking with wrap-aware km and metres of gain/loss, the
  narrow-water-gap collapse, the sliver absorb, the 14-stage cap, then per-stage
  settlement/claimed/dry-km/midpoint measurement and infra/route-condition
  resolution.
- **5c — `_jp_plan`.** The orchestrator: `_jp_effective_stage_plan`,
  `_jp_ensure_plan`, the v1.52 season-drift pre-pass, the per-stage vessel
  fallback, the supply forecast, hazards, the elevation profile, the daily
  timeline and the roll-up.

**Helpers on no milestone list that this one needed**:

1. **`buildCartBiome`/`buildCartTerrain`** (reference 6817/6860) — the Cartalith
   15-biome and 13-terrain paint layers. `_jpDeriveStages` samples both on every
   route point, and the port had never built either; `build_biome_raster` is the
   *climate* raster, a different vocabulary. Ported as
   `build_cart_biome`/`build_cart_terrain` with `CART_BIOMES`/`CART_TERRAINS`
   and `jp_legacy_biome_of`. Checked, not assumed: `ELEV_TO_CART` is indexed by
   `BIOME_INDEX`, which puts **shrub before savanna** — this port's own
   `BIOME_*` order, so the table transfers unchanged.
2. **`_civTransshipments`/`_civTransferOverhead`** (19198/19204) —
   `jp_journey_cost` takes the transshipment count as an argument.
3. **`_civWalkWayCells`** (21766) — `_jpRoadCells` cannot exist without it.
4. **`_civPassedSettlements`** (21154) — `_jpPlan`'s stops list.

**Two on its list are deliberately not Rust functions:**

- **`_jpLayovers`** is a JS lazy-init idiom ("give me `jn.layovers`, creating it
  if this journey predates the field"). A `HashMap<String, i64>` keyed by
  `jp_stop_key` needs none: the `JpLayovers` alias, which `jp_plan` takes.
- **`_jpSettlements`** is `state.places.filter(p => CIV_SETTLE_KEYS.has(p.kind))`
  — a runtime type test over one untyped array. This port's settlements are
  already typed, so building the `JpPlace` list *is* the filter.

**How the plan/stage shapes resolved:**

- `_jpDeriveStages` produces `JpDerivedStage`, which **does** carry `mx`/`my`
  (a genuine map measurement). `JpStage` does not: the calculators consume the
  finished wildlife multiplier. `JpDerivedStage::to_stage(wildlife_forage_mod)`
  is the bridge; `JpStage` needed no change.
- `JpPlan` gained `_jpEnsurePlan`'s remaining defaults (`route_cond`, `infra`,
  `stage_overrides`, `season_drift`, `rest_cadence`) and a sparse
  `JpStageOverride`. `jp_effective_stage_plan` is the reference's
  `Object.assign({},plan,ov)` with its per-species animal merge preserved.
- `jp_ensure_plan` derives the route's real stages and corrects the vessel guess
  through `jp_auto_pick_vessel` (`JP_VESSEL_PREFERENCE.find(jp_vessel_fits)`
  over the water stages) — milestone 2's function, built here because
  `_jpEnsurePlan` cannot exist without it.

**Two reference quirks reproduced as written**, recorded so nobody "fixes" them:

- `_jpDeriveStages` falls back to `state.mapWidthKm || 12000` while
  `_jpInfraContext`, two functions away, uses `|| 800`. Both kept.
- `_jpRoadCells` keys its map by string concatenation (`x+','+y`), and
  `_civWalkWayCells` emits a way's *first* and seam-break points **unrounded** —
  keys like `"12.5,3"` that no integer lookup can hit. Reproduced by not
  recording a non-integral emission: same observable behaviour, no float keys.

## Milestone 6 — verdict/reporting

`jp_verdict`/`JpVerdict`, `jp_confidence`/`JpConfidence`,
`jp_pack_range`/`JpPackRange`, `jp_fmt_days`, and `jp_risk` — the reference's
four-tier campaign-duration caption on `_jpPlan`'s return, which is a verdict
string and so belongs here, not to milestone 5. (`jp_fmt_kg` is milestone 4's.)

- The plan object is `JpJourneyPlan`; every field `_jpVerdict` consults is on it
  (`blocked_idx`/`results`/`resupply_reach`/`riv_x`/`pass_km`/`desert_km`/
  `bad_wx_pct`/`stops`). The reference's `blockedMsg` is not duplicated — it is
  read off the blocking stage's `JpBlocked`. `jp_verdict` returns a `JpVerdict`,
  not an `Option`: the reference's `if(!plan) return null` has no Rust
  equivalent.
- `JpJourneyPlan` carries the day count, not the `risk` caption; `jp_risk(days)`
  derives it.
- `_jpPackRange` reads `plan.plan` and `plan.hasDesert` off the finished
  journey; `JpJourneyPlan` does not carry the party plan back out (the caller
  owns it), so `jp_pack_range` takes `(&JpPlan, has_desert: bool)`.

## How the goldens were built, and what the harness taught

Every milestone from 3 on is golden-verified against the reference itself, not
hand arithmetic: reference line ranges sliced out of `reference/Cartalith Gen1
v2.10.html` and evaluated in a bare Node `vm.runInContext` with no DOM. Each
expected value in the Rust tests is that run's output.

- **Slice boundaries break silently.** Milestone 3's first run had an
  unterminated block comment at a slice boundary swallowing the next slice.
  From milestone 4 on, every slice carries a **block-comment balance
  assertion**; on milestone 5's eight slices it caught three boundary errors,
  and the JS parser caught a fourth (a slice that cut `_jpPlan`'s closing
  brace).
- **Balanced is not self-sufficient.** Milestone 5's slice `2641-2675` began one
  line below `TERRAIN_DETAIL_MAX_K` (2640), which `riverCoarseEase` reads — and
  `_jpDeriveStages` catches its own exceptions and returns an empty stage list,
  so the whole world derived to **zero stages** with no error printed. Found by
  instrumenting that `catch`; the slice is `2640-2675`. The balance assertion
  proves a slice is *syntactically* whole, not that it is *semantically*
  self-sufficient.
- **Milestone 5/6's slice list**: `riverCoarseEase`/`terrainDetailK`
  (2640-2675), `classifyBiome` (5736-5743), `BIOME_KEYS`/`BIOME_INDEX`
  (6796-6797), the cart paint layers (6810-6877), the whole Journey Planner
  (17297-19532, extended from 19419 to take v1.49's verdict layer),
  `_jpModeForRoute` (20368-20379), `_civPassedSettlements` (21154-21175) and
  `_civWalkWayCells` (21766-21777).
- **The fixture world** is synthetic but exactly reproducible: every field is a
  closed form in `+ - * /` over exact values, no transcendental, so the Rust test
  rebuilds identical `f32` grids and embeds only the outputs. A 24×16 map with
  an ocean margin, a lake, a ridge, a river column, a highway, a road spur,
  claimed territory and five settlements, crossed by a 24-point route that
  derives into seven stages (2 sea, 1 river, 4 land): 760.847480700888… km,
  41.317750030325… days, one transshipment, and a genuinely unmet resupply
  requirement. Because that unmet requirement alone forces `severe`, the verdict
  band probes edit exactly the signals `_jpVerdict` reads on this **real** plan,
  and the harness makes the identical edits.
- **Reusing a shared helper exposed a real bug in it.** `js_fixed` (JS
  `toFixed`'s round-half-*away-from-zero*, now `cartalith_jsmath::js_fixed`)
  decided ties by scaling, `(v*10^d + 0.5).floor()`, which **fabricates** ties:
  `61.5/30` is `2.0499999999999998`, which JS renders `"2.0"`, but `×10` rounds
  to exactly `20.5` in `f64` and the `+0.5` carried it to `"2.1"`.
  `jp_fmt_days(61.5)` caught it. The tie is now decided on the value's **exact**
  decimal expansion (a double is a dyadic rational, so a real tie at place
  `d+1` ends in a 5 there), with Rust's correctly-rounded `{:.N}` for everything
  else. Verified against `Number.prototype.toFixed` on 30 cases, including pairs
  that look identical and are not (`1.25` is a real tie, `2.05` is not), and
  `jp_fmt_kg(1250.0)` = `"1.3 t"`, the tie that reaches a user-visible string.

## The function census (closeout, 2026-08-18)

The census makes "every real `jp*`/`_jp*` function" a checkable bar: it fixes the
denominator and says which names are deliberately not Rust functions. **The
frozen reference defines exactly 74** `jp*`/`_jp*` functions.

- **To port: 66.** Every name not excluded below is a `cartalith-civ` function
  under its snake_case name. Check the set mechanically (reference name →
  snake_case → `fn <name>`), never by counting milestone write-ups; how many
  exist is `STATUS.md`'s answer.
- **Not portable: 6.** `_jpRunAuto`, `_jpRefresh`, `_jpSyncAssetInputs`,
  `_jpRenderPartyForm`, `_jpRenderStops`, `_jpRenderResults` — DOM rendering,
  which `ARCHITECTURE.md` assigns to Godot and which the shell re-expresses
  (`_jpRunAuto` + `_jpSyncAssetInputs` are the party form's carriage-Auto
  write-back). `_civRenderJourneyList` is a `_civ*` name and outside the 74.
- **Not Rust functions, by design: 2.** `_jpLayovers` and `_jpSettlements`
  (milestone 5).

66 + 6 + 2 = 74. `_jpRerouteForMode` is one of the 66, but its body is
`_civDijkstraPath`, so it waited on the Route tool's pathfinder (below) rather
than on any milestone here.

**Six helpers outside the `jp*` namespace** came along because a milestone
needed them: `buildCartBiome`/`buildCartTerrain` (with `CART_BIOMES`/
`CART_TERRAINS` — two paint layers this port had never built),
`_civTransshipments`/`_civTransferOverhead`, `_civWalkWayCells` and
`_civPassedSettlements`. They are real additions to the port, not planner
overhead.

**The pathfinder `_jpRerouteForMode` needed.** `cartalith_civ::tools::
civ_dijkstra_path` ports all three of the reference's domains
(`RouteMode::Land`/`Water`/`Mixed`, i.e. `_civLandCostGrid`/`_civWaterCostGrid`/
`_civMixedCostGrid`) **and** v1.47's `reachable` flag — exactly what
`_jpRerouteForMode` needs, since it *"never silently accepts `_civDijkstraPath`'s
straight-line fallback as if it were a real path"* — golden-verified bit-exact
over 16 cases including both unreachable directions. Note that
`_civDijkstraPath` is **not** `road_dijkstra`: that is the bare relaxation
kernel; the cost grids, the existing-way discount, settlement gravity,
wrap-aware smoothing and the `reachable` flag are all in the wrapper. What was
left for `jp_reroute_for_mode` was `jp_mode_for_route`, a `reachable` check, the
call, v1.100's `forceMode` override and two refusal strings.

## Engine additions that are not ports

The reference has no counterpart for these, so they are not counted among the
74:

| Addition | Why it is not a port |
|---|---|
| `jp_plan_cost` | The reference's call site is inline JS at line 19854, not a function; this is that call site's Rust half |
| `JpTerm` / `JpLandCalc::trace` / `JpWaterCalc::trace` | The reference builds a `formula` **string** in the same place. Prose stays in Godot; the structured chain is engine fact that could not be re-derived across the boundary without a second copy of every table. Invariant: `∏ factor == daily_km` |
| `jp_trim_points` | `JOURNEY_PLANNER_SPEC.md` §3's ⇧-drag trim; v2.10 has no distance spine. The trimmed polyline goes through the same `jp_plan` as an untrimmed one |
| `jp_auto_stage_picks` | Applies milestone 2/6's per-stage suggestions instead of only showing them, behind `jp_compute`'s opt-in `auto_stage` — `DECISIONS.md` §7j (owner decision, 2026-08-26), which keeps the reference's +10 % margin |
| `JpAnimalResolver` / `JpVesselResolver`, the `_ex`/`_full` variants (`jp_capacity_ex`, `jp_calc_land_ex`, `jp_calc_water_ex`, `jp_plan_ex`, `jp_plan_full`) | The Travel Library's hooks (`TRAVEL_LIBRARY_SPEC.md` §6): `jp_plan` → `jp_plan_ex` (animal resolver) → `jp_plan_full` (both). With no resolver each is identical to the plain port |
| `jp_leg_supply` | One leg's share of `_jpPlan`'s supply forecast, lifted out of the roll-up so story planning's SP-2 can spread consumption over the leg's own days; the roll-up sums exactly these terms in the same order |
| `civ_path_water_frac` | Not a `jp*` name: `_civPathWaterFrac` (21142), needed to make `_jpEnsurePlan` route-aware in the shell (route-planner conformance, below) |

## Integration — the five steps

A finished engine is not a finished feature: the reference's planner is a form
the player fills in per journey — origin, destination, party, season,
transport, supply days, grazing, per-stage overrides — not something a generator
auto-computes for every settlement pair. So it is **not** wired into
`compute_civilisation()`: that would invent journeys nobody asked for. Making it
a feature takes five steps. (This section absorbs the former dated sections
"What integration would actually mean", "Update (2026-08-19)", "Update
(2026-08-20)", "Update (2026-08-23)" and "Redesign: the distance spine".)

1. **A route to plan.** `jp_plan` takes a polyline. The Route tool
   (`route_begin`/`route_append_stop`/`route_commit`/`route_discard`,
   `UNIFIED_TOOL_PLAN.md` milestone F) supplies it, and `route_count()` /
   `route_get(index) → {points, brks, km, mode, unreachable_legs}` read it back.
2. **A `JpWorld` assembled from live state.** `journey_bridge::JourneyWorld`
   borrows what `WorldGen` already holds — `field`/`temp`/`rain`/`flow_field`
   from `WorldState`; `water_bodies`/`territory`/`ways`/`settlements` from
   `CivData`; `peak_m`; `flow_thresh` from the same
   `cartalith_hydrology::river_flow_thresh` call `compute_civilisation` makes —
   and computes `cart_biome`/`cart_terrain`/`road_cells` at call time from
   rasters that already exist. **No generation stage was added.**
   - **Ocean-current and wind fields** come from `coarse_ocean_wind_fields`,
     which calls `cartalith_climate::current_ocean_field`/`current_wind_field`
     on demand (the Wind/Ocean-currents debug views' recipe) — no `WorldState`
     retention needed. `None` stays `jp_sea_condition`'s supported input for a
     caller that has neither.
   - **Road cells** read the generated ways, the hand-drawn ways
     (`tools::ManualWay`, with the reference's `'ancient' → ["Dirt
     Track","Deteriorated"]` mapping) and `CivData::road_edges` — the edges of
     `civ_hierarchical_network_topology`, standing in for the reference's second
     road source, `state.roads.edges`.
     `JourneyWorld::build` passes empty slices for the last two and the caller
     overwrites `road_cells` straight after, deliberately: widening `build`'s
     signature would re-derive `road_cells` for callers that also want the
     world's other tables (`journey_bridge.rs`' module doc).
   - **`jp_claimed_at` tests `territory[i] >= 0`**, and `assign_territory` uses
     `0` for unowned, so every cell reads as claimed. That is exactly the
     reference's behaviour (its `civTerritory` is a `Uint8Array`, so `>= 0` is
     always true); "correcting" it at the boundary would silently diverge from a
     golden-verified consumer.
3. **A party form.** `JpPlan` is ~20 fields, ten party counts and a sparse
   per-stage override map — a GUI surface, not a `#[func]` signature. Built as
   `shell/journey_planner_view.gd`, an in-shell takeover laid out per
   `JOURNEY_PLANNER_SPEC.md` (direction 1a). It seeds from the engine, never
   from restated defaults; see route-planner conformance for which call.
4. **`#[func]`s over the boundary.** `journey_bridge.rs` is `godot`-free (the
   isolation `sculpt_bridge`/`civ_tools_bridge`/`infra_tools_bridge`
   established) and owns the form parser, its inverse, `JourneyWorld` and the
   option tables; `lib.rs` owns the `Variant` conversion.
   - `jp_options()` — every dropdown vocabulary, keyed by the field names
     `jp_compute` accepts; `route_cond` nested per travel category (a
     "Maintained" road cannot describe a sea leg), plus a `reference` table set
     a results panel needs to *label* what came back.
   - `jp_default_plan()` — `JpPlan::default()` flat: 28 keys
     (`journey_bridge::plan_to_pairs`) plus `party_fields`.
   - `jp_plan_for_route(route_index)` — `_jpEnsurePlan` in full over a
     committed route, the same shape plus `sea_journey`.
   - `jp_compute(request)` — `jp_plan_full` → `jp_verdict` → `jp_confidence`,
     flattened. Request keys: `route` or `points`, and optional `plan`,
     `stage_overrides`, `layovers`, `animal_entries`, `auto_carriage`,
     `auto_stage`, `trim`. The return nests to the data's real depth: `stages`,
     `results` (each with its `land`/`water` calculation **and** the effective
     plan that leg ran under, which season drift and the vessel fallback can
     both alter), `timeline`, `stops`, the cost, and the verdict/confidence.
     `JpJourneyPlan` flattens through several helpers and each leg's `eff` is a
     whole second `JpPlan`: the plumbing is small, the serialization is not.
   - `jp_reroute(route_index, transport, force_mode)` — `jp_reroute_for_mode`
     over a committed route.
   - **Unknown or wrong-typed keys come back in `rejected`**, per this
     codebase's "a typo'd key is a bug worth seeing" policy; a plan can be `ok`
     *with* rejections and still be real, computed from the defaults.
   - **Every option table is tested against the engine's own lookup.** A
     dropdown offering a key the engine does not know never errors — it falls
     through to `?? 1.0` and reports a plausible number from the wrong row.
     Pace, grazing, foraging, rest cadence and route condition are `match` arms
     rather than `pub const` tables, so those tests are the only guard against a
     transcription typo becoming a silently wrong journey.
5. **The presentation the port left out.** Every HTML hint, every trace and the
   elevation profile are Godot's to draw; the values they print are fields on
   the returned structs.

**Saved journeys.** A named journey is story planning's engine entity
(`cartalith_civ::travel_library::Journey`, `STORY_PLANNING_SCOPE.md` SP-1),
persisted as `entities/journeys.json` (`SAVEFILE_COMPAT.md` §9.6) and re-snapped
on a regenerate (Ruling AO). The planner's own per-journey working state — stage
overrides, layovers, animal entries, trim — is shell state and is not part of
that entity.

## The Travel Library and the party form

Two couplings, each deliberately narrow; `TRAVEL_LIBRARY_SPEC.md` §6 has the
full reasoning.

- **`JpParty` stays four fixed species.** Widening it to a generic animal map
  was examined and declined: `jp_capacity_ex` reads per-species seasonal
  physiology (`jp_seasonal_animal`) and desert multipliers
  (`jp_desert_animal_mod`) that the spec's §3.1 has no fields for, so a wholly
  new species would silently take neutral `1.0`s — the "plans silently wrong"
  failure the spec's §5 exists to prevent. The shipped path is **substitutes
  for**: a custom entry occupies one of the four slots with its own capacity,
  speed, fodder, water and terrain table.
- **No vessel ↔ water-type coupling is invented.** The spec's §3.3 gives a vessel
  a `sailing_window` (daylight / continuous); the engine's sailing window is
  `jp_water_window`, a property of the **water type**, and nothing in the
  reference couples the two. The planner prints the engine's hours and says
  which one it shows. Likewise `ShipStats::invalid_water` has no §3.3 field, so
  a custom vessel is constrained by mode and water rating only, and the picker
  says so.

## Route-planner conformance (2026-08-25)

Prompted by the owner: *"check the route planner option and if it still performs
as the html — finding the optimal path and clipping to a route as soon as that
method is cheaper and faster."*

**What the reference does, the port does.** `civ_dijkstra_path` carries all
three cost grids, the ×0.25 `_CIV_EXISTING_WAY_DISCOUNT` over land ways *and* sea
lanes with v1.53's `isFinite ? … : 1.0` branch, `_civApplySettlementGravity`,
wrap-aware `_civSmoothPath` with the mode-matched terrain-validity repair, and
the straight-line fallback plus `reachable`. `civ_join_dijkstra_segs` chains it
per waypoint pair; `route_commit` hands it the generated ways **plus** the
manual ways — the port's substitute for the reference's `state.roads.edges`
marking — so "clip onto an existing route where that is cheaper" holds on commit
and on re-route. Pinned by `golden_parity_civ_tools.rs`
(`case0_land_route_matches_civ_dijkstra_path`,
`case0_settlement_gravity_bends_the_route`,
`case0_existing_way_discount_pulls_the_route_onto_it`,
`case0_mixed_route_crosses_the_ocean`, the last with land mode refusing the same
pair). Nothing in the path uses RNG.

**`_jpEnsurePlan`'s route-awareness must reach the shell.** Seeding the form from
`jp_default_plan()` (Walking, Keelboat, route-blind) opened a route the `mixed`
grid took across open ocean *because that was cheaper* on the land itinerary —
the failure the reference's own v0.6 comment says it fixed. The missing input
was `_civPathWaterFrac` (21142), which `_civCommitRoute` thresholds at `>= 0.5`
to set `jn.sea`:

- `cartalith_civ::civ_path_water_frac` ports both branches of `wb ? wb[fi] !== 0
  : field[fi] < sea` (`commit_route_water_fraction_decides_the_sea_flag`).
- `jp_plan_for_route` runs `_jpEnsurePlan` over the committed route; an empty
  `Dictionary` on a stale binary or bad index lets the view fall back to
  `jp_default_plan()` rather than break.
- The form seeds from it **only when its plan values are empty** — the
  reference's own `isNewPlan` gate — so re-entering never overwrites a party the
  user has edited.

**A re-route solves under the transport's domain, and sizes its inputs from
it.** `RouteInputs::build` derives the biome raster and river orders for `Mixed`
only, so building them from the route's *committed* mode while solving under the
*transport's* domain gave a river re-route of a non-mixed route a cost grid with
no `_civNavigableRiverDiscount` — a silently worse path, not an error.
`jp_reroute_mode(transport, force_mode)` decides the domain once and both use
it. A successful re-route also rewrites `CommittedRoute::mode`, so `route_get`'s
`"mode"` names the domain actually used.

The probe `godot-project/_routeplanner_probe.gd` generates a fixed-seed world,
commits a route between the two furthest settlements, and asserts a solved path
(more than two points, no unreachable legs), determinism across a re-solve,
`jp_plan_for_route`'s transport agreeing with its `sea_journey`, a real
`jp_compute` with stages and a verdict, a land re-route succeeding, and a sea
re-route between inland endpoints being refused rather than faked.

## Done means (whole plan)

Every real `jp*`/`_jp*` function (UI-only ones excluded) ported and tested in
`cartalith-civ`, reachable from a real `#[func]` taking a journey request
(origin, destination, party composition) and returning a real plan.
