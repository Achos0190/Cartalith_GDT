# Economy / Journey Planner: investigation and first bounded slice

Prompted by the owner's own `/goal` directive to keep working through Phase 2's
remaining scope. `PHASE2_SCOPE.md` has repeatedly named "economy" and the
"Journey Planner" as out-of-scope-for-whatever-milestone-was-current without
ever actually reading the reference for either — `ROADMAP.md`'s own Phase 2
entry already warns *"The Journey Planner is large and largely self-contained.
Consider it a sub-phase rather than bundling it,"* but that warning had never
been checked against the real source, the same discipline this session
already applied twice before (territory in milestone 9, provinces after it)
where a scope-doc one-liner turned out to need correcting on contact with the
actual reference code.

## What "economy" and "Journey Planner" actually are, on inspection

They are **two genuinely separate, both genuinely large, subsystems** —
confirmed by reading the real reference source (`reference/Cartalith Gen1
v2.10.html`), not assumed from `FUNCTION_INDEX.md` names alone.

### 1. Faction/settlement economy & trade layer (~20 real functions)

Reference lines ~23400-24600. The reference's own header comment (line 23530)
calls this *"Pure UI/data-exposure layer feeding the new Factions/
Settlements/Economy/Statistics pages — NOT new simulation... a cheap
on-demand aggregation of already-computed per-cell fields... or an
explicitly-labeled heuristic composite."* Real pieces, by size:

- **`_civFactionAggregates`** (line 23575, ~165 lines) — the one `O(GW·GH +
  nPlaces)` pass behind the Factions/Economy/Statistics pages: per-faction
  population, territory km², food production capacity/surplus, trade volume,
  tax income, resource means, a five-axis "power" heuristic
  (military/economic/political/cultural/religious — explicitly labeled
  heuristic, not simulated), sector output (fishing/agriculture/livestock/
  forestry/mining/craft), and v1.55's "Territory Fit" terrain-mix stats
  (river/coast/arid/forest/hills fractions). Reads `currentPopulationDensity`,
  `currentResourcePotentials`, `buildBiomeRaster`, the flow field, and
  `civTerritory` (this port's milestone 10 `assign_territory` output stands
  in cleanly here, per the same reasoning that unblocked provinces).
- **`_civPlaceTrade`** (line 24459) — the per-settlement counterpart:
  specialisation-implied exports/imports, hinterland resource balance (via
  `_civResourceTradeBalance`, **ported this pass**), food surplus/deficit
  (`_civPlaceFoodSurplus`/`_civFoodShed`), fuel-limited smelting
  (`_civPlaceSmelting`, read in full, not yet ported — see below), and salt
  self-sufficiency (`_civSaltAccess`, not yet read in detail).
