# Generation-pipeline architecture research (agent-produced, 2026-08-24)

> **This document is agent-produced, not owner-supplied.** Unlike
> `TERRAIN_ARCHITECTURE_RESEARCH.md`, `HETEROGENEOUS_COMPUTE_RESEARCH.md` and
> `HARDWARE_ACCELERATION.md` — which arrived from the owner and were preserved
> verbatim under an applicability annotation — this one was written by reading
> the repository's own code and by searching the literature. It therefore
> carries a stricter evidence discipline: **every claim is tagged**, and the
> tags mean what they say.
>
> **Status, updated 2026-08-24: it has since been acted on.** As written,
> nothing here had been implemented. Since then all four of §4's open
> questions have been answered by the owner and built: §3.2.1 (the dead
> pre-carve `compute_flow`, `DECISIONS.md` §7f, commit `ac1de2d`), §3.2.2
> (the radix sort, commit `ae260cd`) and §3.2.4 with its erosion-cycle design
> question (`recompute_stale`, §4 items 3 and 4 below). The **analysis** in
> Parts 1–3 is left exactly as written — it is the record of what was true
> when the decisions were taken, not a live description of the code — so read
> §4 first for what has changed underneath it.

## Evidence tags used throughout

| Tag | Means |
|---|---|
| **[code]** | Verified by reading this repository's source. Cited by symbol, not only by line — see the warning below. |
| **[repo-doc]** | Taken from a document already in this repository (its own audits, scope docs, changelog). |
| **[source]** | External, published, with a URL. Retrieved and read unless noted. |
| **[source·indirect]** | External claim obtained from a search summary or secondary page, where the primary could not be retrieved. Lower confidence, flagged as such. |
| **[inference]** | My reasoning from **[code]**/**[source]** facts. Should follow, but is not itself a citation. |
| **[judgement]** | My opinion. Reasonable people could disagree. |

> **Line numbers are unstable and are given as of `f3ae851` plus the working
> tree on 2026-08-24.** `cartalith-engine/src/lib.rs` shifted by ~28 lines
> *during* the writing of this document (another session is editing it), so
> every code claim below is anchored to a **symbol name** first and a line
> number second. If a line number does not match, search for the symbol.

---

## 0. The question

The owner's words, verbatim:

> "Currently we have different generation cycles all flowing from one system to
> the next. Just how [do] real planetary systems inform landscapes? Can't we
> have everything run from the first generation and keep feedback loops to a
> minimum? And more importantly how, without blowing up resources used or
> making generation take forever."

Three separable questions, answered in Parts 1, 2 and 3:

1. What *is* the pipeline, really, and where are its actual feedback edges?
2. Do nature and serious scientific models run as one simultaneous coupled
   system, or do even they decompose into staged solvers?
3. Given (1) and (2), is "everything from the first generation, minimal
   feedback" achievable here — and if the answer is partly no, what *is*
   available?

**The short version, stated up front so the rest can be read as evidence:**
the default pipeline already is a one-way cascade with exactly **one**
deliberate feedback edge, and the iterative machinery is default-off. The
owner is asking for an architecture the port largely already has. The cost is
therefore **not** in feedback loops — it is concentrated in one repeatedly-run
kernel, and one of its four runs per generation appears to be dead work.

---

# Part 1 — The pipeline as it actually is

## 1.1 There are two orchestrators, not one

This matters and is easy to miss.

- **`cartalith_engine::generate_terrain(&WorldParams) -> WorldState`**
  (`cartalith-native/crates/cartalith-engine/src/lib.rs`) is the *terrain*
  pipeline: tectonics → height → volcanism/impacts → flow → climate → flow →
  carve → refresh, plus an opt-in erosion-pass block. **[code]**
- **`compute_civilisation(...) -> CivData`**
  (`cartalith-native/crates/cartalith-godot/src/lib.rs`, a plain private fn,
  not a `#[func]`) is the *civilisation* pipeline: water bodies → biome →
  lithology → soil → carrying capacity → resources → suitability →
  settlements → roads → territory → economy. **[code]**

They are joined only at `WorldGen::absorb()`, which calls
`compute_civilisation` unconditionally right after every `generate_terrain`
(`self.civ = Some(compute_civilisation(&ws, …))`). **[code]** So from the
user's point of view "Generate" is one button and one cost; architecturally it
is two pipelines in two crates with a hard, one-way, whole-struct handoff
(`&WorldState`).

`cartalith-engine`'s stated contract is *"orchestrates; it does not compute"*
(`ARCHITECTURE.md`) **[repo-doc]** — and that holds: every stage below is a
call into a subsystem crate. `compute_civilisation` living in
`cartalith-godot` rather than `cartalith-engine` is an architectural
irregularity worth noting **[judgement]**, though it has a real justification:
it constructs Godot-facing types (`CivData`, `SettlementExplanation`,
`FactionRoster`) and deliberately frees large intermediates before returning
(`MEMORY_OPTIMIZATION_SCOPE.md`'s pass) **[code]**.

## 1.2 The terrain pipeline, in execution order, at default parameters

Defaults are `WorldParams::defaults(...)`: `carve_rivers: true`,
`world_structure.enabled: false`, `passes: ErosionPassParams::off()`,
`use_gpu: false`, `climate.currents: true`,
`climate.terrain_wind_deflection: true`, `volc.provinces: true`. **[code]**

| # | Stage | Function | Reads | Writes |
|---|---|---|---|---|
| 1 | Warp | `compute_warp` | seed | `warp_x/y` |
| 2 | Plates + Lloyd | `build_plates` | seed | `plates` |
| 3 | Plate assignment | `assign_plates` | plates, warp | `plate_id` |
| 4 | Stress | `compute_stress` | plate_id, plates | `stress`, `boundary_mask`, `boundary_type`, `shear` |
| 5 | Flexure | `compute_flexure` | boundary_mask, stress | `flexure_field` |
| 6 | Base crust blur | `gauss_blur(base_raw)` | plate_id, plates | `base_field` |
| 7 | Crustal age | `build_age_field` | boundary_mask | `age_field` |
| 8 | Heterogeneity | `compute_heterogeneity` | age, warp, seed | `heterogeneity_field` |
| 9 | Resistance | `compute_resistance` | plate_id, plates, age | `resistance_field` |
| 10 | *(orogeny — WS only, skipped at defaults)* | `trace_boundaries` → `build_orogeny_field` → `smooth_orogeny` | — | `oro` |
| 11 | Height | `compute_height` | base, stress, flexure, hetero, age, warp, oro | `raw_height` |
| 12 | Normalise | `normalize_field` | raw_height | `field` |
| 13 | Volcanism | `stamp_volcanoes_provinces` | boundary_mask, stress, plate_id | `field`, `volcanic_field` |
| 14 | Impacts | `stamp_craters` | seed, g | `field`, `impact_field` |
| 15 | Clamp 0..1 | inline | field | `field` |
| 16 | **Flow #1 (area)** | `compute_flow(use_rain=false)` | field | `flow_area` |
| 17 | **Temperature #1** | `compute_temperature` | field | `temperature` |
| 18 | **Weather #1** | `simulate_weather` | field | `rainfall` |
| 19 | Moisture correctors #1 | `apply_climate_moisture_correctors` | field, **flow_area** | `rainfall` |
| 20 | Ocean currents #1 | `apply_ocean_currents` | field | `temperature`, `rainfall` |
| 21 | **Flow #2 (discharge)** | `compute_flow(use_rain=true)` | field, **rainfall** | `flow_discharge` |
| — | *`if carve_rivers` (default true):* | | | |
| 22 | Stream-power carve | `stream_power_kernel` (9 iters) | stress, resistance, **rainfall** | `field` |
| 23 | Isostatic rebound | `isostatic_rebound` | pre-erosion field | `field` |
| 24 | *(dynamic lithology — off at defaults)* | `recompute_resistance_after_erosion` | — | `resistance_field` |
| 25 | **Flow #3 (network)** | `compute_flow(use_rain=true)` | **carved** field, rainfall | `flow_for_network` |
| 26 | Channels | `build_channels` | field, flow_for_network | `ch.recv`, `ch.chan` |
| 27 | Strahler | `strahler_from_receivers` | ch.recv, flow_for_network | `order` |
| 28 | Polylines | `trace_river_polylines` | order, ch.recv | `polys` |
| 29 | Descent carve + lock | `enforce_channel_descent` per polyline | field, order | `field`, `river_mask`, `river_floor` |
| 30 | **Flow #4 (post-carve)** | `compute_flow(use_rain=true)` | **carved** field, rainfall | `flow_discharge` |
| 31 | **Temperature #2** | `compute_temperature` | **carved** field | `temperature` |
| 32 | **Weather #2** | `simulate_weather` | **carved** field | `rainfall` |
| 33 | Moisture correctors #2 | `apply_climate_moisture_correctors` | field, **flow_discharge** | `rainfall` |
| 34 | Ocean currents #2 | `apply_ocean_currents` | field | `temperature`, `rainfall` |
| — | *`if passes.any()` — **all off at defaults**, block skipped entirely:* | | | |
| 35 | velocity / glacial / coastal / hillslope | `velocity_erode_kernel`, `glacial_kernel`, `coastal_process`, `hillslope_diffuse` | field, rainfall, temperature, discharge | `field` |
| 36 | **`evolve_cycles` × (carve + rebound + full climate refresh)** | `stream_power_kernel` + `isostatic_rebound` + **`refresh_climate`** | field, rainfall | `field`, `temperature`, `rainfall`, `flow_discharge` |
| 37 | Sediment fill | `stream_power_kernel` + `compute_flow` + `route_sediment` | field, rainfall | `field`, `flow_discharge` |
| 38 | Final clamp + one `refresh_climate` | inline + `refresh_climate` | field | all four |

All **[code]**.

**Count of whole-grid re-runs at defaults:** `compute_flow` **×4**,
`compute_temperature` **×2**, `simulate_weather` **×2**,
`apply_climate_moisture_correctors` **×2**, `apply_ocean_currents` **×2**.
`ocean_sst_anomaly` runs **×4** — twice inside the two `simulate_weather`
calls (gated on `WeatherParams::currents`, `true` here) and twice inside the
two `apply_ocean_currents` calls. **[code]**

The `generate_terrain` source itself acknowledges the flow count: *"flow
accumulation is called up to FOUR times below … so its pipeline is built once
here rather than per call."* **[code]**

## 1.3 The civilisation pipeline, in execution order

`compute_civilisation`, all **[code]**:

```
build_water_bodies ─┬─► build_biome_raster
                    │
build_slope_field ──┤   build_lithology ──► build_soil_fertility
                    │           ▲                    │
                    │           └── age/volc/crust/resist/rain (WorldState)
                    │
river_flow_thresh ──┴─► build_water_access ──┐
                                             ├─► build_carrying_capacity ──┐
      (build_wetland_mask, opt-in) ──────────┘                             │
                                                                           │
build_resource_potentials (15 fields) ─────────────────────────────────────┤
build_raw_slope_field → build_route_corridors ─────────────────────────────┤
build_landmass_quality / build_coast_sdf / build_flood_field ──────────────┤
fresh_river_order ─────────────────────────────────────────────────────────┤
                                                                           ▼
                                                    build_settlement_suitability
                                                                           │
                                       find_settlement_seeds ──────────────┤
                          place_settlements_with_water_edge_snap ──────────┤
                     civ_hierarchical_network_topology (roads) ────────────┤
              [opt] civ_select_metropolises (needs road betweenness) ──────┤
                       name_and_populate_settlements_with_rng ─────────────┤
                                [opt] civ_seed_villages ───────────────────┤
                                [opt] civ_apply_recovery ──────────────────┤
                    civ_world_mean_resources + trade balances ─────────────┤
                       explain_settlement_suitability (per settlement) ────┤
                                        build_travel_cost ────────────────►│
                                                assign_territory ──────────┤
                                            civ_generate_provinces ────────┤
                                    civ_consolidate_and_smooth_ways ───────┤
                                                 civ_sea_routes ───────────┤
                                       civ_current_agrarian_density ───────┘
```

**This half is a strict DAG. It has no feedback edges at all.** **[code]** Two
things that *look* like feedback are not:

- `civ_select_metropolises` runs after roads and reads their betweenness, then
  promotes settlements. Roads are not rebuilt afterwards. The source comment
  is explicit that this is the reference's own ordering. **[code]**
- `civ_consolidate_and_smooth_ways` runs after naming because it needs names —
  it consumes `topology`, it does not re-derive it. **[code]**

There is one genuine *duplicate computation* across the boundary:
`build_water_bodies` is called inside `compute_civilisation` and again in
`absorb()` to seed the Paint editor, because the first call's result is not
retained past `compute_civilisation`'s local scope. The source discloses this
in a comment: *"a second, cheap call to the same pure function, not a new
algorithm."* **[code]**

## 1.4 The real feedback edges

Stripping the DAG down to the edges that are genuinely *cyclic* — a later
stage causing an earlier stage's output to be recomputed or discarded:

```
   ┌─────────────────────────────────────────────────────────────┐
   │  TECTONICS ──► HEIGHT ──► VOLCANISM/IMPACTS ──► field       │
   └─────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
                 ┌──── flow #1 (area) ───────────────┐
                 │                                    ▼
                 │                          CLIMATE (T, wind, rain)
                 │                                    │
                 │  ◄── E1 ──  rain seeds ──────────► │
                 ▼                                    │
             flow #2 (discharge)  ◄───────────────────┘
                 │
                 ▼
   ┌─────────────────────────────────────────────────────────────┐
   │  CARVE: stream power (rain-weighted) → rebound → channels   │
   └─────────────────────────────────────────────────────────────┘
                                  │
                                  │  E2  (the one real feedback edge)
                                  ▼
             flow #4  ──►  CLIMATE re-run from scratch  ──► rain, T
                                  │
                                  ▼
   ┌─────────────────────────────────────────────────────────────┐
   │  [opt] EROSION PASSES; evolve_cycles closes E2 N more times │  ◄── E3
   └─────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
   ┌─────────────────────────────────────────────────────────────┐
   │  CIVILISATION — pure DAG, no feedback                       │
   └─────────────────────────────────────────────────────────────┘
```

| Edge | What it is | Where | Cost per closure | Runs at defaults? |
|---|---|---|---|---|
| **E1** | Climate needs topography; the corridor moisture corrector needs drainage; drainage needs runoff. Resolved by **one** iteration: `flow(area)` → `climate` → `flow(discharge)`. | `generate_terrain`, stages 16–21 | 1 extra `compute_flow` | **Yes**, always |
| **E2** | Erosion changed the surface, so discharge and climate no longer describe it. `refreshClimate()`'s shape: `compute_flow` + `compute_temperature` + `simulate_weather` + correctors + currents. | `generate_terrain`, stages 30–34; and `refresh_climate()` as a `pub fn` | 1 flow + 1 temperature + 1 weather + 2 correctors | **Yes**, once (inside `carve_rivers`) |
| **E3** | `evolveCoupled` — the genuinely iterative loop. Carve → rebound → **full** `refresh_climate` → repeat, so next cycle's rain reflects the orography the last built. | `generate_terrain`, `ErosionPassParams::evolve_cycles` | E2's cost **per cycle** | **No** — `evolve_cycles: 0` |
| **E4** | Dynamic lithology: erosion strips layers, resistance is recomputed, next erosion pass reads it. | `recompute_resistance_after_erosion`, gated on `tect.dynamic_lithology` | one O(N) pass | **No** — `false` |
| **E5** | Sediment fill: carve, then re-derive discharge *on the carved surface*, then route mass down it. | `route_sediment` block | 1 extra `compute_flow` | **No** — `sediment_fill: false` |

All **[code]**.

**So: at default parameters this pipeline has exactly two closed loops — E1 and
E2 — and both are single-shot, not iterated to convergence.** Everything else
is one-way. **[code]** This matches the repository's own June-2026 audit
verdict: *"Predominantly a one-way cascade with one genuine feedback edge and
two partial ones."* (`docs/research/system-coupling-audit.md`) **[repo-doc]**

And E1's existence is not an accident. `docs/research/pipeline-order-audit.md`
records it as a deliberate design decision, closing "gap 1" (flow was
originally computed *before* climate, so rivers ignored rainfall), with the
explicit note: *"The circularity … is resolved the way coupled
landscape-evolution models do — iterate once … One extra O(N log N)
accumulation per generate — negligible."* **[repo-doc]**

## 1.5 Post-generation edit paths, and what they do and do not re-run

This is where the owner's "different generation cycles all flowing from one
system to the next" is most visible, and where the port has already made
several conscious *anti*-feedback decisions:

| Entry point | Re-runs | Explicitly does **not** re-run |
|---|---|---|
| `WorldGen::sculpt_commit` | Bake stamps, re-clamp locked river channels, carve+lock new river stamps, deposit lakes. Marks tiles dirty. **[code]** | Erosion, hydrology, climate, civ. The doc comment is blunt: *"the eager form was measured at ~7s/stroke at 2048² and rejected."* And: *"this binding does not currently expose an entry point that consumes that dirty set."* **[code]** |
| `WorldGen::carve_fjords` | The fjord mask and the carve. Pushes undo. **[code]** | *"Flow, river extraction and climate are **not** recomputed … this port has no re-runnable path for those."* **[code]** |
| `WorldGen::center_landmasses` | Rotates every retained raster. **[code]** | Drops `civ` and `sculpt` to `None` rather than shifting them — coordinates would be meaningless. Requires a full re-generate for a civ layer. **[code]** |
| `WorldGen::paint_commit` | Bakes paint overrides only. **[code]** | Nothing. Reports `stale_stages: ["ecology_biomes", "resources_soils"]` **as data for a status bar**, not as a live invalidation. **[code]** |
| `param_set` / `reset_params` / `apply_archetype` (GDScript) | Nothing. Sets `params_dirty = true`. **[code]** | Everything. `engine_bridge.gd`: *"Cartalith is a one-shot generator: a moved dial does not recompute a stage, it marks the world stale until the next full generate."* **[code]** |

**This is the single most important fact for answering the owner's question.**
**[judgement]** The product is a batch generator. There is no continuous
simulation loop, no per-frame recompute, no live convergence. Every "feedback
loop" cost is paid **once per Generate click**, bounded, and then never again
until the next click.

## 1.6 The staleness graph exists, is correct, and is not wired in

`cartalith_engine::staleness::pipeline_stage_graph(tile_count)` builds a real
`cartalith_spatial::StageGraph` over four stages — `height → hydrology →
climate → civ` — with the extra direct edges (`height → civ`,
`hydrology → civ`, `height → climate`) that the linear spine misses. It has
transitive staleness, per-tile dirty tracking, and reason strings. It is
tested. **[code]**

Its module doc opens: *"**Unwired on purpose.** Nothing in `generate_terrain`
or any other pipeline entry point calls this yet."* **[code]** A grep confirms
it: the only references to `pipeline_stage_graph` in the entire workspace are
its own definition and its own tests. **[code]**

Its "what this deliberately does not model" section is directly on-topic for
this research:

> "Erosion, which sits between height and hydrology and is genuinely
> two-way-coupled with climate (`ARCHITECTURE.md` already flags
> `evolveCoupled()` as the known acyclicity pressure point). A cycle cannot be
> expressed here by construction, and inventing an edge direction for it before
> a tool actually needs one would be guessing." **[code]**

`ARCHITECTURE.md` said the same thing before any code existed: *"One known
pressure point: climate and erosion may need a tighter loop than a one-way
dependency allows … Read that function before assuming the graph stays
acyclic."* **[repo-doc]**

## 1.7 What it actually costs

Measured on this project's own benchmark sizes and machine.

**End-to-end, CPU, `generate_terrain` only** (`CPU_MULTITHREADING_SCOPE.md`,
after the Rayon pass) **[repo-doc]**:

| Size | Before Rayon | After Rayon |
|---|---|---|
| 128² | 0.0973 s | 0.0936 s |
| 512² | 0.6019 s | 0.4859 s |
| 1024² | 1.8328 s | 1.3143 s |
| 2048² | 7.0670 s | 5.1071 s |

**Civ layer (`cartalith-civ` per-cell functions)**, same document: 2048²
3.5568 s → 1.9625 s. Combined: *"roughly 10.62 s sequential to roughly 7.07 s
parallelized."* **[repo-doc]**

**Per-stage, the numbers that matter here:**

| Stage | Cost | Source |
|---|---|---|
| `compute_flow`, one call, 2048², CPU | **488.9 ms** (GPU: 31.5 ms, 15.5×) | `cartalith-native/docs/CHANGELOG.md` **[repo-doc]** |
| `compute_flow`, one call, 1024², CPU | ~10.4× slower than GPU | same **[repo-doc]** |
| Climate refresh (T + weather + correctors), whole grid | 37–53 ms CPU, 25–41 ms GPU, **near-flat with grid size** | `SCULPT_LIVE_SCOPE.md` L0 **[repo-doc]** |
| Full "bake + reclamp + carve + lake, then flow, then climate" | ~123 ms @512², ~204 ms @1024², **~564 ms @2048²** CPU; ~94/100/131 ms with GPU flow+weather | `SCULPT_LIVE_SCOPE.md` L0 **[repo-doc]** |

**The derived figure that reframes the owner's question** **[inference]**:
four `compute_flow` calls × 488.9 ms ≈ **1.96 s of a 5.11 s CPU generate at
2048² — about 38%.** This is independently corroborated from the other side:
the JS engine's own audit found *"the `carveRivers` pipeline (41% of
generate)"* was the largest remaining cost, and separately measured
`computeFlow`'s sort as *"the single hottest generate() line"*
(`docs/research/performance-audit-gen1.md`). **[repo-doc]**

Climate, by contrast, is *cheap and nearly grid-independent*, because
`simulate_weather` runs on a coarse grid capped at `min(gw, 240)`. **[code]**
**[repo-doc]** So the E2 feedback edge — the one the owner is most likely
picturing when they say "feedback loops" — is dominated by its
`compute_flow` component, not by its climate component.

---

# Part 2 — How real systems and real models handle this coupling

## 2.1 Does nature run as one simultaneous system?

Physically, yes: tectonics, topography, climate, erosion and hydrology are
mutually coupled, continuously, everywhere. There is no natural "stage order."

But nature also does something a simulator can exploit: it **separates
timescales by orders of magnitude.** Attaining steady state between topography
and tectonic forcing in an active orogen takes roughly 10⁶–10⁷ years; knickpoint
propagation after a change in uplift rate is ~10⁶ years; eroding through enough
rock for steady state is on the order of 10⁵ years
([Allen 2008, *Time scales of tectonic landscapes and their sediment routing
systems*](https://www.lyellcollection.org/doi/abs/10.1144/sp296.2))
**[source·indirect — abstract-level, full text not retrieved]**. Climate varies
on 10⁰–10⁵ years. Weather varies on 10⁻² years.

**[inference]** That separation is *why* staged pipelines are physically
defensible rather than merely convenient. Over the interval in which the
tectonic substrate is essentially fixed, climate has already equilibrated to it
thousands of times over. Computing tectonics once and climate once "on top of"
it is not a shortcut around physics; it is an explicit statement that the
system is stiff and the fast variable is slaved to the slow one.

The places where that breaks down are exactly the places this repository's own
audit already identified as real coupling: erosion changing orography enough
to change the rain that drives the erosion (E2/E3), and the glacial buzzsaw
(glacial erosion pinning mountain height to the snowline, which is itself a
climate variable — Egholm et al. 2009, *Nature* 460, cited in
`docs/research/pipeline-order-audit.md`) **[repo-doc]**.

## 2.2 Scientific models decompose too — universally, and for exactly this reason

This is the crux of the owner's question, and the literature is unambiguous.

### Landscape Evolution Models

**Landlab** — the most widely used component-based LEM framework — states the
decomposition outright in its user guide. Fetched and quoted verbatim:

> "during each global time step, you are effectively de-coupling the components
> for a small interval of time"

and, on the hazard that creates:

> "if you have one component that calculates solar radiation on a landscape and
> another that calculates the resulting evapotranspiration (ET), when you run
> the ET component, you assume that the radiation is constant for the duration
> of the time step — as if the sun's position in the sky became frozen for that
> period of time"

and on the remedy, which is sub-stepping *inside* a component, not global
simultaneity:

> "if the component determines that 1 year is the biggest step it can get away
> with, then it should divide the requested 10-year run into 10 steps of 1 year
> each"

([Landlab user guide, "Time steps"](https://landlab.readthedocs.io/en/latest/user_guide/time_steps.html))
**[source — retrieved and quoted]**

That is *operator splitting*, named plainly. Landlab's own advice for choosing
the global step is empirical: *"test your model with different global time-step
sizes to identify a time step that is small enough not to impact the solution
in a significant way."* **[source]**

**SPACE 1.0**, the Landlab component for coupled sediment transport and bedrock
erosion, is explicitly designed to be composed with *separate* components for
basin hydrology, hillslope evolution, weathering and lithospheric flexure —
i.e. the physics is split across independently-stepped modules by design
([Shobe, Tucker & Barnhart 2017, *Geosci. Model Dev.* 10:4577](https://gmd.copernicus.org/articles/10/4577/2017/gmd-10-4577-2017.pdf))
**[source·indirect — abstract and framing verified via search, full PDF not read]**.

**FastScape** is the tractability argument in its purest form. Braun & Willett
(2013) give an **O(n), implicit, unconditionally stable** solver for the stream
power equation — implicit specifically so that *"large time steps can be used
without affecting numerical stability"*
([Braun & Willett 2013, *Geomorphology* 180:170–179](https://www.sciencedirect.com/science/article/abs/pii/S0169555X12004618))
**[source·indirect — abstract-level; the scheme itself is already credited in
this port's own `stream_power_kernel` comments as "Cordonnier"]** **[repo-doc]**.
FastScape is then *coupled to* geodynamic codes (ASPECT, FANTOM) through a
Fortran interface — again, separate solvers exchanging state, not one monolith
([GFZ FastScape project page](https://www.gfz.de/en/section/earth-surface-process-modelling/projects/current-projects/fastscape-landscape-evolution-model-development),
[fastscape.org](https://fastscape.org/)) **[source·indirect]**.

The point of an implicit unconditionally-stable scheme is worth stating
plainly **[inference]**: it exists precisely *because* the coupled system is
too stiff to integrate simultaneously at a physical timestep. The response to
stiff coupling in the geomorphology literature is not "solve it all at once,"
it is "split it, and make each split piece take bigger steps."

### Earth System Models

Full climate models — the most expensive coupled simulations humans run — are
built the same way. CESM's CPL7 driver runs *"component models … sequentially,
concurrently, or in some mixed sequential/concurrent layout"*, with the coupler
responsible for *"flux calculations, mapping (regridding), diagnostics"*
([CESM CPL 7.0](https://www.cesm.ucar.edu/models/cpl/7.0))
**[source — retrieved]**. Each component solves its own PDEs with its own
numerics, resolution and timestep; the coupler exchanges state at fixed
intervals.

Those intervals are *different per component pair* — atmosphere/land/ice
coupled far more often than atmosphere/ocean, on the order of 30 minutes vs 12
hours **[source·indirect — obtained from a search summary of CESM
configuration documentation; I could not retrieve the primary page or the
CPL7 paper ([Craig, Vertenstein & Jacob 2012, *IJHPCA*](https://www.mcs.anl.gov/uploads/cels/papers/P1838.pdf),
403 on fetch), so treat the specific numbers as illustrative rather than
citable]**.

**[inference]** The generalisable principle is not the numbers: it is that
"how often do these two subsystems need to talk?" is a **tunable per-edge
parameter**, chosen from the physics of each pair, not a single global answer.
That is directly applicable here (see §3.2.4).

### The honest summary of §2.2

**Nothing in the scientific literature runs a coupled tectonics–climate–erosion
system simultaneously.** Every serious model splits it, steps the pieces
separately, and exchanges state at chosen intervals. The differences between
models are *which* pieces, *how often* they exchange, and *how big* a step each
piece dares take. **[inference, but strongly supported]**

## 2.3 The one genuine escape hatch: analytical steady state

There *is* a line of work that does something close to what the owner is
imagining — deriving the landscape directly rather than iterating to it — and
it is worth knowing about precisely because its limits are instructive.

Tzathas, Gailleton, Steer & Cordonnier (2024) open with the exact dichotomy
the owner is probing:

> "Terrain generation methods have long been divided between procedural and
> physically-based. Procedural methods build upon the fast evaluation of a
> mathematical function but suffer from a lack of geological consistency, while
> physically-based simulation enforces this consistency at the cost of
> thousands of iterations unraveling the history of the landscape."

([*Physically-based analytical erosion for fast terrain generation*, Computer
Graphics Forum 43(2), 2024](https://onlinelibrary.wiley.com/doi/abs/10.1111/cgf.15033);
abstract quoted from [physicsbasedanimation.com](https://www.physicsbasedanimation.com/2024/05/04/physically-based-analytical-erosion-for-fast-terrain-generation/))
**[source — abstract retrieved and quoted; full PDF not parseable]**

Their contribution is to exploit **analytical solutions of the stream power
law**, so that time becomes *"the parameter of a mathematical function, a
slider that controls the aging of the input terrain."* **[source·indirect —
from the search summary of the same paper]**

**But note the caveat that comes with it, in the authors' own framing:**
analytical solutions exist for the **1D** case in the geomorphology literature;
*"extending them to a 2D heightmap proves challenging,"* and their solution is
*"an efficient implementation with a multigrid accelerated iterative process"*
plus separate handling for landslides and hillslope processes.
**[source·indirect]**

**[inference]** So even the state-of-the-art "closed form" answer is *still
iterative*, just with far better convergence. The irreducible reason is
structural: the stream power law's incision term depends on drainage area,
drainage area depends on flow routing, and flow routing depends on the
elevation the equation is solving for. That circular dependency is not an
artefact of any implementation — it is the equation.

## 2.4 What procedural generators actually do

Six data points, ordered from "most like Cartalith" outward.

**Cordonnier et al. 2016** — the closest academic analogue to what
`generate_terrain` does. Given a user-painted uplift map, they combine crustal
uplift with stream-power erosion over a **stream graph** embedding elevation
and flow, then convert that graph to a DEM by blending landform feature
kernels, *"generat[ing] large realistic terrains at a low computational cost"*
([Cordonnier et al., *Computer Graphics Forum* 35(2), 2016](https://onlinelibrary.wiley.com/doi/10.1111/cgf.12820))
**[source·indirect — abstract-level; PDF fetch exceeded size limit and the
publisher pages 403'd]**. **[inference]** The architectural lesson is that they
work on a *graph abstraction of the drainage network* rather than a full raster
per iteration — the graph is the coupling term, and it is what gets reused
across iterations.

**Cordonnier et al. 2017** went the *other* direction and added coupling on
purpose: *"bi-directional feedback between erosion and vegetation simulation …
Vegetation and terrain erosion have strong mutual impact and their interplay
influences the overall realism of virtual scenes, despite these complex
interactions being neglected in computer graphics"*
([ACM TOG 36(4):134](https://dl.acm.org/doi/10.1145/3072959.3073667))
**[source·indirect]**. **[judgement]** This is worth the owner's attention: the
graphics research community's trajectory over the last decade has been to add
feedback loops, because that is where the realism was, not to remove them.

**Schott et al. 2023** is the best evidence that a coupled loop can be made
interactive without abandoning it. They model in the **uplift domain rather
than the elevation domain** and use *"a fast yet accurate approximation of
drainage area and flow routing to compute the erosion interactively."* The
reported cost: *"One iteration takes a few milliseconds and the stream power
erosion algorithm converges to the final steady-state elevation in a few
seconds for medium sized grids, i.e., ≤ 1,024 × 1,024."*
([*Large-scale Terrain Authoring through Interactive Erosion Simulation*, ACM
TOG 42(5):162](https://dl.acm.org/doi/10.1145/3592787); code at
[H-Schott/StreamPowerErosion](https://github.com/H-Schott/StreamPowerErosion))
**[source·indirect — quotation obtained via search summary of the paper; the
ACM and HAL pages both 403'd on direct fetch]**

**[inference]** Compare that with this port's numbers: **a few seconds to full
convergence at 1024²** for an iterated stream-power loop, against
**1.31 s for one non-iterated pass** here. The gap is not enormous, and the
technique bridging it is the drainage-area approximation.

**Schott et al. 2024 — multi-scale erosion** is the coarse-to-fine answer
applied to exactly this problem: amplify a low-resolution input into a
high-resolution hydrologically-consistent terrain by running *"thermal, stream
power erosion and deposition … at different scales"*
([*Terrain Amplification using Multi-scale Erosion*, ACM TOG 43(4), 2024](https://dl.acm.org/doi/abs/10.1145/3658200);
[HAL record](https://hal.science/hal-04565030); code at
[H-Schott/MultiScaleErosion](https://github.com/H-Schott/MultiScaleErosion))
**[source·indirect — abstract-level; the HAL PDF is behind an anti-bot wall]**

**Dwarf Fortress** — a shipped game whose worldgen is the classic staged
pipeline. Per the wiki's summary of Toady One's own description, the order is:
allocate → choose pole → seed base fields (elevation, rainfall, temperature,
drainage, volcanism, wildness) on a grid → fill fractally → adjust altitudes →
derive vegetation → check biome rejections → smooth mid elevations → place
volcanoes → **erosion and river stage** (dry small oceans; "many fake rivers
flow downward from these points, carving channels"; then permanent rivers) →
**smooth elevations again** → **recalculate temperature and rainfall against
the new elevation** → identify biomes → geological layers → wildlife/weather →
history
([Dwarf Fortress Wiki, World generation](https://dwarffortresswiki.org/index.php/World_generation))
**[source — retrieved]**; see also
[Tarn Adams, "Simulation Principles from Dwarf Fortress", *Game AI Pro 2*, ch. 41](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf)
**[source·indirect]**.

**[inference]** Note what that sequence contains: erosion, then a climate
recalculation *against the eroded surface*. That is **E2**. One of the most
elaborate procedural worlds ever shipped independently arrived at the same
single feedback edge Cartalith has, in the same place, for the same reason.
That is a meaningful convergent-design signal.

**mapgen4** (Amit Patel) — the minimal end of the spectrum. Elevation (noise +
distance fields + user painting) → rainfall/moisture (wind, orographic) →
rivers (downslope assignment + flow accumulation) → biomes, and the pipeline
is **unidirectional**, with *"only changed map aspects … recalculated when
parameters change"*
([redblobgames/mapgen4](https://github.com/redblobgames/mapgen4);
[mapgen4](https://www.redblobgames.com/maps/mapgen4/))
**[source·indirect — pipeline order and the incremental-recompute claim came
from a third-party generated wiki
([DeepWiki](https://deepwiki.com/redblobgames/mapgen4/4-map-generation)),
not from Patel's own writing; treat as indicative, not authoritative]**.
The stated design goal is to *"run fast enough to regenerate in real time as
you paint"* — and the price paid for that is no erosion feedback at all.
**[inference]**

**World Machine / Gaea** — the node-graph DCC tools. Both are explicit DAGs
with cached node outputs. Gaea's documentation on baking is the relevant part:
*"Baking means building a node to a specific high resolution and then storing
it in a locked or 'baked' form"*; *"When building the final terrain, the Build
Swarm will use that cache for those nodes instead of building them. This can
significantly speed up build times"*; and the invalidation rule,
*"Changing the resolution AFTER nodes have been baked will invalidate existing
bakes."*
([Gaea docs, Baking and Caching](https://docs.quadspinner.com/Guide/Using-Gaea/Cache.html))
**[source — retrieved and quoted]**; [World Machine](https://www.world-machine.com/)
**[source·indirect]**.

**[judgement]** The node-graph tools are the industry's answer to precisely the
owner's question, and their answer is: *make the pipeline an explicit DAG,
cache every node, invalidate downstream on change, and let the user pin
(bake) the expensive upstream portions.* They do not eliminate the staging —
they make it addressable.

## 2.5 Techniques for making iterative coupling cheap without abandoning it

Five families, with what each would actually mean here.

**(a) Better asymptotics on the hot kernel.** FastFlow (Jain et al. 2024)
gives GPU flow routing in *"O(log n) iterations for a terrain with n
vertices"* and depression routing in *"O(log² n) iterations"*, reporting
**5× speedup on flow routing and 34–52× on depression routing at 1024²**, with
the explicit motivation of enabling *"interactive control of terrain
simulation"*
([*FastFlow: GPU Acceleration of Flow and Depression Routing for Landscape
Simulation*, Computer Graphics Forum 43, 2024](https://onlinelibrary.wiley.com/doi/10.1111/cgf.15243))
**[source — retrieved]**. This port has already independently built something
in this family: `gpu_flow.wgsl` uses *"per-cell D8 direction + pointer-doubling
subtree sums"* rather than a translation of the CPU sort-and-walk. **[code]**

**(b) Implicit / unconditionally stable schemes** so each step can be large —
Braun & Willett 2013, already discussed and already the basis of this port's
`stream_power_kernel`. **[source·indirect]** **[repo-doc]**

**(c) Multigrid / coarse-to-fine.** Recover large-scale structure on a coarse
grid cheaply, interpolate onto progressively finer grids as an improved
initial guess. This is the classical multigrid idea and it is exactly what
Tzathas et al. 2024 use to make their analytical solution tractable in 2D
**[source·indirect]**, and what Schott et al. 2024 build a whole terrain
amplification method around **[source·indirect]**.

**(d) Dirty-region / incremental invalidation.** Recompute only what changed.
This is what the node-graph tools do, what mapgen4 claims to do, and what this
repository has *already built the machinery for* and deliberately not wired
(`cartalith_spatial::StageGraph` / `DirtyTracker` / `PassBuffer`,
`cartalith_engine::staleness`). **[code]**

**(e) Asymmetric coupling frequency.** Different edges exchange at different
intervals, chosen per-pair from the physics — the ESM coupler pattern.
**[source·indirect]**

---

# Part 3 — Recommendation for this project

## 3.1 The direct answer to the owner's question

**"Can't we have everything run from the first generation and keep feedback
loops to a minimum?"**

**You already do, and to a greater degree than almost any comparable system.**
**[code]** **[judgement]**

At default parameters the terrain pipeline has two closed loops, both
single-shot: E1 (the one-iteration flow↔climate resolution, which exists
because rivers ignoring rainfall was a *correctness* bug — see
`docs/research/pipeline-order-audit.md` gap 1 **[repo-doc]**) and E2 (one
climate refresh after the carve, which Dwarf Fortress independently arrived at
too **[source]**). The civilisation half is a strict DAG with zero feedback
**[code]**. Everything genuinely iterative — `evolve_cycles`, dynamic
lithology, sediment routing — is **off by default and asserted bit-identical
when off** by `erosion_passes_off_leave_generation_bit_identical` **[code]**.
And no edit path re-runs any upstream stage; a moved dial marks the world
stale and waits for the next Generate **[code]**.

**Could feedback be reduced further? Only two edges remain, and removing
either has a stated cost:**

- Removing **E1** returns the engine to the state
  `docs/research/pipeline-order-audit.md` calls a bug: *"rivers ignore rainfall
  and erosion ignores rivers' real discharge."* **[repo-doc]** Not
  recommended.
- Removing **E2** means the final rainfall, temperature and discharge describe
  the *pre-carve* surface. Rain shadows would sit on ridges that erosion has
  since cut through, and every civ field — soil, carrying capacity, settlement
  suitability, biome — reads those stale fields. **[inference]** Also not
  recommended.

**"…without blowing up resources or making generation take forever?"**

**[inference]** The cost is not in the feedback. At 2048² on CPU, four
`compute_flow` calls are ≈1.96 s of a 5.11 s `generate_terrain` — about 38% —
while the entire climate half of E2 costs 37–53 ms and is nearly
grid-independent **[repo-doc]**. If generation feels slow, the fix is in flow
accumulation and in the civ layer's per-cell rasters, not in loop structure.

**So the honest headline: the architecture the owner is asking for is the
architecture that already exists. The question worth asking instead is "why is
flow accumulation run four times, and does each run earn its place?" — and
there, there is a real finding.** **[judgement]**

## 3.2 Concrete opportunities, ranked, with citations

### 3.2.1 One of the four `compute_flow` calls appears to be dead work — **highest value, smallest change**

**The finding.** In `generate_terrain`, `flow_discharge` is computed
pre-carve (`let mut flow_discharge = match flow_on_gpu(&field, Some(&rainfall),
true) { … }`, ≈ line 1112) and then **unconditionally overwritten** post-carve
(`flow_discharge = match flow_on_gpu(…)`, ≈ line 1192, the block commented
*"(3) recompute so overlay + rainfall reflect the carved valleys"*). I read
every statement between those two points: the carve block uses `pre`,
`stress.stress_field`, `resistance_field`, `rainfall`, and its own locally
computed `flow_for_network` — **it never reads `flow_discharge`.** **[code]**

**Therefore, when `p.carve_rivers` is `true` (the default), the pre-carve
`compute_flow` call's result is discarded unread.** **[code]** **[inference]**

**What it is worth.** One `compute_flow` at 2048² CPU = **488.9 ms**
**[repo-doc]** — roughly **10% of a 5.11 s default generation**, for deleting a
call, not for writing an algorithm.

**Why it is there.** It is a faithful port. The reference JS `generate()` does
`computeFlow(true)` before `carveRiverValleys()` because `flowField` is a
module global the renderer and overlays can read at any moment, and
`carveRiverValleys` is conceptually a separate op. **[repo-doc]** **[inference]**
In Rust it is a local that nothing observes.

**Caveats that must be honoured before touching it.**
- When `carve_rivers` is `false`, this call **is** the output. The skip must be
  conditional, not unconditional.
- It writes `gpu_stages_used.push("flow")` on the GPU path; skipping changes
  that vector's contents on a `use_gpu` run where no other flow call reached
  GPU. `generate_terrain_gpu_path_is_deterministic_and_valid` asserts on it.
  **[code]**
- Per `CLAUDE.md`, this is a deviation from the reference's own call sequence
  and must be **disclosed in the source**, not silently taken — even though it
  is output-identical.
- **It must be proven, not assumed.** The proof is cheap: assert
  `generate_terrain` output is `assert_eq!`-identical with and without the
  call, at several seeds and both `world` modes, in the shape
  `erosion_passes_off_leave_generation_bit_identical` already uses. **[code]**

### 3.2.2 `compute_flow`'s sort is the same hot line the JS engine already fixed — and the fix is explicitly sanctioned here

**The finding.** `cartalith_hydrology::compute_flow` orders cells with

```rust
order.sort_by(|&a, &b| flow_cmp_desc(field[a], field[b]).then(a.cmp(&b)));
```

— a comparison sort over `N` indices with an indirection into `field`, i.e.
O(N log N) and cache-hostile. **[code]**

The reference JS used to do the same thing and **replaced it with an LSD radix
sort on IEEE-754 bit patterns**, measured at **1,005 ms → 120 ms per call**,
proven element-identical, and described as *"the single hottest generate()
line, ≥3 runs per generate + one per terrain edit."*
(`docs/research/performance-audit-gen1.md`) **[repo-doc]**

The Rust port carries the *pre-optimisation* form.

**Crucially, this is not a parity risk, and the port's own documentation says
so.** `flow_cmp_desc`'s doc comment:

> "The JS implementation is a radix sort operating on IEEE-754 bit patterns (an
> order-preserving float→uint key, inverted for descending order); the
> *algorithm* is a correctness-equivalent substitution target per
> `PROVENANCE.md` … only the ordering guarantee matters for parity, not the
> sort implementation" **[code]**

**Two quirks must survive the substitution**, both already documented in that
same comment: JS normalises `-0.0`'s key to `+0.0` (`if(b===0x80000000) b=0`),
which `f32::total_cmp` does not; and ties break by **ascending index**, so the
sort must be stable or the key must include the index. **[code]**

**What it is worth.** Unknown here, and I will not extrapolate the JS ratio —
different language, different data layout, different sort implementation. But
`compute_flow` is ~38% of a default CPU generate **[inference]**, and the sort
is a strictly-dominant term inside it in the one place it was ever measured
**[repo-doc]**. **[judgement]** This is the highest expected-value *measured
experiment* in this document: the JS reference implementation exists to port
from, the parity rule already permits it, and the answer is a benchmark away.

### 3.2.3 Duplicate `ocean_sst_anomaly` inside each climate refresh — small, bounded, real

`ocean_sst_anomaly` (which itself runs `build_wind` plus a 20-iteration
`compute_ocean_current`) is computed **twice per climate refresh** from the same
`field`: once inside `simulate_weather` when `WeatherParams::currents` is true,
and once inside `apply_ocean_currents`. **[code]** At defaults `currents: true`,
so a default generation computes it **four times** for **two** distinct
surfaces. **[code]**

**Bounded by design**: it runs on the coarse `min(gw, 240)` grid, so it does
**not** grow with resolution past 240. **[code]** **[judgement]** Worth a
memoisation only if a measurement shows it matters; do not assume it does. And
before caching, the two call sites' `ww`/`wh`/`step`/`wrap_x` arguments must be
**verified identical**, not eyeballed — they are computed independently in the
two functions. **[code]**

### 3.2.4 Wire the staleness graph that already exists — the real architectural opportunity

**[judgement]** This is the largest available win and it is not a performance
change; it is a change in what the product can do.

The pieces are built and tested: `cartalith_spatial::{StageGraph, DirtyTracker,
PassBuffer}`, `cartalith_engine::staleness::pipeline_stage_graph`,
`sculpt_commit` already returning `tiles_marked`, `paint_commit` already
returning `stale_stages` as data. **[code]** What is missing is a consumer:
`sculpt_commit`'s own doc says *"this binding does not currently expose an entry
point that consumes that dirty set."* **[code]**

The measured case for it is already on record: an eager
recompute-everything-on-commit was rejected at ~7 s/stroke at 2048²
**[repo-doc]**, while the *actual* downstream cost of a commit (bake + reclamp
+ carve + lake + flow + climate) is **~564 ms at 2048² CPU, ~131 ms with GPU
flow and weather** **[repo-doc]**. That is the difference between "unusable" and
"a beat after you release the drag."

**[inference]** And the literature endorses the shape: this is precisely what
World Machine and Gaea do (explicit DAG, cached nodes, downstream invalidation,
user-pinned bakes) **[source]**, what mapgen4 claims to do
**[source·indirect]**, and what the ESM couplers formalise as per-edge exchange
intervals **[source·indirect]**.

**The two honest obstacles, both already identified in-repo:**

1. **`cartalith-hydrology` and `cartalith-civ` are not tile-incremental.**
   `compute_flow` is a *global* descending-height walk — a wavefront dependency
   the port already confirmed rather than assumed **[code]**; `build_water_bodies`
   is a global priority flood **[repo-doc]**. You cannot recompute drainage for
   one tile. `SCULPT_LIVE_SCOPE.md` §6 makes this argument explicitly, and it
   stands. **[repo-doc]**
2. **Erosion↔climate is a genuine cycle** a `StageGraph` cannot express by
   construction. `staleness.rs` refuses to invent an edge direction for it, and
   that refusal is correct until a tool makes the question concrete. **[code]**

**[judgement]** The way through is to be honest about granularity: the *stage
graph* is the right mechanism at whole-field granularity even where per-tile
incrementality is impossible. "This commit made hydrology and climate stale;
recompute them once when the user asks, and leave civ stale until they ask for
that too" is a real, achievable, ~131–564 ms answer that needs no
tile-incremental hydrology at all.

### 3.2.5 Coarse-to-fine — genuinely promising, and genuinely blocked

**[source]** Both Tzathas et al. 2024 and Schott et al. 2024 make coarse-to-fine
the central mechanism, and the payoff in the literature is large.

**[judgement]** It is the strongest *long-term* idea in this document and the
one I would recommend **least** for near-term action, for a reason this
repository already wrote down. `TERRAIN_ARCHITECTURE_RESEARCH.md`'s own
applicability annotation names multi-resolution generation fields as the item
that *"would be a real, current numerical-parity-breaking change, since every
downstream JS formula this port has golden-verified assumes matching grid
dimensions across fields; not a free architectural win, a redesign of every
formula that reads two fields together."* **[repo-doc]** Nothing in the 2024
literature changes that constraint.

There is one narrow, already-precedented exception worth noting **[inference]**:
`simulate_weather` and `apply_ocean_currents` *already* run on a coarse
`min(gw, 240)` grid and bilinearly resample to full resolution **[code]** — so
the pattern is not foreign to this codebase, it is simply not extensible to
fields the golden fixtures pin at full resolution.

## 3.3 What I would not do, and why

**[judgement]**, each with its grounding:

- **Do not remove E1 or E2.** Both were added to fix identified realism
  failures, both are single-shot, and both together cost less than one
  `compute_flow` call. **[repo-doc]** **[inference]**
- **Do not chase "one simultaneous solve."** No LEM and no ESM does this
  **[source]**; the one paper that gets closest still needs a multigrid
  iteration **[source·indirect]**; and the underlying circularity (incision
  needs drainage area, drainage area needs elevation) is in the equation, not
  the implementation **[inference]**.
- **Do not enable `evolve_cycles` by default to "improve realism."** Each cycle
  costs a full `refresh_climate`, and the current default is asserted
  bit-identical to pre-existing goldens. Making it non-zero re-baselines every
  fixture. **[code]**
- **Do not restructure before measuring.** This project's own discipline
  (`CPU_MULTITHREADING_SCOPE.md`, `GPU_LAYER_INTEGRATION_SCOPE.md`) is
  measurement-first and has repeatedly produced honest negative results — GPU
  weather losing to CPU at 0.93× **[repo-doc]**, GPU flow losing at 128²
  (0.20×) **[repo-doc]**. Two of my three top recommendations (§3.2.1, §3.2.2)
  are cheap **experiments**, deliberately, not designs.
- **Do not treat `TERRAIN_ARCHITECTURE_RESEARCH.md`'s dirty-region/dependency-
  graph sections as a mandate.** That document assumes an interactive 3D editor
  Cartalith is not; its own annotation says so. What §3.2.4 proposes is far
  narrower: wire the graph that exists, at whole-field granularity, for the
  commit paths that already return dirty sets. **[repo-doc]** **[judgement]**

## 3.4 Two observations that fall out of the trace, offered without recommendation

- **`WorldState::flow_area` has no reader outside the pipeline.** A workspace
  grep finds it only in `generate_terrain` (where `apply_climate_moisture_
  correctors` #1 consumes it), `center.rs` (rotates it), `import.rs`, and
  tests. Nothing in `cartalith-godot/src/` reads it. **[code]** At 2048² that
  is a retained 16 MB `Vec<f32>`. It is part of `WorldState`'s public contract
  and `import.rs` refers to *"the drainage-area debug"* view as a consumer, so
  this may be a deliberately-retained affordance rather than an oversight —
  flagged, not judged.
- **`compute_temperature`'s first call is also unread on the default path.**
  Between it and its post-carve replacement, nothing reads `temperature`:
  `simulate_weather` builds its own coarse `tc` internally and takes no
  temperature argument; `apply_climate_moisture_correctors` takes none; and
  `apply_ocean_currents` reads `temp_row[x]` only to add to it, deriving its
  `sst_b` from `field` and latitude parameters alone. **[code]** Unlike §3.2.1
  the saving is small (one O(N) per-cell pass, not a global sort-and-walk), and
  skipping it requires restructuring `apply_ocean_currents`' `&mut temperature`
  argument. **[judgement]** Mentioned for completeness; probably not worth the
  churn on its own, but free if §3.2.1 is done and the block is being touched
  anyway.

---

## 4. Open questions for the owner

Per `DECISIONS.md`'s convention — raise, do not silently build.

1. **Is a measured experiment on §3.2.1 (the apparently-dead pre-carve
   `compute_flow`) authorised?** It is a ~10%-of-generate saving at 2048² for a
   conditional skip plus a bit-identity test. It is also a disclosed deviation
   from the reference's own call order, which `CLAUDE.md` requires be recorded
   rather than absorbed. **Approve the deviation, or decline it and leave the
   call as faithful-to-reference?**

2. **Is the radix-sort substitution in `compute_flow` (§3.2.2) worth a
   measurement pass?** `PROVENANCE.md` already classifies the sort algorithm as
   a correctness-equivalent substitution target, and the JS side has a
   proven-identical implementation to port. The question is only whether to
   spend the time, and whether the result should be gated behind a flag or
   taken outright once proven element-identical.

3. ~~**Should the staleness graph be wired at whole-field granularity
   (§3.2.4)?**~~ **RESOLVED — owner approved the engine-side half; IMPLEMENTED
   2026-08-24.**

   `cartalith_engine::staleness::recompute_stale(&mut StageGraph,
   &WorldParams, &mut WorldState) -> RecomputeReport` is the consumer. It
   re-runs exactly the stale downstream stages — hydrology and climate,
   through **one** `refresh_climate` rather than two calls, because that
   function's first statement already *is* hydrology's output and a second
   call would buy a duplicate whole-grid `compute_flow`. Civ, the carve-time
   river network and `flow_area` are left alone deliberately, each for a
   stated reason, each held to `assert_eq!` bit-identity by test.

   Wired into all three commit paths so the mechanism is not unused a second
   time: `WorldGen::sculpt_commit` marks `Height` at the pass's own tiles,
   `carve_fjords` marks `Height` whole-map, `paint_commit` marks `Civ` — and
   therefore correctly re-runs *nothing*, since a mid-chain edit does not make
   its own upstreams stale. A new `#[func] recompute_stale_stages()` exposes
   the same call for deferred or batched cases. **No UI was built**; the hold
   stands, and the future wiring is tracked as `GUI_GAP_REGISTER.md` MS-06.

   **Measured `--release`: 76.5 ms @512², 97.8 ms @1024², 188.9 ms @2048²** —
   inside this document's own 131–564 ms prediction and 18.8× cheaper than the
   3.558 s full generation it replaces. Verified in the real GPU-backed
   editor, not only headless: a committed sculpt stroke moved temperature in
   48 of 92 transect cells, precipitation in 15 and drainage in 79, all of
   which were 0 before. See `cartalith-native/docs/CHANGELOG.md`, "The
   staleness graph gets its consumer".

   `param_set` is deliberately still unwired: mapping a moved dial onto the
   stage it invalidates needs a per-parameter → stage table, which is a real
   design rather than an improvisation, and is not what a commit path needs.

4. ~~**How should the erosion↔climate cycle be represented?**~~ **RESOLVED —
   owner picked candidate (a); IMPLEMENTED 2026-08-24.**

   > Erosion is part of the height stage, which internally iterates — not a
   > separate "iterate N times" stage-graph primitive.

   Against the real graph that resolves to something smaller than it sounds:

   - **The graph does not change.** No `erosion` node, no new edge, no new
     stage kind. `pipeline_stage_graph` is the same four-node graph it was.
   - **`Height` is a source node whose *body* contains the cycle.** That body
     is `generate_terrain`'s own carve-and-evolve block — the light
     stream-power pass, `isostatic_rebound`, and the `evolve_cycles` loop
     whose every iteration ends in `refresh_climate` so the next cycle's
     incision reads the rain the last cycle's orography produced. The cycle
     runs; it is invisible to the graph because it never crosses a node
     boundary, which is exactly what lets the DAG stay a DAG.
   - **So the consumer never runs erosion.** By the time `Height` is marked
     changed, height — erosion included — is whatever it is going to be.

   Candidate (b) would have needed a fixed-point iteration *between* nodes: a
   new primitive in a data structure whose whole safety argument is that
   `add_stage` requires its upstreams to already exist. The decision is pinned
   by `the_owners_erosion_decision_keeps_the_graph_at_four_acyclic_stages` in
   `cartalith-engine/src/staleness.rs`, so reversing it has to be argued
   rather than drifted into, and `staleness.rs`'s module doc now carries the
   reasoning in place of its old "what this deliberately does not model"
   section.

5. **RESOLVED (2026-08-24).** Does the owner want more feedback anywhere, or
   less? The research pointed both ways and left it to project direction. The
   reduction path is already essentially complete (§3.1). **Owner's answer:
   open to the addition path** — new opt-in coupling (erosion↔vegetation,
   cryosphere↔albedo, dynamic lithology; the latter two already listed as
   documented follow-ups in this repo's own `system-coupling-audit.md`) is
   approved as a standing pattern, on the same off-by-default /
   physically-justified / cost-only-when-enabled terms `ErosionPassParams`
   already demonstrates. Recorded durably as `DECISIONS.md` §7g — this was the
   one owner answer from this document that `PARITY_AUDIT.md` pass 2 (finding
   F3) found existed only in conversation, with no home in the repo. It has
   one now.

6. **DEFERRED, not decided (2026-08-24).** Is `compute_civilisation` in the
   right crate? It is the second half of the generation pipeline and it lives
   in `cartalith-godot`, not `cartalith-engine`. Out of scope for this
   document's core question, but it is what makes the pipeline hard to see as
   one thing, and it would matter for §3.2.4. Owner's answer: not a priority
   right now. `compute_civilisation` remains in `cartalith-godot/src/lib.rs`;
   raised, not acted on.

---

## Sources

**External, retrieved and read**

- [Landlab user guide — Time steps](https://landlab.readthedocs.io/en/latest/user_guide/time_steps.html)
- [CESM — CPL Version 7.0](https://www.cesm.ucar.edu/models/cpl/7.0)
- [Dwarf Fortress Wiki — World generation](https://dwarffortresswiki.org/index.php/World_generation)
- [Gaea documentation — Baking and Caching](https://docs.quadspinner.com/Guide/Using-Gaea/Cache.html)
- [FastFlow: GPU Acceleration of Flow and Depression Routing for Landscape Simulation (Jain et al., CGF 43, 2024)](https://onlinelibrary.wiley.com/doi/10.1111/cgf.15243)
- [Physically-based analytical erosion for fast terrain generation — abstract (Physics-Based Animation)](https://www.physicsbasedanimation.com/2024/05/04/physically-based-analytical-erosion-for-fast-terrain-generation/)

**External, abstract-level or via search summary (marked `[source·indirect]` in text)**

- [Braun & Willett 2013, *A very efficient O(n), implicit and parallel method to solve the stream power equation*, Geomorphology 180:170–179](https://www.sciencedirect.com/science/article/abs/pii/S0169555X12004618)
- [Whipple & Tucker 1999, JGR Solid Earth 104(B8)](https://agupubs.onlinelibrary.wiley.com/doi/10.1029/1999JB900120)
- [Yuan et al. 2019, *A New Efficient Method to Solve the Stream Power Law Model Taking Into Account Sediment Deposition*, JGR Earth Surface](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2018JF004867)
- [Shobe, Tucker & Barnhart 2017, *The SPACE 1.0 model*, Geosci. Model Dev. 10:4577](https://gmd.copernicus.org/articles/10/4577/2017/gmd-10-4577-2017.pdf)
- [FastScape](https://fastscape.org/) · [FastScapeLib documentation](https://fastscape.org/fastscapelib-fortran/) · [GFZ FastScape project page](https://www.gfz.de/en/section/earth-surface-process-modelling/projects/current-projects/fastscape-landscape-evolution-model-development)
- [Craig, Vertenstein & Jacob 2012, *A New Flexible Coupler for Earth System Modeling developed for CCSM4 and CESM1* (PDF 403'd on fetch)](https://www.mcs.anl.gov/uploads/cels/papers/P1838.pdf)
- [Cordonnier et al. 2016, *Large Scale Terrain Generation from Tectonic Uplift and Fluvial Erosion*, CGF 35(2)](https://onlinelibrary.wiley.com/doi/10.1111/cgf.12820)
- [Cordonnier et al. 2017, *Authoring Landscapes by Combining Ecosystem and Terrain Erosion Simulation*, ACM TOG 36(4):134](https://dl.acm.org/doi/10.1145/3072959.3073667)
- [Schott et al. 2023, *Large-scale Terrain Authoring through Interactive Erosion Simulation*, ACM TOG 42(5):162](https://dl.acm.org/doi/10.1145/3592787) · [code](https://github.com/H-Schott/StreamPowerErosion)
- [Schott et al. 2024, *Terrain Amplification using Multi-scale Erosion*, ACM TOG 43(4)](https://dl.acm.org/doi/abs/10.1145/3658200) · [HAL record](https://hal.science/hal-04565030) · [code](https://github.com/H-Schott/MultiScaleErosion)
- [Tzathas, Gailleton, Steer & Cordonnier 2024, *Physically-based analytical erosion for fast terrain generation*, CGF 43(2)](https://onlinelibrary.wiley.com/doi/abs/10.1111/cgf.15033)
- [Allen 2008, *Time scales of tectonic landscapes and their sediment routing systems*, Geol. Soc. London Spec. Pub. 296](https://www.lyellcollection.org/doi/abs/10.1144/sp296.2)
- [Smith & Barstad 2004, *A Linear Theory of Orographic Precipitation*, J. Atmos. Sci. 61:1377](https://journals.ametsoc.org/view/journals/atsc/61/12/1520-0469_2004_061_1377_altoop_2.0.co_2.xml)
- [Tarn Adams, *Simulation Principles from Dwarf Fortress*, Game AI Pro 2, ch. 41](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf)
- [redblobgames/mapgen4](https://github.com/redblobgames/mapgen4) · [Mapgen4](https://www.redblobgames.com/maps/mapgen4/) · [DeepWiki summary (third-party, generated — indicative only)](https://deepwiki.com/redblobgames/mapgen4/4-map-generation)
- [World Machine](https://www.world-machine.com/)

**In-repository**

- `cartalith-native/crates/cartalith-engine/src/lib.rs` — `generate_terrain`, `refresh_climate`, `ErosionPassParams`, `WorldParams::defaults`
- `cartalith-native/crates/cartalith-engine/src/staleness.rs` — `pipeline_stage_graph`, and its "unwired on purpose" note
- `cartalith-native/crates/cartalith-engine/src/sculpt_commit.rs` — the five-step commit and its "staleness" section
- `cartalith-native/crates/cartalith-godot/src/lib.rs` — `compute_civilisation`, `absorb`, `sculpt_commit`, `carve_fjords`, `center_landmasses`, `paint_commit`
- `cartalith-native/crates/cartalith-hydrology/src/lib.rs` — `compute_flow`, `flow_cmp_desc`
- `cartalith-native/crates/cartalith-climate/src/lib.rs` — `simulate_weather`, `apply_ocean_currents`, `ocean_sst_anomaly`
- `cartalith-native/godot-project/shell/engine_bridge.gd` — `mark_dirty`
- `docs/research/pipeline-order-audit.md` · `docs/research/system-coupling-audit.md` · `docs/research/performance-audit-gen1.md`
- `CPU_MULTITHREADING_SCOPE.md` · `SCULPT_LIVE_SCOPE.md` · `GPU_LAYER_INTEGRATION_SCOPE.md` · `TERRAIN_ARCHITECTURE_RESEARCH.md` · `ARCHITECTURE.md` · `cartalith-native/docs/CHANGELOG.md`
