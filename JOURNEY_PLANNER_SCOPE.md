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

## Real milestone breakdown for what remains (not started)

Ordered by dependency, per `ECONOMY_SCOPE.md`'s own categorization:

2. ~~**Transport mode selection**~~ — see "Milestone 2" above. Four
   functions remain genuinely blocked: `jp_auto_pick_transport`/
   `jp_auto_pick_vessel` need milestone 5's `_jpEnsurePlan`/`_jpDeriveStages`;
   `_jp_best_land_transport_for_stage` needs milestone 3's `jpCalcLand`;
   `_jp_best_package_for_stage` needs milestone 5's `_jpEffectiveStagePlan`
   output shape. Re-attempt each once its real dependency milestone lands.

3. **Physical travel cost** — `jp_train_pace`, `jp_sail_factor`,
   `jp_water_window`, `jp_wx_weighted`, `jp_weather_factor`,
   `jp_animal_terrain_mod`, `jp_column_length_km`, `jp_column_factor`,
   `jp_calc_land`, `jp_calc_water`, `jp_journey_cost` (line 18873, the
   apparent top-level cost function — read its real signature before
   assuming what it needs; it may depend on milestone 2's transport
   selection already having run). Depends on milestone 1's primitives
   (`jp_fatigue`/`jp_load_penalty`/`jp_surface_gain` all feed into pace
   calculations) and on weather-distribution data (`JP_BIOMES[...].weather`
   in the reference — a real data table not yet identified as ported or
   not, check before assuming).

4. **Consumption/resupply** — `jp_human_water_carry_days`,
   `jp_human_water_rate`, `jp_animal_water_carry_days`,
   `jp_consumption_factors`, `jp_capacity`, `jp_foraging`,
   `jp_assess_resupply`, `_jp_world_mean_richness`,
   `_jp_wildlife_forage_mod`, `_jp_resupply_reach`,
   `_jp_drinking_coarse_ease`, `_jp_stage_dry_km`, `_jp_desert_tier_for_gap`.
   `jp_human_water_rate`/`jp_animal_water_carry_days` are small and
   independent (already read this pass, not yet ported — real quick wins
   for whoever picks this milestone up). The rest need real settlement/
   terrain context.

5. **Route/stage derivation** — `_jp_derive_stages`, `_jp_effective_stage_
   plan`, `_jp_plan`, `_jp_ensure_plan`, `_jp_layovers`, `_jp_stop_key`,
   `_jp_road_cells`, `_jp_settlements`, `_jp_infra_context`,
   `_jp_claimed_at`, `_jp_stage_infra`, `_jp_river_condition`,
   `_jp_sea_condition`, `_jp_coarse_idx`, `_jp_water_reach_cells`,
   `_jp_mode_for_route`, `_jp_reroute_for_mode`. The real orchestration
   layer — needs milestones 2-4 done first, needs this port's road network
   (`civ_hierarchical_network_topology`/`civ_consolidate_and_smooth_ways`,
   already real) and settlement data as real inputs. Almost certainly the
   largest single milestone in this whole plan.

6. **Verdict/reporting** — `_jp_verdict`, `_jp_confidence`, `_jp_pack_range`,
   `jp_fmt_kg`, `jp_fmt_days`. Small, needs milestone 5's plan output to
   verify against.

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