- **`_civResourceTradeBalance`** (line 24175, v1.33) — **ported this pass**
  (`civ_resource_trade_balance` in `cartalith-civ`). The one shared
  trade-threshold rule both the faction and settlement views now use, after
  v1.33 unified two copies that had drifted (the reference's own comment:
  "this is the third time this shape has appeared… so the rule now lives in
  exactly one place"). Fully self-contained: takes a settlement's (or
  faction's) own catchment-mean resource values and the world mean for the
  same keys, returns which resources are exports vs. imports. No new upstream
  dependency — operates on caller-supplied means, not raw resource fields.
- **`_civPlaceSmelting`** (line 24208, v1.31) — read in full, **not yet
  ported**. A genuinely interesting real constraint (iron smelting gated by
  charcoal fuel, not ore — cites `settlement-resources.md` §10.2-§10.3, with
  real historical grounding: the Elba case, ore-rich-but-fuel-poor). Pure,
  reads `iron`/`timber` from `ResourcePotentials` (both still retained after
  the memory-optimization fix, commit `62b9b51` — only clay/buildstone/
  flint/obsidian/sulfur/alum were freed) over a settlement's catchment
  radius. **Real new dependency it needs that this port doesn't have yet**:
  `_CIV_CATCHMENT_KM2`/`_civCatchmentRadiusCells` (a per-settlement-tier
  catchment-radius lookup, reference lines 23407/23481) — a small, bounded,
  genuinely portable piece, but a real gap, not assumed away.
- **`_civFoodShed`/`_civPlaceFoodSurplus`/`_civPlaceCatchmentCeiling`/
  `_civCatchmentPop`/`_civSettlementPopulation`** (lines 23774/23765/23490/
  23506) — a small interlocking cluster computing "people this settlement's
  catchment can feed" vs. actual population. Read in outline, not yet ported
  in full detail.
- **`_civSaltAccess`** (line 24430), **`_civPlaceResourceContext`** (line
  24567) — read by name/location, not yet read in full.
- **`_civRenderEconomyPage`** (line 16511) — UI rendering only, same
  disposition as every other `_civRender*` function this port has correctly
  never ported (Godot owns presentation, `ARCHITECTURE.md`).

**Real tension found, not yet resolved**: `_civResourceTradeBalance`
operates over all 15 `CIV_RESOURCE_KEYS`, but this session's own
memory-optimization pass (`MEMORY_OPTIMIZATION_SCOPE.md`, commit `62b9b51`)
frees 6 of those 15 fields (clay/buildstone/flint/obsidian/sulfur/alum)
immediately after `build_resource_potentials` returns in
`compute_civilisation()`, because nothing consumed them at the time. A full
port of `_civPlaceTrade`/`_civFactionAggregates` (which do read all 15 for
their resource-mean aggregation) would need that memory fix revisited —
either stop freeing those six fields, or restructure so the trade layer runs
*before* they're freed. **Not resolved in this pass** — a real design
decision for whoever picks up the next milestone here, not something to
silently reverse a deliberate, measured memory fix to unblock.

### 2. Journey Planner (`jp*`/`_jp*`, ~70 real functions)

Reference lines ~17300-20400. `ROADMAP.md`'s "consider it a sub-phase"
warning is **confirmed accurate, not overcautious** — this is genuinely one
of the largest single features in the whole reference, larger than the
entire civ-layer settlement/faction/road pipeline this port has spent 15+
milestones on. Real scope, by category (read function names/signatures, not
full bodies, given the size — a proper investigation of this alone would be
its own multi-hour task):

- **Transport mode selection**: `jpAutoPickTransport`, `jpBestAnimalForContext`,
  `jpPickSpeciesForRoute`, `jpCanUseWheels`, `jpResolveMount`,
  `jpAutoPickVessel`, `jpVesselMatrix`, `_jpVesselFits`, `_jpBestLandTransportForStage`,
  `_jpBestPackageForStage`, `_jpAutoStageVessel` — choosing pack animals,
  wheeled vehicles, or vessels per route segment based on terrain/season.
- **Physical travel cost**: `jpTrainPace`, `jpSailFactor`, `jpWaterWindow`,
  `jpFatigue`, `jpLoadPenalty`, `jpSurfaceGain`, `jpWxWeighted`,
  `jpWeatherFactor`, `jpAnimalTerrainMod`, `jpColumnLengthKm`,
  `jpColumnFactor`, `jpCalcLand`, `jpCalcWater`, `jpJourneyCost` (the
  apparent top-level cost function, line 18873) — real physical modeling
  (fatigue, load, weather, terrain, column length for large groups).
- **Consumption/resupply**: `jpHumanWaterCarryDays`, `jpHumanWaterRate`,
  `jpAnimalWaterCarryDays`, `jpConsumptionFactors`, `jpCapacity`,
  `jpForaging`, `jpAssessResupply`, `_jpWorldMeanRichness`,
  `_jpWildlifeForageMod`, `_jpResupplyReach`, `_jpDrinkingCoarseEase`,
  `_jpStageDryKm`, `_jpDesertTierForGap` — water/food logistics along a
  route, including desert-crossing viability.
- **Route/stage derivation**: `_jpDeriveStages`, `_jpEffectiveStagePlan`,
  `_jpPlan`, `_jpEnsurePlan`, `_jpLayovers`, `_jpStopKey`, `_jpRoadCells`,
  `_jpSettlements`, `_jpInfraContext`, `_jpClaimedAt`, `_jpStageInfra`,
  `_jpRiverCondition`, `_jpSeaCondition`, `_jpCoarseIdx`,
  `_jpWaterReachCells`, `_jpModeForRoute`, `_jpRerouteForMode` — building a
  real multi-stage journey plan from infrastructure/terrain/water access.
- **Seasonal/closure logic**: `jpSeasonalClosure`, `jpRestDays`,
  `jpSeasonAt`, `jpSeaClosure` — routes that close seasonally (passes,
  ice, storm seasons).
- **Verdict/reporting**: `_jpVerdict`, `_jpConfidence`, `_jpPackRange`,
  `jpFmtKg`, `jpFmtDays` — journey feasibility verdicts with confidence
  bands.
- **UI-only, not portable**: `_jpRunAuto`, `_jpRefresh`,
  `_jpSyncAssetInputs`, `_jpRenderPartyForm`, `_jpRenderStops`,
  `_jpRenderResults`, `_civRenderJourneyList` — DOM-coupled, same
  disposition as every other `_civRender*`/UI function in this port.

**Not investigated further, and not attempted this pass** — matching
`ROADMAP.md`'s own explicit instruction. This is a real sub-phase requiring
its own dedicated scope document with a proper milestone breakdown (likely
10+ milestones given the size, following this project's own precedent of
breaking the civ layer itself into 15+), not something to bundle into an
economy investigation. Whoever picks this up should budget accordingly —
this alone is comparable in size to the entire Phase 2 civ-layer effort to
date.

## What was actually built this pass

**`civ_resource_trade_balance`** (`cartalith-civ/src/lib.rs`) — a direct,
faithful port of `_civResourceTradeBalance` (reference line 24175), plus
`CIV_RESOURCE_KEYS` (the full 15-key vocabulary, distinct from the existing
`SUIT_RESOURCE_KEYS` 9-key ore subset) and `CIV_CONSUMED_RESOURCES` (the
8-key subset an import can ever apply to). Fully self-contained: operates on
caller-supplied `mean`/`world_mean` maps, no dependency on the
memory-optimization tension noted above, no new upstream data needed.

Chose real unit tests over a Node-harness golden extraction for this one
function specifically: it's ~12 lines, pure, branch-complete (four real
branches: world-absent-export-only, ratio-and-floor export, consumed-only
import, and the implicit "neither" case), and every branch is directly
traceable from the reference source with no RNG/state/iteration order to get
subtly wrong — the category of function this project's own `PARITY_TESTING.md`
discipline treats real unit tests as a legitimate stand-in for (same
precedent as milestone 10's territory/the provinces work: real algorithmic
verification, not a fabricated golden fixture). Seven tests cover: empty
inputs, the world-essentially-absent branch's absolute floor, the
ratio-clears-but-absolute-floor-fails case (a real branch-order subtlety),
a genuine export, import gated correctly to `CONSUMED_RESOURCES` only (a
resource that's locally scarce but never consumed, like `gems`, must never
import), missing-key-as-zero fallback, and the full 15-key vocabulary order.

