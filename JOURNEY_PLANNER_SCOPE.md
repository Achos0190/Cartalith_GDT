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

## Milestone 2 — transport mode selection: done (2026-08-17)

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

## Milestone 3 — physical travel cost: done (2026-08-17)

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

## Real milestone breakdown for what remains (not started)

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

| Build next | Milestone | Why this position |
|---|---|---|
| 1st | **4 — Consumption/resupply** | Unblocks milestone 3's tail (`jp_calc_land`/`jp_calc_water`) and part of milestone 2's. Carries `jp_fmt_kg` from milestone 6. Contains real quick wins (four near-one-liners) plus one genuinely hard piece (`jp_foraging`'s wildlife-richness plumbing). |
| 2nd | **3 (tail) + 2 (partial)** | `jp_calc_land`/`jp_calc_water` become portable the moment 4 lands; `_jp_best_land_transport_for_stage` follows immediately after `jp_calc_land`. Not a milestone of its own — fold into 4's own verification pass. |
| 3rd | **5 — Route/stage derivation** | The orchestration layer; needs 2-4 done. Almost certainly the largest single milestone here. Pick up `_civTransshipments` alongside it (on no list, needed by the already-ported `jp_journey_cost`). |
| 4th | **6 — Verdict/reporting** | Needs 5's plan output to verify against. Minus `jp_fmt_kg`, already taken by 4. |
| last | **2 (remainder)** | `jp_auto_pick_transport`/`jp_auto_pick_vessel`/`_jp_best_package_for_stage` all need milestone 5's plan shapes. Re-attempt after 5. |

Milestones 1 and 3 (main body) are **done**; see their own sections above.

The numbered list that follows preserves the original numbering for
cross-reference; consult the table above for what to actually build next:

2. ~~**Transport mode selection**~~ — see "Milestone 2" above. Four
   functions remain genuinely blocked, re-checked against the reference
   again after milestone 3 and **still all four blocked**:
   `jp_auto_pick_transport`/`jp_auto_pick_vessel` need milestone 5's
   `_jpEnsurePlan`/`_jpDeriveStages` (and `jp_auto_pick_transport` needs
   milestone 4's mass model besides); `_jp_best_land_transport_for_stage`
   needs `jpCalcLand`, which milestone 3 could not port either and which now
   sits behind **milestone 4**, not milestone 3; `_jp_best_package_for_stage`
   needs milestone 5's `_jpEffectiveStagePlan` output shape. Re-attempt each
   once its real dependency milestone lands.

3. ~~**Physical travel cost**~~ — see "Milestone 3" above. Seven of eleven
   shipped; two (`jp_water_window`, `jp_animal_terrain_mod`) had already
   shipped with milestone 2. **`jp_calc_land`/`jp_calc_water` remain
   blocked on milestone 4**, which this list orders after them — a real
   ordering error in this document, corrected in the milestone 3 section
   above. `JP_BIOMES[...].weather` was indeed unported and is now ported.
   `jp_journey_cost` turned out genuinely portable, no milestone-5 plan
   object needed.

4. **Consumption/resupply — do this next; milestone 3's tail depends on it.**
   `jp_human_water_carry_days`, `jp_human_water_rate`,
   `jp_animal_water_carry_days`, `jp_consumption_factors`, `jp_capacity`,
   `jp_foraging`, `jp_assess_resupply`, `_jp_world_mean_richness`,
   `_jp_wildlife_forage_mod`, `_jp_resupply_reach`,
   `_jp_drinking_coarse_ease`, `_jp_stage_dry_km`, `_jp_desert_tier_for_gap`
   — **plus `jp_calc_land` and `jp_calc_water`**, milestone 3's two
   deferrals, which become portable the moment `jp_capacity`/`jp_foraging`/
   `jp_assess_resupply`/`_jp_desert_tier_for_gap` exist. `jp_human_water_rate`
   /`jp_human_water_carry_days`/`jp_animal_water_carry_days`/
   `_jp_desert_tier_for_gap` are one-liners over data mostly already ported
   (real quick wins). `jp_capacity` also needs the `JP_BIOMES` columns
   milestones 2 and 3 each deliberately left out (`water`/`forage`/
   `waterForage`/`grazing`) and the seasonal tables `JP_SEASONAL_ANIMAL`/
   `JP_SEASONAL_HUMAN`/`JP_DESERT_ANIMAL_MOD`/`JP_GRAZING`, none of which are
   ported yet. `jp_foraging` is the one genuinely hard piece: through
   `_jp_wildlife_forage_mod` it reads the world's wildlife-richness region
   field, which this port has never plumbed into the Journey Planner — expect
   that to be its own real decision, not a transcription.

5. **Route/stage derivation** — `_jp_derive_stages`, `_jp_effective_stage_
   plan`, `_jp_plan`, `_jp_ensure_plan`, `_jp_layovers`, `_jp_stop_key`,
   `_jp_road_cells`, `_jp_settlements`, `_jp_infra_context`,
   `_jp_claimed_at`, `_jp_stage_infra`, `_jp_river_condition`,
   `_jp_sea_condition`, `_jp_coarse_idx`, `_jp_water_reach_cells`,
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

6. **Verdict/reporting** — `_jp_verdict`, `_jp_confidence`, `_jp_pack_range`,
   `jp_fmt_kg`, `jp_fmt_days`. Small, needs milestone 5's plan output to
   verify against — **except `jp_fmt_kg`, which is needed at milestone 4**,
   not here: `jpCalcLand`/`jpCalcWater` both format their overload/hold
   blocked-message text with it. Port it with milestone 4 and leave the rest
   of this milestone where it is.

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
