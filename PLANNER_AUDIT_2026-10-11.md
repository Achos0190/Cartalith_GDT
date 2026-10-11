# Journey planner route audit, 2026-10-11

This is a read-only audit. No engine behaviour, speed, cost table or cost model was changed.

**Experiment.** I added one new file, `cartalith-native/crates/cartalith-godot/tests/planner_audit_journey.rs`. It holds 6 `#[ignore]`d tests. Each has a Ruling BK header and a `Protects:` line, and none of them pins a value.

**Command.** `CARGO_TARGET_DIR=<scratch>/planner_target cargo test -j 2 -p cartalith-godot --test planner_audit_journey -- --ignored --nocapture --test-threads 1`.

**Result.** `6 passed`, in 0.8 s, run twice in separate processes with identical output. The full output is in `run_output.txt` next to this file.

**Fixture.** A real world from `generate_terrain(WorldParams::defaults(256,192,24601))`. These are the **engine** defaults, so the divergence switches are `false`. The `JpWorld` is assembled the way `story_bridge::WorldGen::jp_world_parts`/`jp_world` assemble it, from the same public functions. The fixture has no settlements and no territory, so the infrastructure tier is identical on every compared run.

**Blocked land runs.** Every land plan returned `total_days = None`, meaning blocked. The cause is probably supply or resupply over about 450 km with no settlements, but I did not measure the cause. For that reason the land comparisons use `travel_days`.

## How the planner gets a route (background for all four questions)

The planner **never paths anything itself**. Two `#[func]`s pass it the committed Route's polyline (`InfraTools.routes[i].pts`):

- `jp_compute`, at `request.route` or `points`.
- `jp_plan_for_route`.

The planner then calls `jp_plan_full` → `jp_derive_stages(world, pts, plan)`. That function classifies each **route point** and chunks contiguous runs into stages.

The path itself comes from the Route tool:

- `route_commit` → `civ_join_dijkstra_segs` → `civ_dijkstra_path_guided` in `cartalith-civ/src/tools.rs`.
- `jp_reroute` → `jp_reroute_for_mode` re-paths **only the first and last point**. It drops any intermediate stops, which is faithful to the reference's `_jpRerouteForMode`.

So "does the journey follow ways" splits in two:

- **(a)** Does the Route's least-cost path ride the ways?
- **(b)** Does the planner credit the way along the points it was given?

---

## Q1. Routes followed — verdict: **PARTLY**

**Symbols read:**
- `tools::civ_dijkstra_path_guided`
- `EXISTING_WAY_DISCOUNT` (0.25)
- `civ_mark_ways_on_grid`, `civ_mark_way_neighborhood`, `civ_walk_way_cells`
- `civ_sea_lane_cells`
- `WayRef::from(&ManualWay)`
- `WorldGen::route_commit` and `way_stroke_commit` (`lib.rs`)
- `jp_road_cells`, `jp_derive_stages`
- `civ_smooth_path` (its doc: RDP plus Catmull-Rom)

**Mechanism:**
- The router multiplies every non-hidden way cell, plus its 3×3 neighbourhood, by ×0.25. This applies to generated ways and `ManualWay`s alike, so ET-D4 drag ways are included. Sea lanes get ×0.25, or 1.0 where the cell is otherwise impassable.
- There is no snapping, no directed edges and no "stay on the way" constraint. A route rides a way only where the 4× discount beats the shortcut.
- `route_commit` passes `civ.ways` plus the manual ways, but **not** `civ.road_edges`.
- The planner then checks each route point's cell against `jp_road_cells`, a 3×3-dilated table, and sets the terrain to "Dirt Track" or "Paved Road" when the point is on a way.

**Experiment:** `q1_land_route_follows_a_hand_drawn_way` and `q1_sea_route_follows_a_sea_lane`. I built a way through a deliberate detour waypoint with `civ_commit_way(A, M, B)` and routed A→B with and without it.

| Case | Share of route on way | Share of way covered | Backsteps | Control (no way): on / covers |
|---|---|---|---|---|
| Land Road / Track / Ancient (way 728.9 km, route 454.1 km) | 0.883 (15 cells off) | 0.581 | 0 | 0.689 / 0.382 |
| Land, reverse B→A | 0.883 | 0.581 | 0 against the reversed way | — |
| Sea lane, Water mode (lane 593.8 km, route 567.8 km) | 0.994 (1 cell off) | 0.959 | 0 | 0.127 / 0.110 |
| Sea lane, Mixed mode | 0.928 | 0.895 | — | — |

