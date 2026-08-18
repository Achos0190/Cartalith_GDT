# Journey Planner: milestone plan

`ECONOMY_SCOPE.md`'s investigation confirmed `ROADMAP.md`'s own warning
accurate: the Journey Planner (`jp*`/`_jp*`, reference `Cartalith Gen1
v2.10.html` lines ~17300-20400) is ~70 real functions, genuinely comparable
in size to this port's entire civ-layer effort to date (15+ milestones).
This doc breaks it into real, sequential milestones, the same discipline
`CPU_MULTITHREADING_SCOPE.md`/`GPU_LAYER_INTEGRATION_SCOPE.md`/
`LOD_TILING_BASE_SCOPE.md` already applied to their own subsystems.

## Milestone 1 — physical-modeling primitives + seasonal/closure cluster: done

The two fully self-contained categories: no dependency on a `plan`/route/
vessel context object, no dependency on any other unported `jp*` function.

- **Physical-modeling primitives** (`jp_fatigue`, `jp_load_penalty`,
  `jp_surface_gain`, `jp_can_use_wheels`) — tiny table/threshold functions.
  `JP_LOAD_INVALID_RATIO` (v1.63's "above this ratio the stage is
  infeasible, not just slow" fix) ported alongside `jp_load_penalty` as the
  constant it's defined against.
- **Seasonal/closure logic** (`jp_season_at`, `jp_rest_days`,
  `jp_seasonal_closure`, `jp_sea_closure`) — the reference's own "v1.52: the
  four items v1.43/v1.49/v1.51 each deferred" block (rest-day/travel-day
  split, season drift over long journeys, sea-lane winter closure, mountain-
  pass winter closure). All four are real, sourced, historically-grounded
  fixes per the reference's own extensive comments — ported faithfully, not
  redesigned.

22 real unit tests (`cartalith-civ/src/lib.rs`, `-- jp_` test filter) — every
graduated band, every closure gate, the season-wrap-around case, the
animal-paced-vs-foot distinction in both `jp_surface_gain` and
`jp_rest_days`' long-haul tightening. No golden-parity harness needed: same
precedent as `civ_resource_trade_balance`/`civ_culture_terrain_fit` — small,
pure, branch-complete functions with no RNG/iteration-order risk.

**Not wired to any caller yet** — same "ship the primitive ahead of the
orchestration" precedent this session has used repeatedly. `jpJourneyCost`
and the actual route/plan orchestration (milestones 2+ below) are what would
call these.

## Milestone 2 — transport mode selection: done (2026-08-17); **fully complete 2026-08-18**

**Its last two landed with milestone 6's pass**, where the build-order table
sent them: `jp_auto_pick_transport` and `_jp_best_package_for_stage` are
ported and golden-verified. (`jp_auto_pick_vessel` had already shipped with
milestone 5 and `_jp_best_land_transport_for_stage` with milestone 4.) Nothing
from milestone 2 remains outstanding. The section below is the original
write-up, unedited apart from this note.

**Real finding, not assumed**: of the 10 functions this milestone originally
listed, four turned out to have a genuine, load-bearing dependency this
scope doc hadn't confirmed by reading the actual reference code —
`jpAutoPickTransport` (line 17814) and `jpAutoPickVessel` (line 18012) both
open by calling `_jpEnsurePlan(jn)` then `_jpDeriveStages(jn,plan)` —
**milestone 5's own route/stage derivation**, not yet started and flagged in
this doc as "almost certainly the largest single milestone in this whole
plan." `_jpBestLandTransportForStage` (line 18053) calls `jpCalcLand`
(milestone 3, physical travel cost, also not started); `_jpBestPackageForStage`
(line 18080) takes an `eff` parameter shaped like milestone 5's
`_jpEffectiveStagePlan` output. Porting any of the four now would mean
inventing the shape of data two unbuilt milestones haven't defined yet — so
they stay unported, re-flagged below under milestones 5/3 respectively
rather than silently dropped.

The other six were genuinely self-contained given a caller-supplied stage
list instead of the full JS `plan`/`jn` orchestration object, and are what
shipped: `jpBestAnimalForContext`, `jpPickSpeciesForRoute`, `jpResolveMount`,
`jpVesselMatrix`, `_jpVesselFits`, `_jpAutoStageVessel` — plus their real
supporting data (`JP_ANIMALS`, `JP_ANIMAL_TERRAIN_OVERRIDE`, `JP_TERRAIN`'s
land/river/sea tables, `JP_SHIPS`, `JP_VESSEL_PREFERENCE`, `JP_WATER_WINDOW`)
and the small pure helpers they call (`jpAnimalTerrainMod`, `jpWaterWindow`,
`_jpVesselWaterBlock`, `jpVesselDayKm`).

**The real biome-mapping design question, resolved by the reference itself,
not invented here**: this scope doc originally worried the reference's
`biome.desertLike`/`biome.bestAnimals` objects wouldn't map onto this port's
`u8` `classify_biome` output. They don't need a new lookup table — the
reference already has one. `jpLegacyBiomeOf` (reference line 18310, its own
`bIdx===13` "Hills" branch) calls `classifyBiome(T,M)` and maps its output
keys (`ice`/`tundra`/`boreal`/`conifer`/`tempForest`/`tempRain`/`grass`/
`savanna`/`shrub`/`desert`/`tropDry`/`tropWet`) onto `JP_BIOMES`' own legacy
V1.915 names — and those keys are, confirmed by reading both side by side,
the exact same climate-biome scheme this port's `classify_biome`
(`cartalith-civ`) already golden-verifies against. Ported as `jp_biome_key`
(`biome_id: u8, temp_c: f64) -> &'static str`), a direct transcription of
the reference's own fallback table, including its `desert` → `T<10 ?
"Cold Desert / Badlands" : "Hot Desert"` split. Water biomes (`BIOME_OCEAN`/
`BIOME_LAKE`) have no JP land-biome meaning and fall through to the
reference's own default (`"Temperate Forest"`), same as the reference does.

`jp_can_use_wheels` was already ported in milestone 1 (bundled there as a
natural fit alongside `jp_surface_gain`, even though this doc's own original
milestone-2 list didn't separately call that out) — not re-ported.

15 real unit tests (`cartalith-civ/src/lib.rs`), same "no golden harness
needed" precedent as milestone 1 — small, pure, branch-complete functions.
Notably includes a real bottleneck-veto case (`jpPickSpeciesForRoute`'s
v1.50 fix: a route mostly plains with one real mountain-pass stretch
switches the whole route's animal choice, hand-verified against the
reference's own `JP_BOTTLENECK_PENALTY`/`JP_BOTTLENECK_MIN_SHARE`
arithmetic) and a hand-computed vessel speed (`jpVesselDayKm`: Cog on
Coastal Waters = 10 × 11 × 0.60 = 66.0 km/day).

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (127 tests, 0 failures, 15 new), `cargo clippy -p cartalith-civ --lib`
clean (one real `collapsible_if` in the new `jp_vessel_matrix` code, fixed;
the two remaining lib warnings are pre-existing, unrelated to this
milestone), `cargo test --workspace` (0 regressions). **Not wired to any
caller** — same "ship the primitive ahead of the orchestration" precedent
milestone 1 and `civ_resource_trade_balance`/`civ_culture_terrain_fit` all
set; no `#[func]`, no `compute_civilisation()` integration, per this doc's
own "Out of scope for all milestones" section below.