Kept a deliberate JS-parity subtlety rather than "fixing" it: the reference's
`!(world>0.002)` (not `world<=0.002`) matters for `NaN` inputs — `!(NaN>x)`
is `true` in both JS and Rust, but `NaN<=x` is `false` in Rust, so rewriting
to satisfy clippy's `neg_cmp_op_on_partial_ord` lint would silently change
behavior on a NaN input. Kept the JS-matching form with a `#[allow]` and a
comment explaining why, per `cartalith-rust-conventions`' own standing rule
that NaN comparison differs between JS and Rust and needs conscious handling,
not a lint-driven rewrite.

## Verification

`cargo test -p cartalith-civ --lib trade_balance` (7 new, all passing),
`cargo test --workspace` (0 regressions, every existing test unmodified),
`cargo clippy -p cartalith-civ --all-targets` (clean — the one real warning
in the new code addressed deliberately, not suppressed blindly, see above).

## Done means (this pass)

`civ_resource_trade_balance` ported, tested, verified — a real, useful,
already-complete building block for whoever tackles the next economy
milestone. Not wired into `compute_civilisation()`/Godot yet — it's a pure
utility function with no caller in this port until the broader
`_civPlaceTrade`/`_civFactionAggregates` orchestration exists to call it; wiring
a function with no real caller into the Godot API surface would be exactly
the "technically done, practically inert" trap `PHASE2_SCOPE.md`'s own
milestone-9 note already named once.

## Real next milestones for whoever continues this (not started)

1. **`_civCatchmentRadiusCells`/`_CIV_CATCHMENT_KM2`** — small, bounded,
   needed by `_civPlaceSmelting` and the whole food-surplus cluster. A clean
   first slice for a follow-up pass.
2. **`_civPlaceSmelting`** — depends on (1), otherwise fully read and ready
   to port faithfully (see above).
3. **`_civFoodShed`/`_civPlaceFoodSurplus`/`_civPlaceCatchmentCeiling`/
   `_civCatchmentPop`** — the food-surplus cluster, depends on (1) and on
   `currentAgrarianDensity`/`currentCarryingCapacity` (check what this port
   already has from milestone 4's `build_carrying_capacity`/`build_npp`
   before assuming a gap).
4. **Resolve the memory-optimization tension** (see above) before attempting
   `_civFactionAggregates`'s full resource-mean aggregation, which needs all
   15 `CIV_RESOURCE_KEYS` resident.
5. **`_civFactionAggregates`** itself — the big one, ~165 lines, real design
   work deciding what subset of its heuristic "power" composite is worth
   porting vs. genuinely UI-only.
6. **The Journey Planner**, as its own separate sub-phase with its own scope
   document — not bundled with the above, per `ROADMAP.md`'s own instruction
   and this investigation's own confirmation of why.