**Planner credit:**
- 48 of 55 route points read as road.
- Walking `travel_days`: 20.94 on the way with road credited; 41.05 on the same route without credit; 33.53 on the no-way route.
- Ancient: 26.43 days, because it is stamped "Deteriorated".
- The sea lane produces **0** planner road entries. `jp_road_cells` skips sea lanes, which is faithful to the reference, so a lane affects the path but not the speed.

**Findings:**
1. **Ways are followed approximately, not exactly.** The land route took the way for 88% of its length but cut off 42% of the way's own detour. That is the reference's design: the discount competes with terrain. Direction is correct (no backsteps either way), and Road, Track and Ancient give **identical** routes.
2. **`road_edges` drift.** The planner credits `civ.road_edges` but the router does not discount them. A road the planner credits can therefore be invisible to the router.
3. **Hidden ways are still credited.** `jp_road_cells` skips a *hidden* generated way but still stamps its raw edge. Test `hidden_generated_way_is_still_credited_through_its_edge`: a hidden highway with its edge stamps 90 cells as Dirt Track/Standard; without the edge it stamps 0. So the planner credits a road the user hid.
4. **The planner reads only the route's sample points.** It checks the RDP/Catmull-Rom-smoothed points, not every cell between them. This is mitigated by the 3×3 dilation; 7 of 55 points still read off-road on a way-following route.