## Milestone 3 — physical travel cost: done (2026-08-17); **fully complete 2026-08-18**

**Its two deferrals landed with milestone 4**, which is where the build-order
table below sent them: `jp_calc_land`/`jp_calc_water` are ported and
golden-verified. Nothing from milestone 3 remains outstanding. The section
below is the original write-up, unedited apart from this note.



**The single biggest real finding of this pass is a dependency-ordering
error in this document itself**: milestone 3 is ordered *before* milestone 4,
and it should be the other way round. `jpCalcLand` (reference line 18912)
calls `jpCapacity` (18177), `jpForaging` (18156), `jpAssessResupply` (18231)
and `_jpDesertTierForGap` (18727); `jpCalcWater` (19124) calls
`jpAssessResupply` and `jpHumanWaterRate` (17626). **Every one of those is on
milestone 4's own list.** They are not thin shims either — `jpCapacity` is
the whole seasonal-physiology / draft-shortfall / mount-saddlebag mass model,
and `jpForaging` reaches through `_jpWildlifeForageMod` (18134) into the
world's wildlife-richness field, real world context this port has not plumbed
into the Journey Planner at all. So `jp_calc_land`/`jp_calc_water` stay
unported here, re-flagged under milestone 4 below rather than silently
dropped, on exactly the discipline milestone 2 used for its own four.
**Milestone 4 must land before milestone 3's two stage calculators can.**

Of the eleven functions this doc listed for milestone 3, two were already
shipped by milestone 2, which needed them for its own work and said so:
`jp_water_window` and `jp_animal_terrain_mod`. Not re-ported.

The remaining **seven shipped**, all genuinely self-contained given a
caller-supplied party/leg summary instead of the full JS `plan`/`jn`
orchestration object: `jp_train_pace`, `jp_sail_factor`, `jp_wx_weighted`,
`jp_weather_factor`, `jp_column_length_km`, `jp_column_factor` and
`jp_journey_cost` — plus the real data they read (`JP_TRAIN_PACE`, `JP_RIG` /
`JP_SHIP_RIG` sail polars, `JP_WEATHER`, `JP_ANIMAL_WEATHER_OVERRIDE`,
`JP_FILES_BY_TERRAIN` and the column-spacing constants, `JP_COST_*`) and one
small shared `JpParty` struct for the plan fields all three of the
party-shaped functions read.

**The `JP_BIOMES[...].weather` flag this doc raised, checked rather than
assumed**: it was **not** ported. Milestone 2 deliberately narrowed its
`JP_BIOMES` port to the two fields `jpBestAnimalForContext` reads
(`desertLike`/`bestAnimals`) and said so in its own doc comment. The weather
distributions (12 biomes × 4 seasons × 5 conditions) are ported here,
alongside the two functions that consume them. The remaining `JP_BIOMES`
columns (`water`/`forage`/`waterForage`/`grazing`) are still unported and
belong to milestone 4.

**`jp_journey_cost` turned out portable**, confirmed by reading its real
signature rather than assuming: the reference's own comment calls it "pure
over the plan object — no globals, no DOM", and that held up. The fields it
actually touches are a small, stable per-leg summary (`cat`/`st.km`/`days`/
`crew`/`blocked`), one `claimedFrac` per stage, the trip totals and the party
composition. None of that needs milestone 5 to have run — the caller supplies
it, the same way milestone 2's functions take a caller-supplied stage list.
Ported with a `JourneyLeg` input struct narrowed to exactly those five
fields, which is also the shape `jp_calc_land`/`jp_calc_water` will produce
when milestone 4 unblocks them.

**Milestone 2's four deferrals: none resolved.** Re-checked by reading each
one again, not inferred. `_jpBestLandTransportForStage` (18053) calls
`jpCalcLand` in its inner loop — and `jpCalcLand` did *not* land this
milestone, so it stays blocked, now behind milestone 4 rather than
milestone 3. `jpAutoPickTransport` (17814) and `jpAutoPickVessel` (18012)
still open with `_jpEnsurePlan(jn)` + `_jpDeriveStages(jn,plan)` (milestone
5); `jpAutoPickTransport` additionally does `jpCapacity`-shaped mass
arithmetic inline, so it needs milestone 4 too. `_jpBestPackageForStage`
(18080) still takes an `eff` shaped like milestone 5's
`_jpEffectiveStagePlan` output.

**Golden-verified against the real reference**, not hand arithmetic: the
reference's own source lines for all seven functions and their tables were
sliced out of `reference/Cartalith Gen1 v2.10.html` by line range and
evaluated in a bare Node `vm.runInContext` with no DOM — the same harness
technique Phase 2 used throughout, applied to functions pure enough not to
need a whole generated world to drive them. Every expected value in the 12
new tests is that run's output, including all 48 `jpWxWeighted` cells
(12 biomes × 4 seasons) checked as a block, the sail polar's five control
points plus interpolation and angle-folding (−90°/270°/400°), and two full
`jpJourneyCost` breakdowns. One real harness bug found and fixed before
trusting any of it: an unterminated block comment at a slice boundary was
swallowing the next slice.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (139 passed, 0 failed, 12 new), `cargo clippy -p cartalith-civ
--all-targets` (two real findings in the new code fixed — a `manual_clamp`
and an `inconsistent_digit_grouping`; the lib is back to the same two
pre-existing, unrelated warnings milestone 2 recorded, and the new test code
adds none), `cargo test --workspace` (0 regressions). **Not wired to any
caller** — no `#[func]`, no `compute_civilisation()` integration, per this
doc's own "Out of scope for all milestones" section.

## Milestone 4 — consumption/resupply: done (2026-08-18)

Built first, per the build-order table below — and it closed milestone 3 and
part of milestone 2 on the way, exactly as that table predicted.

**All thirteen of this milestone's own functions shipped.** The four the doc
called real quick wins went first and were exactly that: `jp_human_water_rate`,
`jp_human_water_carry_days`, `jp_animal_water_carry_days`,
`jp_desert_tier_for_gap`. Then `jp_consumption_factors`, `jp_foraging`,
`jp_capacity` (the whole seasonal-physiology / desert-multiplier /
phantom-draft-shortfall / mount-saddlebag mass model), `jp_assess_resupply`,
`jp_world_mean_richness`, `jp_wildlife_forage_mod`, `jp_resupply_reach`,
`jp_drinking_coarse_ease`, `jp_stage_dry_km` — plus `jp_water_reach_cells`,
which this doc lists under milestone 5 but which `_jpStageDryKm` calls, so it
came along here. All the data milestones 2 and 3 deliberately left out is
ported too (`JP_BIOMES`' `water`/`forage`/`waterForage`/`grazing`, the four
seasonal/grazing tables, and `JP_PACE`/`JP_INFRA`/`JP_ROUTE`/
`JP_GROUP_CLASSES`/`JP_LAND_TRANSPORTS`/`JP_DESERT_WATER`/the vehicle and
ration constants, none of which had been needed before). Milestone 2's
two-field `JP_BIOMES` lookup now delegates to the one full biome record rather
than keeping a second copy of the same table.

