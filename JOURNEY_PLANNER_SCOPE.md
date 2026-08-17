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

## Real milestone breakdown for what remains (not started)

Ordered by dependency, per `ECONOMY_SCOPE.md`'s own categorization:

2. **Transport mode selection** — `jp_auto_pick_transport`,
   `jp_best_animal_for_context`, `jp_pick_species_for_route`,
   `jp_resolve_mount`, `jp_auto_pick_vessel`, `jp_vessel_matrix`,
   `_jp_vessel_fits`, `_jp_best_land_transport_for_stage`,
   `_jp_best_package_for_stage`, `_jp_auto_stage_vessel`. Needs biome/terrain
   context objects this port's civ layer already produces in some form
   (biome raster, water-body classification) — read the reference's real
   argument shapes before assuming a 1:1 translation; the reference's own
   `biome` objects (`.desertLike`, `.bestAnimals`) may not map directly onto
   this port's `u8` biome-id raster and would need a lookup table.

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