**Smallest fixes (owner sign-off needed):**
- For finding 3: skip edges whose generated `Way` is hidden, or stamp only from `ways` and drop the `road_edges` source. The second would change live planner output, not goldens. `jp_road_cells` goldens (`jp_road_cells_reads_hand_drawn_ways_including_ancient`, and milestone 5's) would need checking: a filter keyed on hidden ways changes nothing for visible ones.
- For finding 2: pass `road_edges` cells into `RouteContext` as well.
- Making a route stick to a way exactly would be a new snapping mode. That is a design question, not a fix.

---

## Q2. Terrain and road type — verdict: **PARTLY (separate models, and slope is ignored in planner time)**

**Inputs the planner reads, per point or stage (`jp_derive_stages`, `JpWorld`):**
- `field` and `peak_m`: gain and loss. These are **used only by river condition and the profile/verdict**, never in land speed.
- `cart_biome`, from `build_cart_biome(water_bodies…)`: category (15 = sea, 14 = lake → "river"), plus the biome via `jp_legacy_biome_of(temp, rain)`.
- `cart_terrain` (`build_cart_terrain`): the land terrain row, which a road overrides.
- `road_cells` (`jp_road_cells`):
  - Highway → Paved Road / Maintained.
  - Regional → Paved / Standard.
  - Road and Track (generated and manual) → Dirt Track / Standard.
  - Ancient → Dirt Track / Deteriorated.
  - Road edges → Dirt Track / Standard.
  - Sea lanes skipped.
- `flow_field > flow_thresh`: the `rx` river-crossing count.
- `water_bodies`: the dry-km calculation.
- `territory`: claimed fraction.
- `places`: the infrastructure tier.
- `ocean_field` and `wind_field`: sea condition.

**Speed multipliers and their sources:**

| Multiplier | Source |
|---|---|
| `jp_terrain_land_mod` | `JP_TERRAIN.land`, ref 17446 |
| `jp_route_mod` | `JP_ROUTE`, ref 17464 |
| `jp_infra_mod` | `JP_INFRA`, 17534 |
| `jp_pace_mod` | 17533 |
| Group class | 17553 |
| Fatigue, load | Reference |
| `jp_terrain_water_mod` | Reference |
| `jp_weather_factor` | Reference |

All are reference tables, faithfully ported.

**Router cost, by contrast:**
- Slope `1+50·sl²`, from `build_travel_cost` or inline in the mixed grid.
- Pass relief: §7i, a port addition.
- Swamp penalty and ford-vs-bridge `civ_river_crossing_cost`: §7i, a port addition.
- Biome friction: Mixed only.
- Navigable-river discount: Mixed only.
- Way ×0.25, type-blind.

**Findings:**
1. **Two independent models.** The router thinks a way is **4× cheaper** of any type. The planner thinks a dirt road is **about 1.24× faster** than wild ground (1.0·1.0 vs Open Plains 0.95 × "None / Wild" 0.85) and a highway about 2.1× faster. The two can disagree about which route is best. This is reference-faithful and not a duplicated table, but it is a drift risk: the §7i terrain terms exist only in the router.
2. **The road type barely matters to the planner.** Generated Road and Track, and manual Road and Track, all map to Dirt Track / Standard (measured: identical stages and days). Only Ancient differs (Deteriorated, 26.43 vs 20.94 days), plus generated Highway and Regional. Faithful to `_jpRoadCells`. There is no user-editable way condition (`w.condition`) in the port; that gap is documented at `jp_road_cells`.
3. **Slope, fords and bridges do not affect planned time.**
   - `JpDerivedStage.gain/loss/rx` are dropped by `to_stage`.
   - `JpStage` has no such fields.
   - `rx` reaches only the verdict text ("N river crossings") and the hazards line. The reference does the same (`rivX`, ref 19833/19961).
   - A road cell **replaces** the terrain: a road over Mountain Trails is "Dirt Track" (1.0, vs 0.45).
   - The ford-vs-bridge cost exists only in the router.
4. **Water terrain rows that are never derived.** `jp_terrain_water_mod` and `JP_ROUTE` rows "Moderate River", "River with Shallows", "River Delta", "River with Rapids" and "Rough Open Sea" are never produced by `jp_derive_stages`. Only Calm River, Sheltered Bay, Coastal Waters and Open Sea are. Those rows are reachable only through a manual or stage override. ("Rough Open Sea" is also tested in `jp_stage_infra`.)
5. **Wind and current come from live sliders, not the world.** The coarse fields are built from **`self.params`** (live sliders), not from the parameters the world was generated with. A wind-direction slider change without regenerating moves journey times. The saved-journey cache key hashes `self.params`, so it stays consistent. This is a code reading, not measured in a running app.

**Smallest fix:** none required for parity. If the owner wants consistency:
- Feed `gain/loss` (or the stage's mean slope) and `rx` into `jp_calc_land` as Tobler-style and ford-delay terms. This is a §7p-style departure, but it would move the milestone 3–5 `jp_*` goldens in `cartalith-civ` (`golden_parity_*` and the `jp_plan` fixtures).
- Alternatively, keep the router's way discount type-aware, e.g. `0.25 / jp_terrain_land_mod`.

Both need a ruling.

---

## Q3. River currents — verdict: **NOT (for real rivers); the "river" current is an elevation proxy on lakes**

**Symbols read:**
- `jp_derive_stages`, point classification: only `cart_biome == 14` (a **lake**) within 2 cells of land becomes `cat:"river"`/"Calm River". `build_cart_biome` maps `water_bodies 1→15, 2→14`; river channels are land cells.
- `jp_river_condition(km, gain, loss)`: the gradient of route-point elevation, with thresholds 8 and 35 m/km.
- `jp_route_mod("river", …)`: 1.40 / 1.30 / 1.00 / 0.80 / 0.55.
- `jp_land_transport_kmh` (`None` → portage).

The router's `channel_recv` and flow direction are never consulted. Neither is discharge or order, apart from `flow > thresh` for counting `rx`.

**Experiments:**

`q3_real_river_channel_is_planned_as_land`:
- A 57-cell river traced downstream by maximum discharge, from flow 23.9 to 1477.6 (threshold 19.7).
- Planned as River Transport + Keelboat.
- **All 14 stages are `land`**, and every leg is `Walking portage=true`.
- Downstream 14.71 vs upstream 14.82 travel days. The 0.7% difference is only stage chunking.

`q3_river_condition_reads_elevation_not_flow`, on a lake strip 3 cells wide:

| Case | Direction | Condition | Travel days |
|---|---|---|---|
| Sloping bed | West→east (downhill) | Strong Downstream | 1.212 |
| Sloping bed | East→west (uphill) | Strong Upstream | 3.084 |
| Flat bed | Both | Neutral | 1.696 |

The days were **identical whether the strip's discharge was 0 or 1e6**.

**Findings:**
1. River Transport on a real river is costed as **walking**.
2. "River" stages exist only on lake margins. Their "current" comes from bed-elevation differences, which is DEM noise on a lake. The reference's own v1.102 comment names this.
3. Real discharge and flow direction are never read.

All three are reference-faithful (`_jpDeriveStages`, `_jpRiverCondition`, ref 18911). No ruling changes them.

**Smallest fix (needs a ruling):**
- Classify a land point as `river`/navigable when `flow > thresh` (or Strahler order ≥ k, which `RouteInputs` already builds) and the transport is River Transport.
- Take the condition from the sign of travel along `channel_recv` (downstream if the next point is the current cell's receiver), scaled by discharge or order rather than bed slope.

This would move `jp_derive_stages` goldens: milestone 5's fixture has "a river column" and derives "1 river" stage (JOURNEY_PLANNER_SCOPE.md:274-276), so a ruling is needed.

---

## Q4. Ocean currents and wind — verdict: **FOLLOWED for sailing hulls (banded); PARTLY overall**

**Symbols read:**
- `coarse_ocean_wind_fields` (`lib.rs`) → `cartalith_climate::current_ocean_field` (wind → Ekman 25°, coast deflection, shelf damping, western intensification) and `current_wind_field`. Both are fed `p.climate.wind_manual`, `wind_dir_deg` and `press_k`. `current_k` scales only the SST anomaly and is not an input to the vectors.
- `jp_sea_condition`: per-segment current·heading/maxSpeed and the sail polar at the true wind angle, scored `0.65·wind + 0.35·current`, giving one of 5 bands.
- `jp_route_mod("sea")`: 1.40 / 1.20 / 1.0 / 0.75 / 0.50, applied as `r_mod` in `jp_calc_water_ex`.

**Experiment:** `q4_sea_leg_wind_and_current_both_directions`. A Water-mode route (476.1 km) planned forward and reversed.

Cog (square rig), total days:

| Wind | Forward | Reverse |
|---|---|---|
| Auto | 5.51 (Favorable Wind) | 15.21 (Strong Headwind) |
| Manual 0° (u +2.7) | 5.51 | 15.21 |
| Manual 90° | 8.17 | 5.36 (one stage "Favorable Wind & Current") |
| Manual 180° | 14.81 | 5.50 |
| Manual 270° | 5.51 | 5.90 |

- Control with no fields: every stage Neutral.
- Keelboat, an oared hull, scores on current only: 13.42–14.52 days, with the band moving on one or two stages.

So direction and wind both act, through a 5-step band.

**Findings:**
1. **The condition is computed with the shared plan vessel, not the vessel the stage actually uses.** With Keelboat chosen, the Open Sea stage is auto-substituted to **Dhow**, a fore-and-aft sailing rig, at 135 km/day. Its condition was still derived as oared, so wind was ignored ("Neutral" in all 5 wind cases). A per-stage `vessel` override has the same gap. Reference-faithful (`_jpDeriveStages(jn, plan)` uses `plan.vessel`).
2. **The effect is quantised.** Auto and manual 0° give identical days because both fall in the same band; 5 bands drive ×0.5…×1.4. Sail-polar magnitude is otherwise discarded.
3. **The chunking is direction-asymmetric.** Forward gives 5 stages (Open Sea 345.5 km); reverse gives 4 (339.3 km). The sliver absorb runs in reverse index order. Total km are equal, but stage boundaries, and hence conditions and days, differ slightly by direction even with no fields.
4. **The world's `currents` climate toggle is not consulted.** The planner always applies currents. This is a code reading.

**Smallest fix for finding 1:** in `jp_plan_full`, recompute `jp_sea_condition` for a water stage whenever the effective stage vessel (override or auto-substitute) differs from `plan.vessel`. This is local, but it changes output for affected plans. Check `jp_plan` goldens; the milestone 5/6 fixture uses Keelboat over a sea stage, if any.

---

## Other things that make a planned journey wrong

1. **Settlement count is not wrap-aware.** In `jp_derive_stages`, the count within `pick_r` uses an unwrapped `(p.x-px)`, unlike `civ_passed_settlements`, which is wrap-aware. On wrapped worlds, places across the seam are missed, which lowers the infrastructure tier. This is a code reading; check the reference before calling it a port bug.
2. **`jp_reroute` keeps only the endpoints.** It re-solves the two endpoints only, so intermediate Route stops are silently dropped. Faithful to the reference.
3. **Two map-width fallbacks.** `jp_infra_context` falls back to 800 km and `cell_km` to 12000 km when the map width is 0. Both are documented and reference-faithful.
4. **Possible ×0.25 overlap.** The ET-D4 guided route corridor applies after the way discount, so a drag that passes beside an existing way can rejoin it. I did not test this.
5. **No impassable-cell legs observed.** The land route's repair pass kept it on land. Mixed mode has no validity repair by design (v1.99).

## doc_notes (for the main loop; I edited no documents)

- **`JOURNEY_PLANNER_SCOPE.md`**, "Route-planner conformance": add that `route_commit`'s way list excludes `civ.road_edges` while `jp_road_cells` includes them, and that hidden generated ways stay credited through their edges.
- **`STATUS.md` / `OUTSTANDING_WORK.md`** candidate rows, each needing an owner ruling:
  - (a) River Transport on real channels is costed as walking portage; river current comes from bed slope on lakes only.
  - (b) The sea condition ignores the effective stage vessel (auto-substituted Dhow scored as oared).
  - (c) Slope, rx and ford/bridge do not reach planned time.
  - (d) Hidden-way credit through `road_edges`.
- **`MISTAKES.md`** candidate: none confirmed. The `total_days = None` on settlement-less fixtures is a measurement trap worth one line in the harness only.