**The four things this doc assigns here rather than to their own milestones,
all four done:**

1. **`jp_calc_land`/`jp_calc_water`** — milestone 3 is now fully complete.
   Both return `Result<_, JpBlocked>` instead of the reference's
   `{blocked:"…"}` sentinel, so a blocked stage cannot be read as a computed
   one by accident. Their `formula` trace strings are deliberately **not**
   ported: pure presentation (`ARCHITECTURE.md`), and every value they print
   is a field on the returned struct.
2. **`jp_fmt_kg`** (milestone 6's) — ported; the rest of milestone 6 untouched.
3. **`_jp_best_land_transport_for_stage`** (milestone 2's) — **genuinely
   unblocked, confirmed by reading the real code rather than trusting this
   doc**: its `eff` parameter is only ever a plan with per-stage overrides
   merged in (`_jpEffectiveStagePlan` is a plain field merge), so
   `jp_calc_land` landing was all it needed. Ported. Milestone 2's other
   three are still blocked on milestone 5's plan/stage derivation.
4. The `JP_BIOMES` columns and seasonal tables — see above.

**The genuinely hard piece, resolved by investigation rather than
transcription.** `jp_foraging` reads the world's wildlife *richness* through
`_jpWildlifeForageMod`. This port's Phase 2 ecology work was checked first,
rather than assuming new plumbing: `build_npp` and `build_carrying_capacity`
(`cartalith-civ`) are real and are genuine *inputs* to the reference's own
richness model — but they are not the same quantity. `richness` is a
per-ecoregion **species count** (`assignWildlife`'s `present.length`: a biome
species roster clipped by `regionRichness`'s species-area × energy ×
heterogeneity × latitude curve), and the ecoregion-segmentation + species-
roster subsystem behind it (`buildEcoregions`/`regionRichness`/
`assignWildlife`/`WILD_ROSTERS`) is unported, on **no** milestone in this
document, and larger than this one.

So the input is genuinely new, and it is **caller-supplied**, matching
`civ_resource_trade_balance`'s caller-supplied-means precedent rather than
reaching into world state: `jp_wildlife_forage_mod(region_richness,
world_mean_richness)` and `jp_world_mean_richness(&[Option<f64>])` are pure
ports of the reference's own arithmetic, and `JpStage` carries the finished
multiplier in the slot where the reference carries `mx`/`my`. **This costs
nothing in fidelity**, because the reference's own calibration anchor is that
`_jpWildlifeForageMod` returns exactly 1.0 when wildlife data is unavailable —
and 1.0 is also what an exactly-average region produces. A port with no
ecoregion model therefore behaves identically to the reference running on a
world whose wildlife layer was never built, and the flat `JP_BIOMES.forage`
table stays the anchor it was designed to be.

**Golden-verified against the real reference**, same harness technique
milestone 3 introduced and the same care about its known failure mode:
reference lines 17297-19252 were sliced out of `reference/Cartalith Gen1
v2.10.html` as **one contiguous slice** and evaluated in a bare Node
`vm.runInContext` with no DOM, with a block-comment balance check on the slice
boundaries (milestone 3's harness bug was an unterminated block comment at a
slice boundary; one contiguous slice plus a balance assert removes the whole
class). Two boundary corrections were needed and caught by it. Every expected
value in the 26 new tests is that run's output — all eight `jpCapacity`
configurations field by field, all eleven `jpCalcLand` cases and all seven
`jpCalcWater` cases including their exact verdict and blocked-message strings,
the `_jpStageDryKm` transect and the `_jpResupplyReach` gap measurement.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (165 passed, 0 failed, 26 new), `cargo clippy -p cartalith-civ
--all-targets` (clean; the two remaining lib warnings are the same
pre-existing ones milestones 2 and 3 recorded), `cargo test --workspace` (0
regressions). One new workspace dependency: `cartalith-civ` → `cartalith-
terrain`, for `river_coarse_ease`, which `jp_stage_dry_km` divides back out to
substitute JP's own uncapped ease. **Not wired to any caller** — no `#[func]`,
no `compute_civilisation()` integration, per this doc's own "Out of scope for
all milestones".

## Milestone 5 — route/stage derivation: done (2026-08-18), as three sub-milestones

This doc called it "almost certainly the largest single milestone in this
whole plan" and that held: it did not survive as one flat pass. It is
recorded here as the three sub-milestones the real code actually falls into,
the same honesty milestones 2, 3 and 4 each applied when their own scope
assumptions met the reference:

- **5a — world sampling.** Everything that turns the world into per-stage
  facts, and nothing that needs a route chunked yet: `_jp_road_cells` (with
  `_civWalkWayCells`), `_jp_infra_context`, `_jp_claimed_at`,
  `_jp_stage_infra`, `_jp_river_condition`, `_jp_sea_condition`,
  `_jp_coarse_idx`, `_jp_stop_key`, `_jp_mode_for_route`,
  `_civ_transshipments`/`_civ_transfer_overhead`, `_civ_passed_settlements`.
- **5b — `_jp_derive_stages`.** The chunker itself: per-point classification,
  contiguous-run chunking with wrap-aware km and metres of gain/loss, the
  narrow-water-gap collapse, the sliver absorb, the 14-stage cap, and then
  the per-stage settlement/claimed/dry-km/midpoint measurement and
  infra/route-condition resolution.
- **5c — `_jp_plan`.** The orchestrator: `_jp_effective_stage_plan`,
  `_jp_ensure_plan`, the v1.52 season-drift pre-pass, the per-stage vessel
  fallback, the supply forecast, the hazards, the elevation profile, the
  daily timeline, and the roll-up.

All three shipped in one pass (~1,150 lines of `cartalith-civ` plus tests),
so the split is a description of the work rather than a schedule — but the
sub-milestone boundaries are real, and 5c genuinely cannot be attempted
before 5b, which genuinely cannot be attempted before 5a.

**Four functions this milestone needed that are on no milestone list here.**
Found by reading, picked up rather than stubbed:

1. **`buildCartBiome`/`buildCartTerrain`** (reference lines 6817/6862) — the
   Cartalith 15-biome and 13-terrain paint layers. `_jpDeriveStages` samples
   *both* on every route point, and **this port had never built either**.
   The existing `build_biome_raster` is the *climate* raster, a different
   vocabulary (`cartalith-assets` already documents that distinction). Ported
   as `build_cart_biome`/`build_cart_terrain` alongside `CART_BIOMES`/
   `CART_TERRAINS` and `jp_legacy_biome_of`. They are small, pure, and read
   only fields this port already produces. One ordering detail was checked
   rather than assumed and would have silently broken the port if it had
   not been: `ELEV_TO_CART` is indexed by `BIOME_INDEX`, whose order puts
   **shrub before savanna** — which is exactly this port's own `BIOME_*`
   numbering, so the table transfers unchanged.
2. **`_civTransshipments`/`_civTransferOverhead`** (19198/19205) — as this
   doc predicted, `jp_journey_cost` takes the count as an argument and
   nothing produced it.
3. **`_civWalkWayCells`** (21766) — `_jpRoadCells` cannot exist without it.
4. **`_civPassedSettlements`** (21154) — `_jpPlan`'s stops list.

**Three functions on this milestone's own list that are deliberately *not*
Rust functions, each for a real reason rather than an omission:**

- **`_jp_layovers`** is a JS lazy-init idiom ("give me `jn.layovers`,
  creating the object if this journey predates the field"). A
  `HashMap<String, i64>` keyed by `jp_stop_key` needs no such function;
  shipped as the `JpLayovers` type alias, and `jp_plan` takes one.
- **`_jp_settlements`** is `state.places.filter(p => CIV_SETTLE_KEYS.has(
  p.kind))` — a *runtime* type test the reference needs because
  `state.places` is one untyped array of everything on the map. This port's
  settlements come out of `place_settlements`/`name_and_populate_settlements`/
  `civ_seed_villages` already typed as settlements, so the filter has no work
  left to do. Building the `JpPlace` list *is* the filter.
- **`_jp_reroute_for_mode`** was genuinely **blocked**, and this was the one
  real carry-over out of milestone 5. (**Unblocked 2026-08-18** — see
  "Closing status".) Its whole body is `_civDijkstraPath(s,
  e, domain)` — and `_civDijkstraPath` (25957) with `_civWaterCostGrid`
  (21051) and `_civMixedCostGrid` (21090) were **unported**, on no milestone
  in this document, and are the interactive Route tool's own multi-modal
  pathfinder rather than anything the Journey Planner owns. Re-routing is
  also a UI action ("Re-route land-only" button), so it sits on the far side
  of this doc's own "Out of scope for all milestones" line twice over. Its
  pure half, `_jp_mode_for_route` (transport → cost domain), **is** ported —
  that is the part that carries the reference's real reasoning (the
  disclosed "River Transport prefers rivers, does not require them" scope
  cut).

`_jp_water_reach_cells` was re-checked before any work: already shipped with
milestone 4, as this doc's own strikethrough says. Not re-ported.

**How the plan/stage shapes resolved against milestone 4's `JpStage`** — the
requirement this doc wrote down in advance, answered by building it:

- `_jpDeriveStages` produces a new `JpDerivedStage`, which **does** carry the
  reference's `mx`/`my`, because they are a genuine map measurement made
  here. `JpStage` (milestone 4's) correctly does **not**: what the *stage
  calculators* consume is the finished multiplier the unported
  ecoregion/species-richness subsystem would produce from those coordinates.
  `JpDerivedStage::to_stage(wildlife_forage_mod)` is the bridge, and
  `jp_plan` takes a `&dyn Fn(f64, f64) -> f64` in exactly the reference's own
  `_jpWildlifeForageMod(mx, my)` position. `|_, _| 1.0` is the reference's
  own answer on a world with no wildlife layer. No change to `JpStage` was
  needed — milestone 4 got the shape right.
- `JpPlan` gained the five `_jpEnsurePlan` defaults milestone 4 had not
  needed: `route_cond`, `infra`, `stage_overrides`, `season_drift`,
  `rest_cadence`, plus a new `JpStageOverride` for the sparse per-stage map.
  `jp_effective_stage_plan` is the reference's `Object.assign({},plan,ov)`
  with its per-species animal merge preserved.
- `_jpEnsurePlan`'s remaining debt (this doc's own point (e)) is paid:
  `jp_ensure_plan` derives the route's real stages and corrects its vessel
  guess through `jp_auto_pick_vessel`. That function is milestone 2's, and it
  is ported **here** because `_jpEnsurePlan` cannot exist without it — the
  same "port what this milestone genuinely needs" rule milestone 4 used for
  `jp_calc_land`.

**Two reference quirks reproduced as written rather than tidied**, both
recorded so nobody 'fixes' them later:

- `_jpDeriveStages` falls back to `state.mapWidthKm || 12000` while
  `_jpInfraContext`, two functions away, uses `|| 800`. Both are kept.
- `_jpRoadCells` keys its map by JS string concatenation (`x+','+y`), and
  `_civWalkWayCells` emits a way's *first* and seam-break points
  **unrounded** — so those writes produce keys like `"12.5,3"` that no
  integer lookup can ever hit. Reproduced by simply not recording a
  non-integral emission: same observable behaviour, no float keys.

**Golden-verified against the real reference.** Eight line ranges were sliced
out of `reference/Cartalith Gen1 v2.10.html` — `riverCoarseEase`/
`terrainDetailK` (2641-2675), `classifyBiome` (5736-5743), `BIOME_KEYS`/
`BIOME_INDEX` (6796-6797), the cart paint layers (6810-6877), the whole
Journey Planner (17297-19419), `_jpModeForRoute` (20368-20379),
`_civPassedSettlements` (21154-21175) and `_civWalkWayCells` (21766-21777) —
and evaluated in a bare Node `vm.runInContext` with no DOM. **Milestone 4's
block-comment balance assertion was applied per slice and earned its keep
again**: it caught **three** genuine boundary errors here (the
`riverCoarseEase`, cart-layer and `_civWalkWayCells` slices each ran one line
into the following comment block), and the JS parser caught a fourth (the
Journey Planner slice cut `_jpPlan`'s closing brace).

The world driven through it is synthetic but *exactly* reproducible: every
field is a closed form in `+ - * /` over exact values, with no transcendental
anywhere, so the Rust test rebuilds the identical `f32` grids rather than
embedding them and only the **outputs** are embedded. That world is a real
one for this layer's purposes — a 24x16 map with an ocean margin, a lake, a
mountain ridge, a river column, a highway, a reference-road spur, claimed
territory and five settlements, crossed by a 24-point route that derives into
seven stages (2 sea, 1 river, 4 land), one transshipment, a 41-day timeline
and a genuinely unmet resupply requirement.

19 new tests. Every expected value is that run's output: all seven stages
field by field (km, i0/i1, river crossings, gain/loss, settlements, claimed
fraction, dry km, midpoint), the whole `_jpPlan` roll-up (km, days, food/
water/fodder, hazards, ascent/descent, transshipment overhead, rest and total
days, per-leg days and speeds, the resupply-reach measurement, the timeline's
first/seventh/last day and their camps), plus isolated probes of every
5a helper.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ
--lib` (184 passed, 0 failed, 19 new), `cargo clippy -p cartalith-civ
--all-targets` (the new code adds **no** warnings; the remaining ones are
pre-existing and unrelated — two `needless_range_loop`s in `civ_sea_routes`
and some float-precision literals in two older golden test files), `cargo
test --workspace` (0 regressions). **Not wired to any caller** — no `#[func]`,
no `compute_civilisation()` integration, per this doc's own "Out of scope for
all milestones".

## Milestone 6 — verdict/reporting: done (2026-08-18), with milestone 2's remainder

Both of this document's last two open milestones landed in one pass, in the
order the build-order table set: 6 first (it needed milestone 5's plan output
to verify against), then 2's remainder.

**Milestone 6's own five:** `jp_verdict`/`JpVerdict`,
`jp_confidence`/`JpConfidence`, `jp_pack_range`/`JpPackRange`,
`jp_fmt_days`, and `jp_risk` — the campaign-duration advisory this doc's own
correction (b) assigned here rather than to milestone 5, because it is a verdict
string. `jp_fmt_kg` was re-checked before any work and had indeed shipped with
milestone 4, as planned; not re-ported.

All three of this milestone's recorded corrections held up on contact:

- (a) The plan object is `JpJourneyPlan`, every field `_jpVerdict` consults is
  on it, and the reference's own `blockedMsg` is read back off the blocking
  stage's `JpBlocked` rather than duplicated. `jp_verdict` takes
  `&JpJourneyPlan` and returns a `JpVerdict`, not an `Option` — the
  reference's `if(!plan) return null` guard has no Rust equivalent.
- (b) The `risk` advisory is ported here, as `jp_risk(days)`. `JpJourneyPlan`
  deliberately gained no `risk` field: it carries the day count, not the
  caption.
- (c) The harness was indeed cheap. Milestone 5's slice list already loaded
  everything, and one `jp_plan` call on the same synthetic world produced a
  real `JpJourneyPlan` to verify against.

One shape decision worth writing down: `_jpPackRange` reads `plan.plan` and
`plan.hasDesert` off the finished journey, but `JpJourneyPlan` does not carry
the party plan back out (the caller already owns it), so `jp_pack_range` takes
`(&JpPlan, has_desert: bool)`. Same information, no round-tripped field.

**Milestone 2's remainder:** `jp_auto_pick_transport`/`JpAutoTransport` and
`jp_best_package_for_stage`/`JpPackageFix`. Both were unblocked exactly as
milestone 5 predicted. `jp_auto_pick_transport` needed one thing milestone 5
had not built — `plan.autoPromote`, the last `_jpEnsurePlan` default nothing
had yet read — added as `JpPlan::auto_promote`.

**Their HTML hint strings are deliberately not ported.** Presentation is
Godot's (`ARCHITECTURE.md`), and this doc's own milestone-2 row already said
so: what ports is the structured decision. `JpAutoTransport` is an enum over
the reference's six real outcomes (no land stages / not a land mode / walking
within capacity / walking overloaded / mount picked / baggage train), carrying
every number the hints print — species pick, count, carts, wagons, the
auto-promote flag, and the `fodderInfeasible` divergence case.
`jp_auto_pick_transport` mutates the plan exactly as the reference mutates
`jn.plan`.

**A real bug in a shared helper, found by this milestone's own golden run and
not by reading.** `js_fixed` — milestone 4's reproduction of JS `toFixed`'s
round-half-*away-from-zero* tie-break, which this doc's own milestone-6 note
told this milestone to reuse rather than re-derive — decided the tie by
scaling: `(v*10^d + 0.5).floor()`. That **fabricates** ties. `61.5/30` is
`2.0499999999999998`, which JS renders `"2.0"`, but `2.0499999999999998 * 10`
rounds to exactly `20.5` in `f64` and the `+0.5` then carried it to `"2.1"`.
`jp_fmt_days(61.5)` is the case that caught it. Rewritten to decide the tie on
the value's **exact** decimal expansion — a double is a dyadic rational, so a
genuine tie at place `d+1` means the expansion ends in a 5 there — and to lean
on Rust's own `{:.N}` (already the correctly-rounded exact decimal) for
everything else. Verified against `Number.prototype.toFixed` on 30 cases
including the pairs that look identical and are not (`1.25` is a real tie,
`2.05` is not), and `jp_fmt_kg(1250.0)` = `"1.3 t"`, the tie that reaches a
user-visible string. **No existing test's expected value changed** — reusing
the helper is what exposed it, which is the argument for reusing it.

**Golden-verified against the real reference**, through milestone 5's harness
and fixture unchanged. Eight line ranges in a bare Node `vm.runInContext` with
no DOM, each carrying the **block-comment balance assertion** on its own
boundaries. All eight balanced first time, including the one that moved — the
Journey Planner slice extended from 17297-19419 to **17297-19532** to take
v1.49's verdict layer with it.

The harness did surface one error of a *different* class, which the balance
check is not designed to catch and the JS parser could not either: milestone
5's recorded slice `2641-2675` starts one line **below**
`TERRAIN_DETAIL_MAX_K` (line 2640), which `riverCoarseEase` reads — and
`_jpDeriveStages` catches its own exceptions and returns an empty stage list,
so the whole world silently derived to **zero stages** with no error printed
anywhere. Found by instrumenting that `catch`; the slice is now `2640-2675`.
Worth recording alongside the balance assertion as its known blind spot: the
assertion proves a slice is *syntactically* whole, not that it is
*semantically* self-sufficient.

The world, route and party are milestone 5's own, and reproduce its values
exactly (760.847480700888… km, 41.317750030325… days, seven stages, one
transshipment, a genuinely unmet resupply requirement). That last property is
why the verdict probes are shaped the way they are: the m5 route cannot reach
every band on its own — an unmet requirement alone forces `severe` — so each
band probe edits exactly the signals `_jpVerdict` reads on a **real** plan, and
the harness made the identical edits to the identical fields.

**Verified**: `cargo build -p cartalith-civ`, `cargo test -p cartalith-civ`
(194 passed, 0 failed, **10 new**) — every expected string and number is that
run's output, covering all five verdict levels and both Strained texts, all
fourteen contributing reasons, every `_jpConfidence` threshold from both sides,
`_jpPackRange` across species/grazing/desert, `jpFmtDays`' three unit bands and
its rounding edges, all four `risk` tiers, nine `jpAutoPickTransport`
configurations and thirteen `_jpBestPackageForStage` cases;
`cargo clippy -p cartalith-civ --all-targets` (the new code adds no warnings;
the remaining ones are the same pre-existing, unrelated set milestones 2-5 each
recorded); `cargo test --workspace` (0 regressions). **Not wired to any
caller** — no `#[func]`, no `compute_civilisation()` integration, per this
doc's own "Out of scope for all milestones".

## Original milestone breakdown (all six now done — kept for cross-reference)

### Build order — read this before the numbers below

**The milestone numbers below are historical identifiers, not a build
order.** They were assigned by `ECONOMY_SCOPE.md`'s original categorization,
before anyone had read the real reference code; milestone 3's own
investigation then found a genuine dependency inversion (its two stage
calculators need milestone 4's mass model). The numbers are deliberately
**not** renumbered — they are referenced by name across `CHANGELOG.md`,
`STATUS.md`, several commit messages, and this document's own prose, and
renumbering would invalidate all of it to fix a problem a single table
solves.

| Build order | Milestone | Why this position |
|---|---|---|
| ~~1st~~ **done** | ~~**4 — Consumption/resupply**~~ | Done 2026-08-18. It did unblock milestone 3's tail and part of milestone 2's, exactly as this row predicted; `jp_fmt_kg` came with it; the wildlife-richness piece was indeed the one real decision (see the milestone 4 section above). |
| ~~2nd~~ **done** | ~~**3 (tail) + 2 (partial)**~~ | Folded into 4's own verification pass, as this row said to. `jp_calc_land`/`jp_calc_water` and `_jp_best_land_transport_for_stage` all shipped there. |
| ~~3rd~~ **done** | ~~**5 — Route/stage derivation**~~ | Done 2026-08-18, as three sub-milestones (5a world sampling / 5b `_jpDeriveStages` / 5c `_jpPlan`) — see the milestone 5 section above. `_civTransshipments` came with it as predicted, along with three *other* helpers on no list (`buildCartBiome`/`buildCartTerrain`, `_civWalkWayCells`, `_civPassedSettlements`). One function carried over: `_jp_reroute_for_mode`, blocked on the unported interactive Route-tool pathfinder. |
| ~~4th~~ **done** | ~~**6 — Verdict/reporting**~~ | Done 2026-08-18. `_jpVerdict`/`_jpConfidence`/`_jpPackRange`/`jpFmtDays`, plus the `risk` advisory correction (b) below. It did need 5's plan output, exactly as this row said, and 5's own fixture drove it unchanged. |
| ~~5th~~ **done** | ~~**2 (remainder)**~~ | Done 2026-08-18, in the same pass as 6. Both were unblocked as this row predicted; `jp_auto_pick_transport` needed one `_jpEnsurePlan` default milestone 5 had not built (`plan.autoPromote`). |
| unscheduled | **`_jp_reroute_for_mode`** | Not a milestone of its own; it needed `_civDijkstraPath`/`_civWaterCostGrid`/`_civMixedCostGrid` (the Route tool's multi-modal pathfinder). **Unblocked 2026-08-18**: `UNIFIED_TOOL_PLAN.md` milestone D ported all three as `cartalith_civ::tools::civ_dijkstra_path`, with the `reachable` flag this function exists to check. What is left is a `reachable` check, a call and three field assignments — still a UI action, so still unscheduled here. |

**Every milestone is done**; see their own sections above. The one function
still unported, `_jp_reroute_for_mode`, was never on a milestone list — see
"Closing status" at the end of this document.

The numbered list that follows preserves the original numbering for
cross-reference:

2. ~~**Transport mode selection**~~ — see "Milestone 2" above. **Complete as
   of 2026-08-18.** The two functions this entry recorded as remaining shipped
   with milestone 6's pass; the history below is kept as written:
   - `_jp_best_land_transport_for_stage` shipped with milestone 4.
   - `jp_auto_pick_vessel` shipped with **milestone 5** — `_jpEnsurePlan`
     calls it on first plan creation, so milestone 5 could not be finished
     without it. It is `JP_VESSEL_PREFERENCE.find(jp_vessel_fits)` over the
     derived water stages, and `jp_vessel_fits` had been ported since
     milestone 2.
   - `jp_auto_pick_transport` (17814) needs `_jpEnsurePlan` +
     `_jpDeriveStages` (both now real) and `jpCapacity`-shaped mass
     arithmetic inline (milestone 4's, real): **unblocked, not ported**. Its
     hint strings are HTML and belong to Godot (`ARCHITECTURE.md`); what
     ports is the structured decision (species pick, count, carts/wagons,
     the auto-promote flag, and the reference's own analytically-detected
     `fodderInfeasible` divergence case).
   - `_jp_best_package_for_stage` (18080) turns out to need only a stage and
     an `eff` **plan** — no derived route at all, the same finding milestone
     4 made about `_jpBestLandTransportForStage`. Milestone 5 defining
     `jp_effective_stage_plan`'s output as simply a `JpPlan` is all it was
     waiting on: **unblocked, not ported**.

   Both shipped with milestone 6's pass (2026-08-18) — see "Milestone 6"
   below. Milestone 2 is complete.

3. ~~**Physical travel cost**~~ — see "Milestone 3" above. **Complete as of
   2026-08-18**: seven of eleven shipped in the milestone itself, two
   (`jp_water_window`, `jp_animal_terrain_mod`) had already shipped with
   milestone 2, and the last two (`jp_calc_land`/`jp_calc_water`) shipped
   with milestone 4, which this list orders after them — the real ordering
   error the build-order table above exists to fix.

4. ~~**Consumption/resupply**~~ — see "Milestone 4" above. All thirteen
   functions shipped, plus `jp_calc_land`/`jp_calc_water` (closing milestone
   3), `jp_fmt_kg` (from 6), `_jp_best_land_transport_for_stage` (from 2),
   `_jp_water_reach_cells` (listed under 5 below, but `_jpStageDryKm` calls
   it), and all the `JP_BIOMES`/seasonal data milestones 2 and 3 left out.
   The wildlife-richness question resolved as a **caller-supplied input**
   rather than new world-state plumbing — the reasoning, and why it costs no
   fidelity, is in the milestone 4 section above.

5. ~~**Route/stage derivation**~~ — see "Milestone 5" above. **Done
   2026-08-18** as three sub-milestones. All of the list below shipped except
   `_jp_reroute_for_mode` (blocked on the unported Route-tool pathfinder),
   plus `_jp_layovers` and `_jp_settlements`, which are JS idioms with no
   Rust function to write — reasons in the milestone 5 section. Four helpers
   on **no** list here came with it: `buildCartBiome`/`buildCartTerrain` (the
   paint layers `_jpDeriveStages` samples, never built by this port),
   `_civTransshipments`/`_civTransferOverhead` (predicted in (b) below),
   `_civWalkWayCells` and `_civPassedSettlements`. Requirement (a) resolved
   as `JpDerivedStage` carrying `mx`/`my` with `to_stage(wildlife_forage_mod)`
   bridging to milestone 4's `JpStage` — no change to `JpStage` was needed.
   Original list, for cross-reference: `_jp_derive_stages`, `_jp_effective_stage_
   plan`, `_jp_plan`, `_jp_ensure_plan`, `_jp_layovers`, `_jp_stop_key`,
   `_jp_road_cells`, `_jp_settlements`, `_jp_infra_context`,
   `_jp_claimed_at`, `_jp_stage_infra`, `_jp_river_condition`,
   `_jp_sea_condition`, `_jp_coarse_idx`, ~~`_jp_water_reach_cells`~~
   (shipped with milestone 4 — `_jpStageDryKm` calls it),
   `_jp_mode_for_route`, `_jp_reroute_for_mode`. The real orchestration
   layer — needs milestones 2-4 done first, needs this port's road network
   (`civ_hierarchical_network_topology`/`civ_consolidate_and_smooth_ways`,
   already real) and settlement data as real inputs. Almost certainly the
   largest single milestone in this whole plan. **Two concrete requirements
   milestone 3's reading pinned down**, worth writing here before they're
   re-derived: (a) the per-stage object this layer must produce is read by
   `jpCalcLand` as `{km, terrain, routeCond, infra, biome, cat, mx, my,
   dryKm, claimedFrac}` — `mx`/`my` are map cell coordinates (for
   `jp_foraging`'s wildlife lookup) and `dryKm` is the stage's own measured
   longest waterless run, both genuine map measurements, not plan fields;
   (b) `jp_journey_cost` (already ported) additionally wants a transshipment
   count, which the reference computes with `_civTransshipments` (line
   ~18906) — a small `_civ*` helper that appears on **no** milestone list
   here and should be picked up alongside this one.

   **Three further requirements milestone 4's own reading pinned down**, all
   about what this layer must now *feed* rather than port: (c) the per-stage
   object is the already-existing `JpStage` struct (`cartalith-civ`), and its
   `wildlife_forage_mod` field is where the reference's `st.mx`/`st.my`
   lookup goes — supplying a real value needs the **unported ecoregion/
   species-richness subsystem** (`buildEcoregions`/`regionRichness`/
   `assignWildlife`/`WILD_ROSTERS`), which is on no milestone in this
   document and is its own body of work; until it exists, 1.0 is the
   reference's own correct answer, so this is a quality ceiling, not a
   blocker. (d) `jp_stage_dry_km` and `jp_resupply_reach` are **ported** and
   take explicit parameters (route polyline, `cell_km`, `gw`/`gh`,
   `flow_field`, `water_bodies`, `flow_thresh`, `map_width_km`, stop
   positions) — this layer supplies them, it does not re-derive them. (e)
   `_jpEnsurePlan`'s default block is already reproduced as `JpPlan::default`
   for the fields milestone 4 reads; what milestone 5 still owes it is the
   route-aware vessel correction (`jpAutoPickVessel` on first creation).

6. ~~**Verdict/reporting**~~ — see "Milestone 6" below. **Done 2026-08-18**,
   together with milestone 2’s remainder. Original list, for cross-reference:
   `_jp_verdict`, `_jp_confidence`, `_jp_pack_range`,
   ~~`jp_fmt_kg`~~ (shipped with milestone 4, as planned — both stage
   calculators format their overload/hold text with it), `jp_fmt_days`.
   Small, needs milestone 5's plan output to verify against. One correction
   milestone 4's reading adds: `jp_fmt_kg`'s port carries a `js_fixed`
   helper reproducing JS `toFixed`'s round-half-*away-from-zero* tie-break
   (Rust's `{:.N}` rounds half to even), which `jp_fmt_days` and every other
   verdict string in this milestone should reuse rather than re-derive.

   **Three corrections milestone 5's own reading adds**, all about what this
   milestone now has to read: (a) the plan object it verifies against is
   `JpJourneyPlan` (`cartalith-civ`), and every field `_jpVerdict` consults
   is on it — `blocked_idx`/`results`/`resupply_reach`/`riv_x`/`pass_km`/
   `desert_km`/`bad_wx_pct`/`stops` — except the reference's own `blockedMsg`,
   which is deliberately not duplicated (read `results[blocked_idx]`, whose
   `Err` is a `JpBlocked` with the real message). (b) The reference's
   `risk` advisory (a four-tier day-threshold caption on `_jpPlan`'s return)
   is **not** ported — it is a verdict string, so it belongs here, not to
   milestone 5; the day counts it reads are all on `JpJourneyPlan`. (c) The
   harness for this milestone is cheap now: milestone 5's slice list already
   loads everything `_jpVerdict`/`_jpConfidence` need, and a real
   `JpJourneyPlan` to verify against is one `jp_plan` call on the same
   synthetic world.

**UI-only, not portable** (`ARCHITECTURE.md`: Godot owns presentation) —
`_jp_run_auto`, `_jp_refresh`, `_jp_sync_asset_inputs`, `_jp_render_party_
form`, `_jp_render_stops`, `_jp_render_results`, `_civ_render_journey_list`.

## Out of scope for all milestones above

Wiring any of this into `compute_civilisation()`/Godot — this is real,
interactive, user-driven tool (the reference's own Journey Planner is a
form the player fills in per-journey, not something auto-computed for every
settlement pair), so its real UI is genuine future GUI work
(`GUI_SHELL_SCOPE.md`'s "Simulate → Logistics" section already names this),
not a data-flow wiring task the way settlements/roads/territory were.

## Done means (whole plan)

Every real `jp*`/`_jp*` function (UI-only ones excluded) ported and tested
in `cartalith-civ`, reachable from a real `#[func]` taking a journey request
(origin, destination, party composition) and returning a real plan —
distant future scope, tracked here milestone by milestone rather than
attempted in one pass.

## Closing status (2026-08-18) — what is done, what is not, and what "done" is still missing

**All six milestones are complete.** The engine half of the bar above is met:
every real `jp*`/`_jp*` function is ported and tested in `cartalith-civ`,
bar one. The `#[func]` half is not, and deliberately so — see below.

### The count, honestly

`ECONOMY_SCOPE.md` sized this subsystem at "~70 real functions". Counted
exactly rather than re-estimated, the frozen reference defines **74**
`jp*`/`_jp*` functions. Against that:

- **Ported: 65 of 74**, i.e. everything the four exclusions below do not
  account for — verified mechanically by mapping each reference name to its
  snake_case port and checking every one resolves, not by counting the
  milestone write-ups. Plus their data tables (`JP_ANIMALS`, `JP_SHIPS`,
  `JP_TERRAIN`, `JP_BIOMES` including its weather distributions, `JP_RIG`,
  `JP_INFRA_TIERS`, `JP_LAND_TRANSPORTS`, the seasonal/grazing/desert tables,
  and the `JP_COST_*`/vehicle/ration constants).
- **Not portable, and never were: six of the 74.** `_jpRunAuto`,
  `_jpRefresh`, `_jpSyncAssetInputs`, `_jpRenderPartyForm`, `_jpRenderStops`,
  `_jpRenderResults` — DOM rendering, which `ARCHITECTURE.md` assigns to
  Godot. (`_civRenderJourneyList`, this doc's seventh UI-only entry, is a
  `_civ*` name and so falls outside the 74.) Excluded by this document from
  the start, not dropped.
- **Not Rust functions, for reasons recorded rather than omitted: two.**
  `_jpLayovers` (a JS lazy-init idiom; a `HashMap` needs none — shipped as the
  `JpLayovers` alias) and `_jpSettlements` (a *runtime* kind filter over the
  reference's one untyped `state.places` array; this port's settlements are
  already typed, so building the `JpPlace` list **is** the filter).
- **Blocked at closeout, unblocked 2026-08-18: one.** `_jpRerouteForMode` —
  its dependency, the Route tool's multi-modal pathfinder, was ported by
  `UNIFIED_TOOL_PLAN.md` milestone D. See "Closing status" below.

65 + 6 + 2 + 1 = 74, with nothing unaccounted for.

Separately, **six helpers outside the `jp*` namespace and on no milestone list
here** came along because a milestone genuinely needed them:
`buildCartBiome`/`buildCartTerrain` with `CART_BIOMES`/`CART_TERRAINS` (the
two Cartalith paint layers this port had never built at all — the largest
single finding of the whole sub-phase),
`_civTransshipments`/`_civTransferOverhead`, `_civWalkWayCells` and
`_civPassedSettlements`. Those are real additions to this port, not Journey
Planner overhead.

### The one remaining gap

**`_jp_reroute_for_mode` was blocked; it is UNBLOCKED as of 2026-08-18
(`UNIFIED_TOOL_PLAN.md` milestone D).** What follows is the closeout's original
finding, kept because it was correct and because the resolution only makes
sense against it:

> **`_jp_reroute_for_mode` is unported and stays unported.** Milestone 6
re-checked the finding rather than inheriting it, and it holds: the function's
whole body is `_civDijkstraPath(s, e, domain)`, and `_civDijkstraPath` (25957)
with `_civWaterCostGrid` (21051) and `_civMixedCostGrid` (21090) are the
interactive Route tool's own multi-modal pathfinder — unported, on no milestone
in this document, and larger than a footnote. It is also a UI action (the
"Re-route land-only" button), so it sits on the far side of this doc's own
"Out of scope for all milestones" line twice over. Its pure half,
`jp_mode_for_route` (transport → cost domain, carrying the reference's real
disclosed scope cut that "River Transport prefers rivers, does not require
them"), shipped with milestone 5.

Closing it means porting the Route tool's pathfinder, which is its own scope
document, not a Journey Planner milestone. **No pathfinder was invented here to
make the list look finished.**

**Resolution (2026-08-18).** The pathfinder was ported, as predicted, by
something that is not a Journey Planner milestone: `UNIFIED_TOOL_PLAN.md`
milestone D, the Civilization tool group, whose Draw route/way tool needs the
same function. `cartalith_civ::tools::civ_dijkstra_path` ships all three of the
reference's domains (`RouteMode::Land`/`Water`/`Mixed`, i.e.
`_civLandCostGrid`/`_civWaterCostGrid`/`_civMixedCostGrid`) **and** the `v1.47`
`reachable` flag — which is exactly the piece `_jpRerouteForMode` needs, since
it *"never silently accepts `_civDijkstraPath`'s straight-line fallback as if
it were a real path"*. Golden-verified bit-exact against the reference over 16
cases, including negative controls for both unreachable directions.

That milestone also recorded a correction this document's readers should know:
`_civDijkstraPath` is **not** `road_dijkstra`. `road_dijkstra` is the bare
relaxation kernel; the cost grids, the existing-way discount, settlement
gravity, wrap-aware smoothing and the `reachable` flag are all in the wrapper.
Anyone who read this doc's "unported pathfinder" note as "one function" was
reading it correctly by name and by an order of magnitude too small by volume.

**What is left for `_jp_reroute_for_mode` is no longer a pathfinder.** It is:
`jp_mode_for_route` (already shipped with milestone 5), a `reachable` check, a
call, and the assignment of `jn.pts`/`jn.km`/`jn.brks` — plus the `forceMode`
override and the two failure messages. All of that is small and none of it is
blocked. It remains a **UI action**, so it still sits on the far side of this
doc's "Out of scope for all milestones" line — but only once over now, not
twice.

Two quality ceilings, both disclosed where they were found, neither a blocker:

- **Wildlife richness** (`jp_foraging` via `_jpWildlifeForageMod`) is
  caller-supplied, because the ecoregion-segmentation and species-roster
  subsystem behind it (`buildEcoregions`/`regionRichness`/`assignWildlife`/
  `WILD_ROSTERS`) is unported and on no milestone anywhere. `1.0` is the
  reference's **own** answer on a world with no wildlife layer, and also what
  an exactly-average region gives — so this costs no fidelity today, and the
  flat `JP_BIOMES.forage` table stays the anchor it was designed to be.
- **`_civSeaTimeEdgeCost`** (v1.98 current/wind-costed sea lanes) was already
  flagged unported by Phase 2 milestone 13, for the same reason; the Journey
  Planner reads sea *conditions* from the real fields and is unaffected.

### What integration would actually mean

None of this is wired to anything. **That is the standing boundary this
document set before milestone 1, not a shortcut taken at the end**, and the
"Out of scope for all milestones" section above states why: the reference's
own Journey Planner is a form the player fills in per journey — origin,
destination, party composition, season, transport, supply days, grazing,
per-stage overrides — not something a generator can auto-compute for every
settlement pair. Wiring `jp_plan` into `compute_civilisation()` would mean
inventing a journey nobody asked for.

Making it a real user-facing feature therefore needs, in order:

1. **A route to plan.** `jp_plan` takes a polyline. The reference gets one
   from the interactive Route tool. Its *pathfinder* is now ported
   (`civ_dijkstra_path`, `UNIFIED_TOOL_PLAN.md` milestone D — the same
   function `_jpRerouteForMode` was blocked on), and so is way commitment
   (`civ_commit_way`); what is still missing is the **waypoint-capture
   interaction** and `_civCommitRoute`'s own `civJourneys` push, both
   milestone F. Until a user can draw or
   solve a route, there is no input.
2. **A `JpWorld` assembled from live state.** Every field it borrows is real
   and already computed by this port — `field`, `temp`, `rain`, flow, water
   bodies, territory, settlements, the consolidated way network, the ocean and
   wind coarse fields — but nothing currently gathers them into one struct at
   the `cartalith-godot` boundary, and two of them (`cart_biome`/
   `cart_terrain`) are built by functions milestone 5 added that no pipeline
   stage calls yet.
3. **A party form.** `JpPlan` is ~20 fields plus a sparse per-stage override
   map. That is a real GUI surface, not a `#[func]` signature —
   `GUI_SHELL_SCOPE.md`'s "Simulate → Logistics" section already names it.
4. **`#[func]`s over the boundary.** Small once 1-3 exist: a plan request in,
   a plan plus its verdict/confidence/pack-range out. Note that `JpJourneyPlan`
   is a deep structure (stages, per-leg `Result`s, timeline, stops) and gdext
   wants flat `Variant`-compatible types, so this is a real serialization pass,
   not a one-line export.
5. **The presentation the port deliberately left out.** Every HTML hint string,
   every `formula` trace, and the elevation profile chart are Godot's to draw;
   the values they print are all fields on the returned structs.

Steps 1 and 3 are the substantial ones, and both are GUI work. **The engine is
done; the feature is not, and the gap between them is a user interface.**
