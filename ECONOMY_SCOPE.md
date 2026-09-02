# Economy / Journey Planner: investigation and first bounded slice

> **This document is an investigation record and a scope definition; it does
> not track progress.** What it names as future work — `_civPlaceSmelting`,
> `_civSaltAccess`, the food-surplus cluster, the surfaced resource half of
> `_civFactionAggregates` — is defined here and **tracked only in
> `cartalith-native/docs/STATUS.md`**. The dated pass narratives below say what
> each pass read, built and decided; they are history and are written in the
> past tense on purpose.

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
  (`_civPlaceSmelting`, read in full, not ported by this pass — see below), and
  salt self-sufficiency (`_civSaltAccess`, not read in detail by this pass).
- **`_civResourceTradeBalance`** (line 24175, v1.33) — **ported this pass**
  (`civ_resource_trade_balance` in `cartalith-civ`). The one shared
  trade-threshold rule both the faction and settlement views now use, after
  v1.33 unified two copies that had drifted (the reference's own comment:
  "this is the third time this shape has appeared… so the rule now lives in
  exactly one place"). Fully self-contained: takes a settlement's (or
  faction's) own catchment-mean resource values and the world mean for the
  same keys, returns which resources are exports vs. imports. No new upstream
  dependency — operates on caller-supplied means, not raw resource fields.
- **`_civPlaceSmelting`** (line 24208, v1.31) — read in full, **not ported by
  this pass**. A genuinely interesting real constraint (iron smelting gated by
  charcoal fuel, not ore — cites `settlement-resources.md` §10.2-§10.3, with
  real historical grounding: the Elba case, ore-rich-but-fuel-poor). Pure,
  reads `iron`/`timber` from `ResourcePotentials` (both still retained after
  the memory-optimization fix, commit `62b9b51` — only clay/buildstone/
  flint/obsidian/sulfur/alum were freed) over a settlement's catchment
  radius. **Real new dependency this investigation found the port lacking**:
  `_CIV_CATCHMENT_KM2`/`_civCatchmentRadiusCells` (a per-settlement-tier
  catchment-radius lookup, reference lines 23407/23481) — a small, bounded,
  genuinely portable piece, but a real gap at the time, not assumed away. The
  same pass then ported it; see "Memory-optimization tension" below.
- **`_civFoodShed`/`_civPlaceFoodSurplus`/`_civPlaceCatchmentCeiling`/
  `_civCatchmentPop`/`_civSettlementPopulation`** (lines 23774/23765/23490/
  23506) — a small interlocking cluster computing "people this settlement's
  catchment can feed" vs. actual population. Read in outline only by this
  pass.
- **`_civSaltAccess`** (line 24430), **`_civPlaceResourceContext`** (line
  24567) — read by name/location only; this pass did not read either in full.
- **`_civRenderEconomyPage`** (line 16511) — UI rendering only, same
  disposition as every other `_civRender*` function this port has correctly
  never ported (Godot owns presentation, `ARCHITECTURE.md`).

**The real tension this investigation found** (resolved by a later pass —
see "Memory-optimization tension" below): `_civResourceTradeBalance`
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

## Memory-optimization tension: resolved (2026-08-17)

Confirmed real by direct inspection, not assumed away — grepped the
reference's actual `_civFactionAggregates` (lines 23640/23653) and
`_civPlaceResourceContext` (lines 24572-24580): both genuinely iterate the
full 15-key `CIV_RESOURCE_KEYS` vocabulary, reading `pots[k][i]` for every
key including the six `compute_civilisation()` was freeing immediately
after `build_resource_potentials` returns.

**Compounding finding, not in the original write-up**: per-faction
resource-mean aggregation (`_civFactionAggregates`'s own approach) also
needs `territory` (faction ownership per cell), which this port's
`assign_territory` doesn't compute until much later in the pipeline — so
"just reorder two adjacent lines" doesn't work; the fields would need to
stay alive across the entire settlement-placement/road/naming span either
way. But `_civPlaceTrade`'s own approach (`_civPlaceResourceContext`, a
fixed-radius disc scan around a settlement position) needs no territory at
all — only settlement positions, which exist right after naming/villages,
well before territory is computed.

**Resolution shipped this pass**: the settlement-catchment-based trade
balance (`_civPlaceTrade`'s hinterland term, not `_civFactionAggregates`'s
territory-based one) is what this pass built. `civ_world_mean_resources` (the one
genuinely territory-independent piece of `_civFactionAggregates`, extracted
standalone since `_civPlaceTrade`'s own `worldMean` argument reuses that
exact value per the reference) plus `civ_catchment_km2`/
`civ_catchment_radius_cells` (`_CIV_CATCHMENT_KM2`/`_civCatchmentRadiusCells`,
reference lines 23407/23481) plus `civ_place_resource_context`
(`_civPlaceResourceContext`, reference line 24567) — all new in
`cartalith-civ`, 8 real unit tests including a proof the disc-scan
rejection/wrap/ocean-exclusion logic works correctly, not just the happy
path.

`compute_civilisation()`'s resource-field free (the six unused keys) moved
from immediately after `build_resource_potentials` to right after
settlements are finalized (before `territory`) — a real, bounded, measured
tradeoff: these six fields (~96 MB at 2048×2048) now stay resident through
settlement placement/road-building/naming instead of being dropped
immediately, but steady-state after `compute_civilisation()` returns is
completely unaffected, since they're still freed, just later. New
`get_trade_balances()` `#[func]` in `cartalith-godot` exposes real
per-settlement export/import data, same order/index as `get_settlements()`.

**Left for a later pass, and taken up by one:** `_civFactionAggregates`'s own
territory-based per-faction aggregation (population, tax, the five-axis
"power" heuristic, sector output, Territory Fit). Its own section at the end
of this document records what that pass found.

## Done means (this pass)

The bar this pass set itself: `civ_resource_trade_balance` ported, tested and
genuinely wired — `civ_world_mean_resources`/`civ_place_resource_context`
giving it real inputs, `compute_civilisation()` calling it per settlement, and
`get_trade_balances()` exposing the result to Godot. A GDScript UI was
deliberately **outside** that bar: it belongs to the GUI-shell work when that
reaches the economy panel.

## Real next milestones for whoever continues this

Defined here, tracked in `cartalith-native/docs/STATUS.md`.

1. **`_civPlaceSmelting`** — `_civCatchmentRadiusCells`/`_CIV_CATCHMENT_KM2`
   (its stated dependency) are now ported (`civ_catchment_radius_cells`/
   `civ_catchment_km2` above) — this is now a clean, unblocked first slice,
   fully read and ready to port faithfully (see the original finding above).
2. **`_civFoodShed`/`_civPlaceFoodSurplus`/`_civPlaceCatchmentCeiling`/
   `_civCatchmentPop`** — the food-surplus cluster, depends on (1) and on
   `currentAgrarianDensity`/`currentCarryingCapacity` (check what this port
   already has from milestone 4's `build_carrying_capacity`/`build_npp`
   before assuming a gap).
3. **`_civFactionAggregates`** itself — taken up on 2026-08-18; see the
   section below. The "what subset of the heuristic power composite is worth
   porting" question resolved as **all of it, verbatim**; the memory tension
   turned out **not to bind**, because the half that unblocks
   `civ_culture_terrain_fit` needs no resource field at all.
4. **The Journey Planner** — split out into its own scope document,
   `JOURNEY_PLANNER_SCOPE.md`, which is where its milestones are defined.

## `_civFactionAggregates` itself — the 2026-08-18 pass

Milestone 3 of the "real next milestones" list above. Ported as
`civ_faction_aggregates`
(`cartalith-civ`), together with `_civFactionCapital` (reference line 23566),
the `CIV_TAX_RATE`/`CIV_PRIMARY_SPECIALISATION` tables (23557/23553), and
`_civOceanDistField` (22450) as `civ_ocean_dist_field` — the coast axis needs
an ocean-only chamfer distance transform and this port had `chamfer_dist`
only as a private helper.

### What it actually aggregates

One `O(GW·GH + nPlaces)` pass, and genuinely *not* new simulation — the
reference's own header comment (line 23530) is the scope statement, and it
holds up on a full read. Three groups:

1. **From the grid, per faction (needs `territory`)** — territory cell count
   → km², food production capacity (`Σ populationDensity[i]·cellKm²`), the
   15-key `CIV_RESOURCE_KEYS` territory means, and v1.55's five "Territory
   Fit" terrain fractions (river / coast / arid / forest / hills). Plus the
   same five fractions and the same 15 means over **all land**, which is
   `worldMeanTerrain`/`worldMeanResource`.
2. **From `state.places`, per faction** — population, trade volume,
   economic-importance sum, settlement count, fortified count, tax income
   (`Σ pop · CIV_TAX_RATE[kind]`), and a six-way sector split of a production
   proxy (`pop·(0.4+0.6·economicImportance)`) keyed by specialisation, with
   everything unmapped folding into `craft`.
3. **Derived, per faction** — the capital pick (`_civFactionCapital`: highest-pop
   capital/metropolis if one exists, else highest-pop of any kind), the
   five-axis **power** composite, food surplus (`capacity − pop`), the
   export/import lists (`civ_resource_trade_balance` over the territory means
   vs. the world means, plus a `food` entry from the surplus sign), the
   `>0.4` strategic-resource list, and the craft share.

The five-axis power composite was the one real "port verbatim or simplify?"
decision this milestone had to make. **Ported verbatim**, because the
reference labels it honestly rather than dressing it up ("explicitly derived/
heuristic, never presented as simulated"; `cultural` carries its own comment
"population-proportional placeholder — no spread/assimilation model exists").
Simplifying it would have meant inventing a *different* heuristic with no
reference to check against — strictly worse than porting a disclosed one.

### Four inputs this port does not have, handled without inventing anything

Verified by grep across every crate in the workspace: `p.tradeVolume`,
`p.economicImportance`, `p.specialisation` and `_umInferWalls(p)` have **no
producer anywhere in this port**. The first two are persisted by the
reference's Auto-Populate/Generate-Roads passes, which are not ported; the
last is the urban-morphology block's `_umWallSpec` inference.

They are caller-supplied fields on `FactionPlace`, and
`FactionPlace::from_settlement` (this port's real `NamedSettlement` data)
fills each one with the value the reference itself computes when the field is
absent: `p.tradeVolume||0` → `0.0`, `CIV_PRIMARY_SPECIALISATION[undefined]
||'craft'` → `None`, `_umInferWalls` → `false`. So the aggregation is
complete and correct for whatever data a caller actually has, and no number
is fabricated for data nobody produces. The golden harness captured the
reference's own `_umInferWalls` verdict per place and feeds the same booleans
in, so `fortifiedFraction` and the military axis are genuinely tested rather
than trivially zero on both sides.

### One real JS-semantics trap, found by re-reading rather than by a test

The reference guards every per-place number with `|| 0` (`pop=p.pop||0`,
`p.tradeVolume||0`, `p.economicImportance||0`, and both sides of
`_civFactionCapital`'s `(p.pop||0)>(best.pop||0)`), and every divide-by-max
with a truthiness check (`maxPop?b.pop/maxPop:0`). **`NaN` is falsy in JS**,
so those are not decoration: a `NaN` population is absorbed *at the place*
and contributes zero. A plain Rust read of the same `f64` field would carry
it forward, and one bad settlement would turn its faction's entire row --
population, tax, sector output, all five power axes -- into `NaN`s the
reference never produces. Both coercions are ported (`js_num_or_zero`,
`js_truthy_num`) with a unit test showing the absorbed case.

This is the same class of asymmetry `cartalith-rust-conventions` already
flags for comparison operators, and the reason the power clamp uses
`js_min`/`js_max` rather than `f64::min`/`f64::max` -- though note the
consequence, disclosed rather than hidden: *because* the `||0` coercions
land first, no `NaN` can actually reach that clamp through any
caller-supplied field, so the clamp's NaN behaviour is proved by direct unit
tests on `js_min`/`js_max` rather than through the aggregate.

### The resource-residency tension: it does not bind here

`ECONOMY_SCOPE.md`'s original write-up expected this milestone to force the
memory question — "extend the delayed-free pattern" so the six freed fields
(clay/buildstone/flint/obsidian/sulfur/alum) survive past `assign_territory`.
Checked against what the code actually does rather than assumed:
at the time of this pass `compute_civilisation()` still freed those six at
`cartalith-godot/src/lib.rs`, immediately *before* `assign_territory` — so the
tension remained live **for any caller that wants the resource means**.

It does not bind on this milestone, for a reason that is structural rather
than convenient: **the half of `_civFactionAggregates` that unblocks
`civ_culture_terrain_fit` needs no resource field at all.** Territory Fit is
computed from field / biome / flow / ocean-distance / territory. So
`resources` is an `Option`, which is not a Rust convenience but a direct port
of the reference's own nullable `pots` (`const pots=(typeof
currentResourcePotentials==='function')?currentResourcePotentials():null`,
with every use guarded by `if(pots)`), and its absent branch is a real,
tested path: world sums stay zero, every faction's resource mean is zero, and
the trade rule correctly claims nothing.

That leaves the decision where it belongs — with whoever adds a real caller.
If that caller wants the resource means, the change is exactly one thing:
move `compute_civilisation()`'s six-field free from above `assign_territory`
to below it, extending the same bounded, measured tradeoff the previous pass
already made once (those six fields, ~96 MB at 2048×2048, stay resident
across one more Dijkstra; steady-state after the function returns is
unchanged either way). This pass deliberately did **not** make that change on
speculation, per the standing "don't wire in what nothing calls" discipline —
extending a memory cost for a consumer that did not exist would have been
paying for it twice over.

### The concrete unblock: `civ_culture_terrain_fit` gained real inputs

That is what this pass bought, and it was proved rather than asserted.
The golden test calls
`civ_culture_terrain_fit(culture, &agg.by_faction[f].terrain_mix,
&agg.world_mean_terrain)` for all seven cultures × all seven factions in both
fixtures, straight off the aggregate output, and compares `key`/`value`/
`world_mean`/`ratio`/`verdict` against the reference's own
`_civCultureTerrainFit` over the same aggregates. `common`/`imperial`
correctly return `None` on both sides. `terrain_mix` is a
`HashMap<&'static str, f64>` precisely so it drops into the existing
signature with no adapter.

This pass deliberately stopped short of a `#[func]`, under the UI hold the
owner called on 2026-08-18 — **a hold he lifted later the same day**
(`DCC_SHELL_SCOPE.md`), so it is not a live constraint and nothing here should
be read as one. What this pass changed is that the function has real inputs to
be called with: `GUI_FEATURE_PARITY_SCOPE.md`'s item 5 became a wiring job
rather than a blocked one.

### Verification

**Golden-parity**, `tests/golden_parity_faction_aggregates.rs`, two cases,
via a Node `vm.runInContext` harness over whole `<script>` blocks #1 and #2
asserted by their real `<script>`/`</script>` delimiters, with the standing
block-comment-balance check. That check earned its keep twice by being
**wrong** — a false "newline in regex" on `raw[i]/=cRange` (a `/` after `]`
is division), then a false one inside `_jpPackRange`'s hint builder, where a
`${...}` substitution was being closed by the first `}` inside it, so an
IIFE's `try{...}` ended the substitution early and the rest of the template
literal was scanned as code. Fixed with a per-substitution brace-depth
counter, not by deleting the check. Both blocks are additionally compiled
with `new vm.Script(...)` first — a real parser is a stronger slice-boundary
guarantee than any hand-rolled scanner.

Six input hashes (field, biome raster, lithology, water access, ocean
distance transform, river mask) and the territory raster match **exactly**.
Two do not, and this is disclosed rather than papered over: `tempField`/
`rainField` differ from this port's own by 1–3 f32 ULP in a minority of cells
(case 0: 1 temperature cell of 432, 178 rainfall cells, max relative 2.7e-7)
— a **pre-existing** property of the climate chain, entirely upstream of this
milestone, which propagates into carrying capacity, NPP, population density
and the resource potentials. It changes nothing categorical: no river cell
crosses the flow threshold, no biome class changes, no lithology class
changes (all three hashes are exact). So density and flow are compared by
land-cell sum at 1e-6 relative, resource means at 1e-6, the two
`Math.round`ed density sums to ±1, and **everything else** — populations,
trade volumes, tax, means, counts, capital pick, territory area, the full
terrain mix, all five power axes, the sector split, and the export/import/
strategic lists — at 1e-9 or exactly.

Fixture shapes reach the edges on purpose: a faction with neither territory
nor settlements, a faction with territory but no settlement, a faction with
exactly one settlement, a zero-population hamlet, an unmapped specialisation,
an out-of-range faction id, and (both cases) a faction whose territory spans
the x=0/x=gw−1 seam — case 1 additionally has settlements on both sides of it.
Non-emptiness is asserted explicitly (land count, per-faction territory-cell
counts, `territory.iter().any(|t| t > 0)`).

**15 unit tests** cover what a golden built from a real generated world
cannot reach: `NaN` absorption at the place (`p.pop||0`) and NaN propagation
through the power clamp (`js_min`/`js_max` vs `f64::min`/`f64::max` — Rust's
own would turn an unusable input into a confident-looking full-strength
score); the pre-world guard including the reference's own
`worldMeanResource == {}` / `worldMeanTerrain` zero-filled asymmetry; a
wrong-length territory raster treated as absent; territory ids at or past the
faction count; `Math.round`'s round-half-**up** on a negative `foodSurplus`;
the `Math.max(1e-6, 1-sea)` elevation-denominator floor at a near-ceiling sea
level; the absent-resource-field path; the religion flag and its weights
(every fresh world is all-`'none'`, so no fixture can reach the other
branch); the capital seat-tier preference and its tie-break; the craft fold;
`FactionPlace::from_settlement`'s defaults; and `civ_ocean_dist_field`'s
ocean-only-vs-fallback distinction.

**Mutation testing**: 58 mutations across the new
constants and branches, each applied to a unique **code-only** anchor
(checked to occur exactly once outside any comment line -- the
"pattern matched inside a comment" trap), each run **alone with a full
rebuild**, never as a combined sweep, because a stale binary reports a
healthy `N passed`. **56 killed.**

The first pass's six survivors were not six equivalent mutants. **Four were
real fixture gaps**, closed with new unit tests and then re-killed:

1. The religious axis's `0.7/0.3` weights were invisible because the only
   fixture exercising them had both normalisers saturating to 1, where
   `0.7+0.3` and `0.6+0.4` are the same number. Fixed with unequal
   populations.
2. The territory guard's **upper** bound (`f >= nF`) was never exercised --
   the synthetic raster only ever assigns valid ids. Fixed with a raster
   containing `nF` itself and a far-out id.
3. `Math.round` is round-half-**up** (toward +inf); Rust's `f64::round` is
   round-half-away-from-zero. They differ only on a negative half, and
   `foodSurplus` is the only rounded value here that can go negative. No
   generated world lands on an exact half; a unit test now does.
4. The `Math.max(1e-6, 1-sea)` elevation-denominator floor never activates
   at a real sea level. Fixed with a near-ceiling `sea` where it does.

**Two are genuine equivalent mutants**, and both were *proved* genuinely
tested with discriminating variants rather than accepted on assertion:
`coast <= 1.5 -> 1.6` cannot change anything, because a chamfer distance is
a sum of 1s and sqrt(2)s and `(1.5, 1.6]` is empty (`1.4` and `2.5` both
kill); `flow > thresh -> >=` cannot, because no accumulated discharge lands
exactly on the threshold (`x2` and `/2` both kill).

Two further mutations reported **stale anchors** rather than results --
caused by this milestone's own mid-sweep addition of the `||0` coercions,
which renamed the lines they targeted. Re-run against the corrected anchors;
both killed.

This pass stopped at the crate boundary — it left `compute_civilisation()`
untouched and added no `#[func]` and no GDScript — per the standing "don't
wire in what nothing calls" rule and the since-lifted UI hold.

## Military manpower: the economy layer's first real consumer (2026-08-25)

Not an economy milestone, but it extends this document's territory and belongs
recorded here rather than only in the register. `MILITARY_MANPOWER_SCOPE.md`
carries the owner's specification verbatim and the full derivation; this is
what it means for the economy work.

**It is the first thing that reads the food chain end to end.** Everything
this document built or listed as future scope — `civ_current_agrarian_density`
and its "Land sustains ≈ N" integral, `civ_faction_aggregates`' per-faction
population and territory, `civ_catchment_pop`'s tier tables, IN-13's
`RoadComponents` and `place_navigability` — is now read by one model that
turns them into four headcounts. Before this, the food half of the layer fed
settlement sizing and nothing else.

**And it closes the `farmersPerUrbanite` gap this document's own successor
opened.** `roster.rs`' module doc recorded that `AG_TECH_LEVELS` was ported
with no consumer, because *"`_civFoodShed`/`foodSurplusRatio` — the only two
functions that read them — are not ported"*. `_civFoodShed` **is** ported now
(IN-13, `cartalith_civ::trade`), and `farmersPerUrbanite` has a consumer for a
different reason: it is the agricultural labour ratio, which is the variable
the whole manpower model turns on. `CIV_GOVERNMENTS` got its first consumer in
either codebase at the same time.

**One real number this pass produced that the economy layer should note.** On
a real 233-settlement world, five of six factions' territory sustains **at
least twice** the population the settlement layer puts on it — the manpower
model's `ecological_factor` hits its `2.0` ceiling for all five. That is the
same divergence `civ_agrarian_regional_total`'s own readout has always shown
between "Land sustains ≈ N" and the settled total, quantified per faction for
the first time. Whether generated worlds should be more densely populated
relative to their carrying capacity is a real question for whoever revisits
`civ_settlement_population`'s surplus fractions, and it is older than this
pass.

**Three further items** — `_civPlaceSmelting`, `_civSaltAccess`, and
`_civFactionAggregates`' resource- and density-fed half as a *surfaced*
readout — are defined above and tracked in `cartalith-native/docs/STATUS.md`.
(This paragraph used to assert that no pass had taken any of them up. That was
a status claim in a document that does not track status, and it is the kind of
leftover `CLAUDE.md` says to fix rather than believe; the claim is removed,
not restated with a newer answer.)
